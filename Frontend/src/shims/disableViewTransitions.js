(function () {
  try {
    var proto = Document && Document.prototype;
    if (!proto) return;
    var desc = Object.getOwnPropertyDescriptor(proto, "startViewTransition");
    if (!desc) return; // API non disponible => rien à faire
    Object.defineProperty(proto, "startViewTransition", {
      value: function (callback) {
        try {
          if (typeof callback === "function") {
            callback();
          }
        } catch {
          // ignore
        }
        var done = Promise.resolve();
        return {
          ready: done,
          finished: done,
          updateCallbackDone: done,
          skipTransition: function () {},
        };
      },
      configurable: true,
      writable: true,
    });
  } catch {
    // ignore
  }
})();
