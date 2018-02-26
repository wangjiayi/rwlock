use std::sync::Mutex;
use std::sync::Condvar;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
// first: about the data structure information for state variables, and synchronization variables
pub struct RwLock<T>{
    critical_data: UnsafeCell<T>,
    preference: Preference,//use for reader prefer or write prefer
    order: Order,//use for FIFO or LIFO
    state_variables: UnsafeCell<Variables>,//use to store wait or write state
    mutex: Mutex<()>,
}
pub enum Preference{
    Reader,
    Writer,
}
pub enum Order{
    Fifo,
    Lifo,
}
struct Variables{
    wait_readers: Vec<Rc<Box<Condvar>>>,
    wait_writers: Vec<Rc<Box<Condvar>>>,
    active_readers: u32,
    active_writers:u32,
}
// second: imply the lock information
impl<T> RwLock<T>{
    //need to care the critical section of data, use reader or writer prefer and which method to wake up threads on waiting list
    //init all arguments information
    pub fn new(critical_data: T, preference:Preference, order:Order) -> RwLock<T>{
        RwLock{
            mutex: Mutex::new(()),
            critical_data: UnsafeCell::new(critical_data),
            preference:preference,
            order:order,
            state_variables: UnsafeCell::new(
                Variables{
                    wait_readers:Vec::new(),
                    wait_writers:Vec::new(),
                    active_readers:0,
                    active_writers:0,
                }
            )
        }
    }
    //finish method of when reader need wait, depend on reader prefer(there has active writer) or writer prefer(there has active writer or waiting writers)
    fn read_wait(&self) -> bool{
        let state_variables = self.state_variables.get();
        //here is vec, in heap, so need borrow, also smart pointer change it
        let wait_writers = unsafe{&((*state_variables).wait_writers)};
        //here is u32, in stack, only smart pointer change it
        let active_writers = unsafe{(*state_variables).active_writers};
        match self.preference{
            Preference::Reader =>{return active_writers > 0;},
            Preference::Writer =>{return active_writers > 0 || wait_writers.len() > 0;}
        }
    }
    //finish method of when writer need wait,depend on reader prefer(there has active writer, active reader, waiting readers) or writer prefer(there has active writer or active reader)
    fn write_wait(&self)->bool{
        let state_variables = self.state_variables.get();
        //here is vec, in heap, so need borrow, also smart pointer change it
        let wait_readers = unsafe{&((*state_variables).wait_readers)};
        //here is u32, in stack, only smart pointer change it
        let active_writers = unsafe{(*state_variables).active_writers};
        let active_readers = unsafe{(*state_variables).active_readers};
        match self.preference{
            Preference::Reader => {return active_writers > 0 || wait_readers.len() > 0 || active_readers > 0;},
            Preference::Writer => {return active_writers > 0 || active_readers > 0;}
        }
    }

    // finsh method reader to read, but readers need to follow fifo or lifo

    pub fn read(&self) -> Result<RwLockReadGuard<T>,()>{
        let mut guard = self.mutex.lock().unwrap();
        let condition_variable = Rc::new(Box::new(Condvar::new()
        ));
        //push condition variable to state of variables's waiting readers
        unsafe {
            (*self.state_variables.get()).wait_readers.push(condition_variable.clone());
        }
        // use loop to check if need to wait, if need ,put it to condition variable and put the lock to release it
        while self.read_wait(){
            guard = condition_variable.wait(guard).unwrap();
        }
        //check follow fifo or lifo to pop readers from condition variables
        unsafe {
            match self.order{
                Order::Fifo => {(*self.state_variables.get()).wait_readers.remove(0);},
                Order::Lifo => {(*self.state_variables.get()).wait_readers.pop();}
            }
            //makesure after wake up someone then add 1 to active readers
            (*self.state_variables.get()).active_readers += 1;
        }
        Ok(
            RwLockReadGuard{lock: &self}
        )

    }
    //finish method writer to write, similar like read
    pub fn write(&self) -> Result<RwLockWriteGuard<T>,()>{
        let mut guard = self.mutex.lock().unwrap();
        let condition_variable = Rc::new(Box::new(Condvar::new()));
        unsafe {(*self.state_variables.get()).wait_writers.push(condition_variable.clone());}
        while self.write_wait(){
            guard = condition_variable.wait(guard).unwrap();
        }
        unsafe {
            match self.order{
                Order::Fifo => {(*self.state_variables.get()).wait_writers.remove(0);},
                Order::Lifo => {(*self.state_variables.get()).wait_writers.pop();}
            }
            (*self.state_variables.get()).active_writers += 1;
        }
        Ok(
            RwLockWriteGuard{
                lock: &self
            }
        )
    }

