(function () {
    function Rpc() {
        const self = this;
        // FIXME: add promise counter
        this._promises = {};
        this._callbacks = {};

        // Private internal function called on error
        this._error = (id, error) => {
            if (this._promises[id]) {
                this._promises[id].reject(error);
                delete this._promises[id];
            }
        }

        // Private internal function called on result
        this._result = (id, result) => {
            if (this._promises[id]) {
                this._promises[id].resolve(result);
                delete this._promises[id];
            }
        }

        // Call remote method and expect a reply from the handler
        this.call = function (method) {
            let array = new Uint32Array(1);
            window.crypto.getRandomValues(array);
            const id = array[0];
            const params = Array.prototype.slice.call(arguments, 1);
            const payload = { jsonrpc: "2.0", id, method, params };
            const promise = new Promise((resolve, reject) => {
                self._promises[id] = { resolve, reject };
            });
            window.external.invoke(JSON.stringify(payload));
            return promise;
        }

        // Send a notification without an `id` so no reply is expected.
        this.notify = function (method) {
            const params = Array.prototype.slice.call(arguments, 1);
            const payload = { jsonrpc: "2.0", method, params };
            window.external.invoke(JSON.stringify(payload));
            return Promise.resolve();
        }

        // Register a callback
        this.on = function (method, cb) {
            if (!this._callbacks[method]) {
                this._callbacks[method] = [cb];
            } else {
                this._callbacks[method].append(cb);
            }
        }

        // Deregister callback
        this.removeListener = function (method, cb) {
            for (var i = 0; i < this._callbacks[method].length; i++) {
                if (this._callbacks[method] == cb) {
                    delete this._callbacks[method][i]
                    break;
                }
            }
        }

        // Receive a message and broadcast to callbacks
        this._message = function (method, message) {
            if (!this._callbacks[method]) {
                return;
            }

            for (var i = 0; i < this._callbacks[method].length; i++) {
                this._callbacks[method][i](message);
            }
        }
    }

    window.external = window.external || {};
    window.external.rpc = new Rpc();
    window.rpc = window.external.rpc;

    window.rpc.notify("_webview", { initialize: null })
})();
