# Questions per tool (arch-emulator)

Ask each question of the code and config. Answer with `file:line`, or
"not shown in this repo". Limits change: take every number from the
provider's current docs and write the date you checked. Never give a
provider limit from memory.

## Every part

- Can it run as more than one copy (stateless)? What state does it keep
  in memory or on local disk?
- What fills up first: CPU, memory, connections, threads, file handles,
  disk, a provider quota?
- Is there a timeout on every outbound call? Retries with backoff, jitter
  and a cap?
- What does it do when a dependency is slow (not down, slow)?
- How does it start: cold start time, warm-up, a cache it fills first?
- What does it log and measure: latency percentiles, errors, saturation?

## Calls that wait (HTTP, gRPC)

- How many calls in series does one user request make? Can some run in
  parallel, or move to a queue?
- Is a connection pool reused, or does each call open a new connection
  (TLS handshake each time)?
- Inner timeouts shorter than outer ones? A gateway or load balancer
  timeout that cuts long requests?
- Request and response size limits; large payloads that should go to
  object storage with a link.
- Rate limits of outside APIs; what happens on HTTP 429.
- Pros: simple, the caller gets the answer at once. Cons: the caller
  waits, a failure spreads up the chain, both sides must scale together.

## Queues (SQS, RabbitMQ, Kafka, Pub/Sub, Redis lists)

- Standard or FIFO (ordered) queue? Is order really needed? Ordered
  queues have a much lower throughput limit per group: check the docs.
- Is each consumer idempotent? Most queues deliver a message **at least
  once**, so a message can come twice.
- Visibility timeout (SQS) or ack deadline longer than the longest job?
  Otherwise a slow job runs twice.
- Dead-letter queue and a max receive count? Who looks at it?
- Batching on send and receive; long polling instead of empty polls.
- Message size limit; large data sent as a link.
- Consumer concurrency, and what limits it (a database pool behind it).
- Alarm on queue age (the oldest message), not only queue length.
- Pros: absorbs spikes, decouples producer and consumer, retries for
  free. Cons: no direct answer to the caller, harder to trace, duplicates
  and (often) no order.

## Databases

- Indexes for every frequent `WHERE`, `JOIN` and `ORDER BY`. N+1 queries
  in a loop.
- Connection pool size × number of copies ≤ the database's connection
  limit? A proxy or pooler (PgBouncer, RDS Proxy) when many copies or
  functions connect.
- Long transactions and locks; hot rows that many requests update.
- Reads that could go to a replica or a cache; writes that could batch.
- Table growth: which tables grow without bound; archive or partition.
- Migrations that lock big tables.

## Caches (Redis, Memcached, CDN, in-process)

- What is cached, the TTL, and how a change makes it stale (invalidation).
- Cache stampede: many requests miss at once and all hit the database.
- Is the cache needed for correctness, or only for speed? What happens
  when it is empty or down?
- In-process caches with many copies: each copy has its own view.

## Serverless functions (Lambda, Cloud Functions, Cloud Run)

- Concurrency limit per account or function, and reserved concurrency.
- Cold starts on the path a user waits on.
- Each function copy opens its own database connections.
- Maximum run time and payload size.

## Repo and service structure

- Do the service boundaries follow the data? Two services that write the
  same tables are one service with two deploys.
- Can each service deploy alone? Shared libraries that force deploys
  together.
- Many small services for a small team (high cost to run and trace), or
  one big service whose parts need very different scale.
- Calls between services that form a cycle.
- One place for config and limits, or copies that drift.

## Tests to propose

| Test | Proves | Watch |
|---|---|---|
| Load at the expected peak | It meets the latency goal at normal peak | p95, p99, error rate |
| Spike (10x in a minute) | Autoscaling and queues absorb a burst | errors, queue age, scale-up time |
| Soak (hours at normal load) | No leaks, no slow growth | memory, connections, latency trend |
| Failure (stop or slow one dependency) | Timeouts and fallbacks work; failure does not spread | errors per part, retries |
| Backlog drain | Consumers catch up after an outage | queue age over time |
| Query plans (`EXPLAIN`) on the top queries | Indexes are used | rows scanned |

Run them on a staging copy with fake data, never on production, unless
the people who run production plan it with you.
