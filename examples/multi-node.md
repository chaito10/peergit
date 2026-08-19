# Example: Multi-Node Network

Set up a 3-node PeerGit network on one machine for testing.

## Overview

We'll create three nodes: Alice, Bob, and Carol, all running on localhost with different ports.

```
Alice (:4001) <---> Bob (:4002) <---> Carol (:4003)
     \                                     /
      \-----------------------------------/
```

## Step 1: Create Home Directories

=== "Linux / macOS"

    ```bash
    mkdir -p /tmp/peergit-alice /tmp/peergit-bob /tmp/peergit-carol
    ```

=== "Windows (PowerShell)"

    ```powershell
    mkdir C:\peergit-test\node-a -Force
    mkdir C:\peergit-test\node-b -Force
    mkdir C:\peergit-test\node-c -Force
    ```

## Step 2: Initialize Each Node

=== "Linux / macOS"

    ```bash
    # Alice
    PEERGIT_HOME=/tmp/peergit-alice peergit init
    PEERGIT_HOME=/tmp/peergit-alice peergit config set node.alias alice

    # Bob
    PEERGIT_HOME=/tmp/peergit-bob peergit init
    PEERGIT_HOME=/tmp/peergit-bob peergit config set node.alias bob

    # Carol
    PEERGIT_HOME=/tmp/peergit-carol peergit init
    PEERGIT_HOME=/tmp/peergit-carol peergit config set node.alias carol
    ```

=== "Windows (PowerShell)"

    ```powershell
    # Alice
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit init
    peergit config set node.alias alice

    # Bob
    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit init
    peergit config set node.alias bob

    # Carol
    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit init
    peergit config set node.alias carol
    ```

## Step 3: Get Each Node's Identity

You need to run these in separate terminals, or run them one by one and note the output:

=== "Linux / macOS"

    ```bash
    PEERGIT_HOME=/tmp/peergit-alice peergit identity
    # Note: ALICE_PEER_ID=...  ALICE_PUBKEY=...

    PEERGIT_HOME=/tmp/peergit-bob peergit identity
    # Note: BOB_PEER_ID=...  BOB_PUBKEY=...

    PEERGIT_HOME=/tmp/peergit-carol peergit identity
    # Note: CAROL_PEER_ID=...  CAROL_PUBKEY=...
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit identity
    # Note: ALICE_PEER_ID=...  ALICE_PUBKEY=...

    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit identity
    # Note: BOB_PEER_ID=...  BOB_PUBKEY=...

    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit identity
    # Note: CAROL_PEER_ID=...  CAROL_PUBKEY=...
    ```

## Step 4: Configure Listening Ports

=== "Linux / macOS"

    ```bash
    PEERGIT_HOME=/tmp/peergit-alice peergit config set p2p.listen '["/ip4/127.0.0.1/tcp/4001"]'
    PEERGIT_HOME=/tmp/peergit-bob peergit config set p2p.listen '["/ip4/127.0.0.1/tcp/4002"]'
    PEERGIT_HOME=/tmp/peergit-carol peergit config set p2p.listen '["/ip4/127.0.0.1/tcp/4003"]'
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit config set p2p.listen '"/ip4/127.0.0.1/tcp/4001"'

    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit config set p2p.listen '"/ip4/127.0.0.1/tcp/4002"'

    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit config set p2p.listen '"/ip4/127.0.0.1/tcp/4003"'
    ```

## Step 5: Configure Web Dashboard Ports

=== "Linux / macOS"

    ```bash
    PEERGIT_HOME=/tmp/peergit-alice peergit config set fossil.web_port 3001
    PEERGIT_HOME=/tmp/peergit-bob peergit config set fossil.web_port 3002
    PEERGIT_HOME=/tmp/peergit-carol peergit config set fossil.web_port 3003
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit config set fossil.web_port 3001

    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit config set fossil.web_port 3002

    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit config set fossil.web_port 3003
    ```

## Step 6: Add Peers

Replace the placeholder values with the actual PeerIds and Public Keys from Step 3.

