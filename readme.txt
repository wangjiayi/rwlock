{\rtf1\ansi\ansicpg1252\cocoartf1504\cocoasubrtf820
{\fonttbl\f0\fnil\fcharset0 HelveticaNeue;\f1\ftech\fcharset77 Symbol;}
{\colortbl;\red255\green255\blue255;\red64\green64\blue64;}
{\*\expandedcolortbl;;\csgenericrgb\c25098\c25098\c25098;}
{\*\listtable{\list\listtemplateid1\listhybrid{\listlevel\levelnfc23\levelnfcn23\leveljc0\leveljcn0\levelfollow0\levelstartat1\levelspace360\levelindent0{\*\levelmarker \{disc\}}{\leveltext\leveltemplateid1\'01\uc0\u8226 ;}{\levelnumbers;}\fi-360\li720\lin720 }{\listname ;}\listid1}}
{\*\listoverridetable{\listoverride\listid1\listoverridecount0\ls1}}
\margl1440\margr1440\vieww10800\viewh8400\viewkind0
\deftab720
\pard\pardeftab720\ri0\sb450\sa450\partightenfactor0

\f0\fs24 \cf2 RWLock allows multiple readers threads to simultaneously access the data. But only one writer is allowed to do so at any one time. As usual, to writer such a lock, we need a lock (maybe more) and conditional variables.\
What are so special of this lock that we need to consider\uc0\u8232 carefully ? What are the variables and data structures we may use to implement it ?\
\pard\tx720\pardeftab720\li720\fi-360\ri0\sb100\sa100\partightenfactor0
\ls1\ilvl0
\f1\fs20 \cf2 \'a5	
\f0\fs24 Firstly, we need to consider when the readers/writers should wait. For a reader, it is definitely not allowed to read the data if there is some active writer modifying the shared data and vice versa.\
\ls1\ilvl0
\f1\fs20 \'a5	
\f0\fs24 Secondly, we have to think about how to wake up other threads when the current reader/writer is done. In this homework we are required to implement reader-prefered/writer-prefered and FIFO/LIFO. By the way, the rwlock in std is not so complicated.\
\ls1\ilvl0
\f1\fs20 \'a5	
\f0\fs24 Finally, to those who are not familiar with Rust yet, it will be suffering to write it.\
}