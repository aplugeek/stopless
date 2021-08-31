# Stopless
[![license](https://img.shields.io/github/license/mashape/apistatus.svg)](https://github.com/i-pva/stopless/blob/master/LICENSE)

A Serverless Application Cargo Ship 

Features include:
- Launch on connection in
- Exit on idle
- Graceful shutdown server when timer exit

## Design:

LOCI: Launch On Connection In

TODO：design arch

## Usage
```
$ ./loci --help
Stopless for serverless 1.0
Jerry <Aplugeek@outlook.com>
Does awesome things

USAGE:
    loci [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --command <exe>             server boot command
    -e, --endpoint <server_bind>    server bind address

```
### Start loci
```
./loci -e localhost:8080 -c ./server
```
### Curl loci endpoint
`curl -I localhost:8080`

```
[2021-04-02T04:51:30Z INFO  loci::bootor] Starting loci...
[2021-04-02T04:51:32Z INFO  server] Connection incoming,reset exit timer
[2021-04-02T04:51:33Z INFO  server::timer] Exit timer count:9
[2021-04-02T04:51:34Z INFO  server::timer] Exit timer count:8
[2021-04-02T04:51:35Z INFO  server::timer] Exit timer count:7
[2021-04-02T04:51:36Z INFO  server::timer] Exit timer count:6
[2021-04-02T04:51:37Z INFO  server::timer] Exit timer count:5
[2021-04-02T04:51:38Z INFO  server::timer] Exit timer count:4
[2021-04-02T04:51:39Z INFO  server::timer] Exit timer count:3
[2021-04-02T04:51:40Z INFO  server::timer] Exit timer count:2
[2021-04-02T04:51:41Z INFO  server::timer] Exit timer count:1
[2021-04-02T04:51:42Z INFO  server::timer] Exit timer count:0
[2021-04-02T04:51:43Z INFO  server::timer] Graceful shutdown server
[2021-04-02T04:51:43Z INFO  server] Server has graceful shutdown!
[2021-04-02T04:51:43Z INFO  loci::bootor] Server has stopped
[2021-04-02T04:51:43Z INFO  loci::bootor] Starting loci...

```
