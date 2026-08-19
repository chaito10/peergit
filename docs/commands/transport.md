# peergit transport

Transport adapter command called by Fossil's `--transport-command`.

---

## Usage

```bash
peergit transport <URL> <REQUEST_FILE> <REPLY_FILE>
```

---

## Description

This command is not meant to be run directly. It is called by Fossil when using:

```bash
fossil sync --transport-command "peergit transport" <REMOTE_URL>
```

### How It Works

1. Fossil writes an HTTP xfer request to `<REQUEST_FILE>`
2. Fossil calls `peergit transport <url> <request_file> <reply_file>`
3. PeerGit reads the request, connects to the remote peer via libp2p
4. Sends the request over the `/peergit/xfer/1.0` protocol
5. Receives the response
6. Writes the response to `<REPLY_FILE>`
7. Fossil reads the reply and processes the response

### Arguments

| Argument | Description |
|----------|-------------|
| `<URL>` | libp2p multiaddress of the remote peer |
| `<REQUEST_FILE>` | Path to the HTTP xfer request file |
| `<REPLY_FILE>` | Path to write the HTTP xfer response |

---

## See Also

- [Fossil Sync Concept](../concepts/fossil-sync.md) -- How Fossil sync works
- [Architecture](../architecture.md) -- Transport adapter design