    //finish method to wake up threads
    pub fn wake_threads(&self){
        match self.preference{
            //if reader prefer, wake up readers first
            Preference::Reader => {
                unsafe{
                    let ref mut wait_readers = (*self.state_variables.get()).wait_readers;
                    let ref mut wait_writers = (*self.state_variables.get()).wait_writers;
                    match self.order{
                        Order::Lifo => {
                            if wait_readers.len() > 0{
                                for i in 0..wait_readers.len(){
                                    wait_readers[wait_readers.len()-1-i].notify_one();
                                }
                            }else if wait_writers.len() > 0 {
                                wait_writers[wait_writers.len()-1].notify_one();
                            }
                        },
                        Order::Fifo => {
                            if wait_readers.len() > 0{
                                for i in 0..wait_readers.len() {
                                    wait_readers[i].notify_one();
                                }
                            }else if wait_writers.len() >0 {
                                wait_writers[0].notify_one();
                            }
                        }
                    }
                }
            },
            //if writer prefer, wake up writer first
            Preference::Writer => {
                unsafe {
                    let ref mut wait_readers = (*self.state_variables.get()).wait_readers;
                    let ref mut wait_writers = (*self.state_variables.get()).wait_writers;
                    match self.order{
                        Order::Lifo => {
                            if wait_writers.len() > 0{
                                wait_writers[wait_writers.len()-1].notify_one();
                            }else if wait_readers.len() > 0 {
                                for i in 0..wait_readers.len(){
                                    wait_readers[wait_readers.len()-1-i].notify_one();
                                }
                            }
                        },
                        Order::Fifo => {
                            if wait_writers.len() >0{
                                wait_writers[0].notify_one();
                            }else if wait_readers.len() > 0{
                                for i in 0..wait_readers.len() {
                                    wait_readers[i].notify_one();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

//declares that it is safe to send and reference 'RwLock' between thread
unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}
//A read guard for  RwLock
pub struct RwLockReadGuard<'a, T: 'a>{
    lock: &'a RwLock<T>
}
//provides access to the shared object
impl<'a, T> Deref for RwLockReadGuard<'a,T>{
    type Target = T;
    fn deref(&self) -> &T{
        unsafe{
            &*self.lock.critical_data.get()
        }
    }
}
// Releases the read lock
impl<'a, T> Drop for RwLockReadGuard<'a, T>{
    fn drop(&mut self){
        let guard = self.lock.mutex.lock().unwrap();
        unsafe{
            if(*self.lock.state_variables.get()).active_readers > 0 {
                (*self.lock.state_variables.get()).active_readers -= 1;
            }
            self.lock.wake_threads();
        }
    }
}
// A write guard for 'RwLock'
pub struct RwLockWriteGuard<'a, T: 'a>{
    lock: &'a RwLock<T>
}
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{
            &*self.lock.critical_data.get()
        }
    }
}
//procides accsee to the shared object
impl<'a, T> DerefMut for RwLockWriteGuard<'a,T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe{
            &mut *self.lock.critical_data.get()
        }
    }
}
//releases the write lock
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self){
        let guard = self.lock.mutex.lock().unwrap();
        unsafe {
            if (*self.lock.state_variables.get()).active_writers > 0 {
                (*self.lock.state_variables.get()).active_writers -= 1;
            }
            self.lock.wake_threads();
        }
    }
}

