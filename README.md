# native-hw-action-server

A simple server that can be commanded to perform various mouse actions. It was built as a last resort to execute mouse actions for e2e tests with cypress, as the events would not work on the target application.

## Usage
Start the server with cargo run or build and run the binary. The server will listen on port 8080 by default. Parameters are optional.

```bash
cargo run -- --port 8080 --ip 127.0.0.1
```

## Endpoint
The server has a single endpoint that accepts POST requests. The endpoint is /action and the request body should be a JSON object with the following structure:

POST /mouse-actions
```json
{
  actions: [
    { MouseMove: [start.x, start.y] },
    { MouseDown: 'Left' },
    { MouseMove: [end.x, end.y] },
    { MouseUp: 'Left' },
  ],
  delay_between: 1000,
}
```

The actions array can contain any number of actions, and the delay_between parameter is optional. It will add a delay between each action in the array.
