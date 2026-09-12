// Shared minimal CDP client for packaged verifiers: attach to a WebView2
// page target and evaluate expressions. Kept deliberately small; verify_041
// carries its own inline copy for historical reasons.
//
// Resilience note (R11 campaign): after long driving sessions the page
// target occasionally stops answering Runtime.evaluate while remaining
// listed. The handle rebinds to a fresh attachment and retries the
// evaluation once before the failure is treated as real.
import WebSocket from "ws";

const evaluateOnce = async (send, expression, timeoutMs) => {
  const result = await Promise.race([
    send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
      timeout: timeoutMs,
    }),
    new Promise((resolve) => setTimeout(() => resolve({ __timeout: true }), timeoutMs + 5000)),
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
};

export async function attach(cdpPort, { deadlineMs = 90_000 } = {}) {
  const deadline = Date.now() + deadlineMs;
  for (;;) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${cdpPort}/json/list`)).json();
      const page = list.find(
        (target) => target.type === "page" && target.webSocketDebuggerUrl,
      );
      if (page) {
        const makeConnection = async () => {
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
                text:
                  message.params?.exceptionDetails?.exception?.description ??
                  message.params?.exceptionDetails?.text ??
                  "unknown",
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
          return { socket, send, exceptions };
        };

        const first = await makeConnection();
        const state = { connection: first };
        await state.connection.send("Runtime.enable");
        await state.connection.send("Log.enable").catch(() => {});

        return {
          get socket() {
            return state.connection.socket;
          },
          get exceptions() {
            return state.connection.exceptions;
          },
          send: (method, params = {}) => state.connection.send(method, params),
          async evaluate(expression, timeoutMs = 120_000) {
            for (let attempt = 0; attempt < 2; attempt += 1) {
              try {
                return await evaluateOnce(state.connection.send, expression, timeoutMs);
              } catch (error) {
                const message = String(error?.message ?? "");
                const timedOut = /timed out/i.test(message);
                const closed = state.connection.socket?.readyState !== 1;
                if (attempt === 0 && (timedOut || closed)) {
                  try {
                    state.connection.socket.close();
                  } catch {
                    // already closed
                  }
                  const fresh = await attach(cdpPort, { deadlineMs: 30_000 });
                  // The fresh handle owns its own state; keep our wrapper but
                  // rebind every method through it.
                  const rebound = {
                    socket: fresh.socket,
                    send: fresh.send,
                    exceptions: fresh.exceptions,
                  };
                  state.connection = rebound;
                  continue;
                }
                throw error;
              }
            }
            throw new Error("Runtime.evaluate timed out");
          },
          close() {
            try {
              state.connection.socket.close();
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
