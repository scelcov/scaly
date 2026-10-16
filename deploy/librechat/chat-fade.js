(function () {
  if (window.__chatSectionFade) {
    return;
  }
  window.__chatSectionFade = true;

  var LEAVE_MS = 180;
  var style = document.createElement("style");
  style.textContent = [
    "@media (prefers-reduced-motion: no-preference) {",
    "  #messages-view main[role='main'] {",
    "    transition: opacity 320ms ease;",
    "  }",
    "  #messages-view main[role='main'].chat-section-fade {",
    "    opacity: 0;",
    "    transition-duration: 180ms;",
    "    pointer-events: none;",
    "  }",
    "  /* The greeting is display:inline, so its overflow:hidden does not clip.",
    "     Letters slide up from 50px and briefly stretch the chat scroller. */",
    "  #messages-view .split-parent {",
    "    display: inline-block;",
    "    max-width: 100%;",
    "    overflow: clip;",
    "    vertical-align: bottom;",
    "  }",
    "}",
  ].join("\n");
  document.head.appendChild(style);

  var passing = false;
  var lastKey = conversationKey(location.pathname);

  function reducedMotion() {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }

  function conversationKey(path) {
    var match = String(path || "").match(/\/c\/([^/?#]+)/);
    return match ? decodeURIComponent(match[1]) : "";
  }

  function chatMain() {
    return document.querySelector("#messages-view main[role='main']");
  }

  function assignedNewId(previous, next) {
    return previous === "new" && next !== "" && next !== "new";
  }

  function fadeIn() {
    var main = chatMain();
    if (!main || reducedMotion()) {
      return;
    }
    if (!main.classList.contains("chat-section-fade")) {
      main.classList.add("chat-section-fade");
      main.style.transition = "none";
      void main.offsetWidth;
      main.style.transition = "";
    }
    requestAnimationFrame(function () {
      var shown = chatMain();
      if (shown) {
        shown.classList.remove("chat-section-fade");
      }
    });
  }

  function fadeOut(main, done) {
    if (reducedMotion()) {
      done();
      return;
    }
    var finished = false;
    function finish() {
      if (finished) {
        return;
      }
      finished = true;
      main.removeEventListener("transitionend", onEnd);
      done();
    }
    function onEnd(event) {
      if (event.target === main && event.propertyName === "opacity") {
        finish();
      }
    }
    main.classList.add("chat-section-fade");
    main.addEventListener("transitionend", onEnd);
    setTimeout(finish, LEAVE_MS + 80);
  }

  function noteNavigation() {
    var next = conversationKey(location.pathname);
    var previous = lastKey;
    lastKey = next;
    if (passing || reducedMotion() || previous === next || assignedNewId(previous, next)) {
      return;
    }
    if (previous && next) {
      fadeIn();
    }
  }

  ["pushState", "replaceState"].forEach(function (method) {
    var original = history[method];
    history[method] = function () {
      var result = original.apply(this, arguments);
      noteNavigation();
      return result;
    };
  });

  window.addEventListener("popstate", noteNavigation);

  document.addEventListener(
    "click",
    function (event) {
      if (passing || reducedMotion() || event.defaultPrevented || event.button !== 0) {
        return;
      }
      if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
        return;
      }

      var next = "";
      if (event.target.closest("[data-testid='new-chat-button'], [data-testid='nav-new-chat-fab'], [data-testid='header-new-chat-button']")) {
        next = "new";
      } else {
        var row = event.target.closest("[data-testid='convo-item']");
        if (!row || event.target.closest("button, a, input, textarea, select")) {
          return;
        }
        next = row.getAttribute("data-conversation-id") || "";
      }

      var current = conversationKey(location.pathname);
      if (!next || !current || next === current) {
        return;
      }
      var main = chatMain();
      if (!main) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      var target = event.target;
      fadeOut(main, function () {
        passing = true;
        target.click();
        passing = false;
        requestAnimationFrame(function () {
          requestAnimationFrame(fadeIn);
        });
      });
    },
    true,
  );

  var boot = new MutationObserver(function () {
    if (!chatMain()) {
      return;
    }
    boot.disconnect();
    if (conversationKey(location.pathname)) {
      fadeIn();
    }
  });
  boot.observe(document.documentElement, { childList: true, subtree: true });
})();
