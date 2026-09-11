// Shared minimal CDP client for packaged verifiers: attach to a WebView2
// page target and evaluate expressions. Kept deliberately small; verify_041
// carries its own inline copy for historical reasons.
import WebSocket from "ws";

export async function attach(cdpPort, { deadlineMs = 90_000 } = {}) {
  const deadline = Date.now() + deadlineMs;
  for (;;) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${cdpPort}/json/list`)).json();
      const page = list.find(
        (target) => target.type === "page" && target.webSocketDebuggerUrl,
      );
      if (page) {
        const socket = new WebSocket(page.webSocketDebuggerUrl);
        await new Promise((resolve, reject) => {
          socket.once("open", resolve);
          socket.once("error", reject);
        });
        let nextId = 1;
        const pending = new Map();
        const exceptions = [];
        socket.on("message", (data) => {
          const message = JSON.parse(data.toString());
          if (message.method === "Runtime.exceptionThrown") {
            exceptions.push({
              text: message.params?.exceptionDetails?.exception?.description ?? message.params?.exceptionDetails?.text ?? "unknown",
              at: Date.now(),
            });
            return;
          }
          if (message.method === "Log.entryAdded" && message.params?.entry?.level === "error") {
            exceptions.push({ text: message.params.entry.text, at: Date.now() });
            return;
          }
          const waiter = pending.get(message.id);
          if (waiter) {
            pending.delete(message.id);
            waiter(message);
          }
        });
        const send = (method, params = {}) =>
          new Promise((resolve, reject) => {
            const id = nextId++;
            pending.set(id, (message) => {
              if (message.error) reject(new Error(JSON.stringify(message.error)));
              else resolve(message.result);
            });
            socket.send(JSON.stringify({ id, method, params }));
            setTimeout(() => {
              if (pending.has(id)) {
                pending.delete(id);
                reject(new Error(`CDP ${method} timed out`));
              }
            }, 120_000);
          });
        await send("Runtime.enable");
        await send("Log.enable").catch(() => {});
        return {
          socket,
          exceptions,
          send,
          async evaluate(expression, timeoutMs = 120_000) {
            const result = await Promise.race([
              send("Runtime.evaluate", {
                expression,
                awaitPromise: true,
                returnByValue: true,
                timeout: timeoutMs,
              }),
              new Promise((resolve) =>
                setTimeout(() => resolve({ __timeout: true }), timeoutMs + 5000),
              ),
            ]);
            if (result.__timeout) throw new Error("Runtime.evaluate timed out");
            if (result.exceptionDetails) {
              throw new Error(
                result.exceptionDetails.exception?.description ??
                  result.exceptionDetails.text ??
                  "evaluation failed",
              );
            }
            return result.result?.value;
          },
          close() {
            try {
              socket.close();
            } catch {
              // already closed
            }
          },
        };
      }
    } catch {
      // retry until the deadline
    }
    if (Date.now() > deadline) {
      throw new Error(`The candidate WebView did not expose a CDP page on port ${cdpPort}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}