=== "Linux / macOS"

    ```bash
    # Alice knows Bob
    PEERGIT_HOME=/tmp/peergit-alice peergit peer add <BOB_PUBKEY> \
      --alias bob \
      --addresses "/ip4/127.0.0.1/tcp/4002/p2p/<BOB_PEER_ID>"

    # Bob knows Alice and Carol
    PEERGIT_HOME=/tmp/peergit-bob peergit peer add <ALICE_PUBKEY> \
      --alias alice \
      --addresses "/ip4/127.0.0.1/tcp/4001/p2p/<ALICE_PEER_ID>"

    PEERGIT_HOME=/tmp/peergit-bob peergit peer add <CAROL_PUBKEY> \
      --alias carol \
      --addresses "/ip4/127.0.0.1/tcp/4003/p2p/<CAROL_PEER_ID>"

    # Carol knows Bob
    PEERGIT_HOME=/tmp/peergit-carol peergit peer add <BOB_PUBKEY> \
      --alias bob \
      --addresses "/ip4/127.0.0.1/tcp/4002/p2p/<BOB_PEER_ID>"
    ```

=== "Windows (PowerShell)"

    ```powershell
    # Alice knows Bob
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit peer add <BOB_PUBKEY> --alias bob --addresses "/ip4/127.0.0.1/tcp/4002/p2p/<BOB_PEER_ID>"

    # Bob knows Alice and Carol
    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit peer add <ALICE_PUBKEY> --alias alice --addresses "/ip4/127.0.0.1/tcp/4001/p2p/<ALICE_PEER_ID>"
    peergit peer add <CAROL_PUBKEY> --alias carol --addresses "/ip4/127.0.0.1/tcp/4003/p2p/<CAROL_PEER_ID>"

    # Carol knows Bob
    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit peer add <BOB_PUBKEY> --alias bob --addresses "/ip4/127.0.0.1/tcp/4002/p2p/<BOB_PEER_ID>"
    ```

## Step 7: Start All Nodes

Open three terminals:

=== "Linux / macOS"

    ```bash
    # Terminal 1 - Alice
    PEERGIT_HOME=/tmp/peergit-alice peergit node start

    # Terminal 2 - Bob
    PEERGIT_HOME=/tmp/peergit-bob peergit node start

    # Terminal 3 - Carol
    PEERGIT_HOME=/tmp/peergit-carol peergit node start
    ```

=== "Windows (PowerShell)"

    ```powershell
    # Terminal 1 - Alice
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit node start

    # Terminal 2 - Bob
    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit node start

    # Terminal 3 - Carol
    $env:PEERGIT_HOME="C:\peergit-test\node-c"
    peergit node start
    ```

## Step 8: Verify the Network

Check each node's status:

=== "Linux / macOS"

    ```bash
    PEERGIT_HOME=/tmp/peergit-alice peergit node status
    PEERGIT_HOME=/tmp/peergit-bob peergit node status
    PEERGIT_HOME=/tmp/peergit-carol peergit node status
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PEERGIT_HOME="C:\peergit-test\node-a"; peergit node status
    $env:PEERGIT_HOME="C:\peergit-test\node-b"; peergit node status
    $env:PEERGIT_HOME="C:\peergit-test\node-c"; peergit node status
    ```

Alice should see 1 connected peer (Bob).
Bob should see 2 connected peers (Alice and Carol).
Carol should see 1 connected peer (Bob).

## Step 9: Publish and Sync a Repo

=== "Linux / macOS"

    ```bash
    # Create a Fossil repo
    fossil init /tmp/test-repo.fossil

    # Publish from Alice
    PEERGIT_HOME=/tmp/peergit-alice peergit repo publish \
      --path /tmp/test-repo.fossil --name test-repo

    # Discover from Bob
    PEERGIT_HOME=/tmp/peergit-bob peergit repo discover --name test-repo
    ```

=== "Windows (PowerShell)"

    ```powershell
    # Create a Fossil repo
    fossil init C:\peergit-test\test-repo.fossil

    # Publish from Alice
    $env:PEERGIT_HOME="C:\peergit-test\node-a"
    peergit repo publish --path C:\peergit-test\test-repo.fossil --name test-repo

    # Discover from Bob
    $env:PEERGIT_HOME="C:\peergit-test\node-b"
    peergit repo discover --name test-repo
    ```

## Step 10: Open Dashboards

- Alice: http://localhost:3001
- Bob: http://localhost:3002
- Carol: http://localhost:3003

Each dashboard shows its own node's status and connected peers.

## Cleanup

=== "Linux / macOS"

    ```bash
    rm -rf /tmp/peergit-alice /tmp/peergit-bob /tmp/peergit-carol /tmp/test-repo.fossil
    ```

=== "Windows (PowerShell)"

    ```powershell
    Remove-Item -Recurse -Force C:\peergit-test
    ```
