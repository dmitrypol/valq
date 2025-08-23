# Valkey as a job queue.

A simple job queue built on top of Valkey using a dedicated data structure. 
This project is designed to provide a lightweight and efficient job queue system that can be used for background processing tasks.

## Features
* visibility timeout - how long each consumer has to process a message before it is available for other consumers  
* queue stores the message until a consumer acknowledges it
* if a consumer crashes or times out before acknowledging, the queue re-delivers the message to the same or another consumer
* consumer can extend the visibility timeout of a message to have more time to process it
* on message completion consumer does explicit ack specifying the message ID which removed the message from the queue
* max delivery attempts - the maximum number of times a message can be delivered to consumers before it is moved to the dead letter queue (DLQ)
* dead letter queue - store messages that failed to be processed after the maximum number of delivery attempts
* retention period - how long messages are kept in the DLQ before they are automatically deleted
* delayed message delivery - push messages to the queue with optional delay in seconds

## Commands
```
valq - top level command
valq create - create new q
valq delete - delete q
valq update - update q
valq list - list all queues
valq info - info about q
valq purge - purge messages in q, dlq or delayed q
valq push - push message to q, optionally with delay
valq pop - get message from q
valq ack - ack message completion
valq extend - extend message to have more time to complete it
valq help - display help information
```

## Requirements
- Valkey 7.2 or higher 

## Questions

### How is this different from other job queues?
Valkey gives us speed, rich library ecosystem and features such as replication and persistence.  Valq module supports delayed message delivery (u64 seconds enables up to ~585 billion years in the future), which is not available in many other job queues.

### Does this need to be a module?
While it is possible to implement a job queue using Valkey's existing data structures, such as lists, sorted sets and hashes, this approach can lead to client side complexity where libraries in different languages have to implement the same logic.
By creating a dedicated module, we can provide a consistent and efficient implementation that can be used across different languages and applications.

### Does this module need to be built in Rust?
While it is possible to implement this module in C, Rust provides a safer and more modern programming environment that can help prevent common bugs and security issues.  It also gives us rich ecosystem of libraries and tools.

### Can this guarantee message delivery?
There are still certain Valkey limitations that can lead to message loss if data has not been persisted to disk or replicated to other nodes.

Inspired by https://github.com/antirez/disque