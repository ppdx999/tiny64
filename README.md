# Tiny64 – Time-Ordered Compact Unique IDs

Tiny64 is a compact 64-bit identifier format designed for high-performance systems that require **time-sortable unique IDs** with **low collision probability** and **efficient generation** (even from shell scripts).

Tiny64 encodes a 64-bit integer using **Base64 URL-safe characters**, producing a short **11-character string** that preserves time order in lexical sorting.

---

## ✅ Key Features

* **Short** – Only 11 characters (Base64 URL-safe)
* **Time-sortable** – IDs sort chronologically as strings
* **Low collision rate** – Timestamp + sequence + randomness
* **Fast generation** – Suitable for Bash, shell scripts, or lightweight services
* **Distributed safe** – Works in multi-process environments with atomic file locking
* **Zero external dependencies**

---

## 🔧 Encoding Specification (Tiny64 v1)

Tiny64 is constructed as a 64-bit unsigned integer with the following bit layout:

```
[ 42 bits: timestamp (ms since Unix epoch) ]
[ 12 bits: sequence number      ]
[ 10 bits: randomness           ]
```

| Field        | Size   | Description                                     |
| ------------ | ------ | ----------------------------------------------- |
| timestamp_ms | 42 bit | Milliseconds since 1970-01-01 UTC               |
| sequence     | 12 bit | Incremented if multiple IDs in same ms          |
| random       | 10 bit | Extra entropy to avoid cross-process collisions |

---

## 🔤 String Representation

| Property        | Value                                           |
| --------------- | ----------------------------------------------- |
| Length          | 11 characters                                   |
| Encoding        | Base64 URL-safe (`A–Z a–z 0–9 - _`)             |
| Padding         | No padding (`=` removed)                        |
| Lexical order   | Matches chronological order                     |
| Collision model | Extremely low (up to 4096 IDs/ms per generator) |

Tiny64 uses **big-endian encoding** for the 64-bit value. Base64 encoding is performed without padding.

---

## ✅ Generation Algorithm

Pseudo-code:

```
state: last_time_ms = 0, sequence = 0

function generateTiny64():
    now = current_unix_time_ms()
    if now == last_time_ms:
        sequence = (sequence + 1) mod 4096
        if sequence == 0:
            wait until next millisecond
    else:
        sequence = 0

    random10 = secure_random(0..1023)

    value = (now << 22) | (sequence << 10) | random10
    return base64url_encode(value_as_uint64_be).strip('=')
```

---

## ✅ Properties

| Property           | Behavior                                  |
| ------------------ | ----------------------------------------- |
| Time monotonicity  | Yes – lexical order = time order          |
| High throughput    | Up to 4096 IDs/ms per process             |
| Distributed safety | Add `machine_id` bits if needed           |
| Shell compatible   | Yes – works in POSIX sh environments      |
| Collision handling | Detectable via atomic `mkdir` reservation |

---

## 🚫 Non-Goals

Tiny64 is **not** designed for:

* Cryptographic security
* Hiding timestamps
* UUID compatibility
* Global hard guarantees without collision checking

If you need cryptographic or secure identifiers, use UUIDv7 or ULID instead.

---

## ⚙️ Example Use Cases

* Event ordering
* Distributed logs
* Database primary keys
* File-based lock identifiers
* Message or job IDs
* Web object IDs (/users/<tiny64_id>)

---

## 🔋 Examples

```
B8JFx1n0GMj
B8JFx1n0GMk
B8JFx1n0GMl
```

---

## ✅ Contributing

Contributions are welcome. Before submitting pull requests:

* Keep the Tiny64 core spec stable
* Avoid adding unnecessary dependencies
* Ensure deterministic behavior across environments

---

## 📜 License

MIT License. See `LICENSE`.

