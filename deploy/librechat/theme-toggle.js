(function () {
  var KEY = "slack-theme";

  // Oct 2026 Slack Appearance catalog. 4-color shares are the strings published
  // by the slack.green theme generator (Slack's current System navigation,
  // Selected items, Presence, Notifications). Old 8-color strings map the same
  // way: color1, color3, color7, color8. Gradient stops and neutrals are mixed
  // from those published colors; they are not extra official Slack hex.
  // Aubergine and Banana keep this deployment's palettes (librechat.yaml plus
  // the published classic / Banana strings). No entry is an invented theme
  // identity: every name has a published 4- or 8-color string below.
  var groups = [
    {
      name: "Miles Davis",
      items: [
        ["Kind of Blue", "#4C64A6,#71A3D1,#14ADFF,#1D3F5E"],
        ["Funky Fresh", "#BEB0FF,#71D18F,#00FF50,#CDC8E8"],
        ["Jazz Club", "#9C0000,#070C52,#000261,#026B5A"],
        ["Electric Fusion", "#DCF095,#ADFFBC,#C674F2,#C674F2"],
        ["Brassy", "#FAD961,#F0E6C2,#EDCE34,#ACDFE3"],
        ["Sunglasses Inside", "#2B102E,#D1B4E0,#41B584,#C494D6"],
      ],
    },
    {
      name: "Single color",
      items: [
        ["Aubergine", "#611F69,#F9EDFF,#4CC894,#EABDFB"],
        ["Clementine", "#DC540C,#AC3D00,#007A5A,#C84600"],
        ["Banana", "#FFEB84,#FFF8D4,#FFC806,#591035"],
        ["Jade", "#178F65,#0E674D,#007A5A,#007A5A"],
        ["Lagoon", "#0071A4,#E3F8FF,#4CC894,#67D7F8"],
        ["Barbra", "#FFC4D9,#9E0B2E,#007A5A,#C01343"],
        ["Gray", "#EAEAEA,#333133,#007A5A,#C01343"],
        // Published 10-color Mood Indigo (sachinh19): nav #F8F8FA, selected #001A5E, presence/notify #2153FF.
        ["Mood Indigo", "#F8F8FA,#001A5E,#2153FF,#2153FF"],
      ],
    },
    {
      name: "Vision assistive",
      items: [
        ["Tritanopia", "#000000,#F8F8F8,#20A271,#EF3E75"],
        ["Protanopia & Deuteranopia", "#2D0033,#F9EDFF,#20A271,#EABDFB"],
      ],
    },
    {
      name: "Fun and new",
      items: [
        ["Raspberry Beret", "#F4DAFF,#9E0B2E,#83388A,#C01343"],
        ["Big Business", "#5E5D60,#F0F3FC,#7D95EB,#A0B3F3"],
        ["POG", "#F6D2E2,#FCF2D9,#FFC600,#FFD738"],
        ["Mint Chip", "#C0ECAA,#1E328F,#2840A9,#784C1F"],
        ["PB&J", "#E3CEB5,#7C2852,#943364,#C01343"],
        ["Chill Vibes", "#178F65,#E3FFFF,#4CC894,#78D7DD"],
        ["Forest Floor", "#608813,#663F18,#AA8000,#AA8000"],
        ["Slackr", "#FFC4D9,#E3F8FF,#60CA7E,#FFA3C2"],
        ["Sea Glass", "#A9EDEF,#611F69,#0D6A7B,#83388A"],
        ["Lemon Lime", "#C79600,#0B6400,#83388A,#137700"],
        ["Falling Leaves", "#8A5B2A,#FFFAE0,#F89B6C,#FE9C92"],
        ["Sunrise", "#FFE3D6,#9E0B2E,#A35200,#A86E00"],
      ],
    },
    {
      name: "Updated classics",
      items: [
        // Published 8-color Choco Mint: #544538,#42362B,#5DB09D,#FFFFFF,#4A3C30,#FFFFFF,#FFFFFF,#5DB09D
        ["Choco Mint", "#544538,#5DB09D,#FFFFFF,#5DB09D"],
        ["CMYK", "#9FE8FD,#9E0B2E,#AA8000,#0071A4"],
        ["Haberdashery", "#0B527B,#FFFAE0,#6FC54A,#FFB6BD"],
        // Published Hoth: #F8F8FA,#F8F8FA,#2D9EE0,#FFFFFF,#FFFFFF,#383F45,#60D156,#DC5960
        ["Hoth", "#F8F8FA,#2D9EE0,#60D156,#DC5960"],
        // Published Ochin: #303E4D,#2C3849,#6698C8,#FFFFFF,#4A5664,#FFFFFF,#94E864,#78AF8F
        ["Ochin", "#303E4D,#6698C8,#94E864,#78AF8F"],
        ["Sweet Treat", "#FFD6D5,#92220C,#784C1F,#007A5A"],
      ],
    },
  ];

  function clamp(n) {
    return Math.max(0, Math.min(255, Math.round(n)));
  }

  function parseHex(hex) {
    var h = hex.replace("#", "");
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
  }

  function mix(a, b, t) {
    return [
      clamp(a[0] + (b[0] - a[0]) * t),
      clamp(a[1] + (b[1] - a[1]) * t),
      clamp(a[2] + (b[2] - a[2]) * t),
    ];
  }

  function lighten(c, t) {
    return mix(c, [255, 255, 255], t);
  }

  function darken(c, t) {
    return mix(c, [0, 0, 0], t);
  }

  function channel(v) {
    v = v / 255;
    return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  }

  function lum(c) {
    return 0.2126 * channel(c[0]) + 0.7152 * channel(c[1]) + 0.0722 * channel(c[2]);
  }

  function contrast(a, b) {
    var hi = Math.max(lum(a), lum(b));
    var lo = Math.min(lum(a), lum(b));
    return (hi + 0.05) / (lo + 0.05);
  }

  function hexOf(c) {
    return (
      "#" +
      [c[0], c[1], c[2]]
        .map(function (n) {
          var s = clamp(n).toString(16);
          return s.length === 1 ? "0" + s : s;
        })
        .join("")
    );
  }

  function rgbOf(c) {
    return clamp(c[0]) + " " + clamp(c[1]) + " " + clamp(c[2]);
  }

  function toward(c, target, ink, ratio) {
    var step;
    var out = c;
    for (step = 0; step <= 40; step += 1) {
      out = mix(c, target, step / 40);
      if (contrast(ink, out) >= ratio) {
        return out;
      }
    }
    return out;
  }

  function accentOn(candidates, canvas, fallback) {
    var i;
    for (i = 0; i < candidates.length; i += 1) {
      if (contrast(candidates[i], canvas) >= 4.5) {
        return candidates[i];
      }
    }
    var dark = lum(canvas) < 0.3;
    return toward(fallback, dark ? [255, 255, 255] : [0, 0, 0], canvas, 4.5);
  }

  function pickSend(accent, dark, ink, paper) {
    if (dark) {
      return { fill: toward(accent, [255, 255, 255], ink, 4.5), text: ink };
    }
    return { fill: toward(accent, [0, 0, 0], paper, 4.5), text: paper };
  }

  // Slack keeps the window gradient behind the sidebar and paints the content
  // pane in a flat, hue-matched neutral. Every surface here stays in the nav
  // color's family; the Selected items color is used only as an accent.
  function fromShare(name, share) {
    var parts = share.split(",");
    var nav = parseHex(parts[0]);
    var selected = parseHex(parts[1]);
    var presence = parseHex(parts[2]);
    var notify = parseHex(parts[3]);
    var dark = lum(nav) < 0.3;
    var ink = [24, 22, 28];
    var paper = [248, 247, 250];
    var text = dark ? paper : ink;
    var side;
    var top;
    var deepStop;
    var canvas;
    if (dark) {
      side = toward(nav, [0, 0, 0], text, 4.5);
      top = toward(lighten(side, 0.1), [0, 0, 0], text, 4.5);
      deepStop = darken(side, 0.22);
      canvas = mix(darken(side, 0.5), [22, 22, 26], 0.55);
    } else {
      side = toward(nav, [255, 255, 255], text, 4.5);
      top = side;
      deepStop = lighten(side, 0.45);
      canvas = mix(lighten(side, 0.82), [252, 252, 253], 0.5);
    }
    var chat = dark ? lighten(canvas, 0.05) : darken(canvas, 0.025);
    var composer = dark ? lighten(canvas, 0.1) : [255, 255, 255];
    var userMsg = dark ? lighten(canvas, 0.1) : darken(canvas, 0.045);
    var hover = dark ? lighten(canvas, 0.09) : darken(canvas, 0.06);
    var active = mix(canvas, selected, dark ? 0.32 : 0.2);
    if (contrast(text, active) < 4.5) {
      active = mix(canvas, selected, 0.14);
    }
    var text2 = mix(text, canvas, 0.3);
    var muted = toward(mix(text, canvas, 0.48), text, canvas, 3);
    var border = mix(canvas, text, dark ? 0.16 : 0.13);
    var accent = accentOn([selected, presence, notify], canvas, selected);
    var send = pickSend(accent, dark, ink, paper);
    var danger = notify;
    if (lum(danger) > 0.6 || lum(danger) < 0.06) {
      danger = parseHex("#E01E5A");
    }
    var success = presence;
    if (contrast(success, canvas) < 2.5 || lum(success) > 0.8) {
      success = parseHex("#2EB67D");
    }
    return {
      name: name,
      dark: dark,
      gradient:
        "linear-gradient(160deg, " + hexOf(top) + " 0%, " + hexOf(side) + " 55%, " + hexOf(deepStop) + " 100%)",
      text: rgbOf(text),
      text2: rgbOf(text2),
      muted: rgbOf(muted),
      top: rgbOf(top),
      side: rgbOf(side),
      canvas: rgbOf(canvas),
      chat: rgbOf(chat),
      composer: rgbOf(composer),
      userMsg: rgbOf(userMsg),
      hover: rgbOf(hover),
      active: rgbOf(active),
      border: rgbOf(border),
      link: rgbOf(accent),
      send: rgbOf(send.fill),
      sendText: rgbOf(send.text),
      danger: rgbOf(danger),
      success: rgbOf(success),
      info: rgbOf(accent),
    };
  }

  var themes = [];
  groups.forEach(function (group) {
    group.items.forEach(function (item) {
      var theme = fromShare(item[0], item[1]);
      theme.group = group.name;
      themes.push(theme);
    });
  });

  function replaceTheme(theme) {
    var i;
    for (i = 0; i < themes.length; i += 1) {
      if (themes[i].name === theme.name) {
        theme.group = themes[i].group;
        themes[i] = theme;
        return;
      }
    }
  }

  // Deployment Aubergine: stepped purple window, flat plum content pane.
  replaceTheme({
    name: "Aubergine",
    dark: true,
    gradient: "linear-gradient(160deg, #7C3085 0%, #4A154B 55%, #3F0E40 100%)",
    text: "249 237 255",
    text2: "214 190 222",
    muted: "176 150 186",
    top: "124 48 133",
    side: "74 21 75",
    canvas: "38 20 40",
    chat: "46 24 48",
    composer: "56 30 58",
    userMsg: "56 30 58",
    hover: "54 30 56",
    active: "78 40 80",
    border: "70 48 72",
    link: "234 189 251",
    send: "234 189 251",
    sendText: "63 14 64",
    danger: "205 37 83",
    success: "76 200 148",
    info: "234 189 251",
  });

  // Published Banana 10-color: butter window, warm paper content pane.
  replaceTheme({
    name: "Banana",
    dark: false,
    gradient: "linear-gradient(160deg, #FFC806 0%, #FFEB84 55%, #FFF8D4 100%)",
    text: "89 16 53",
    text2: "122 60 90",
    muted: "140 92 112",
    top: "255 200 6",
    side: "255 235 132",
    canvas: "255 251 238",
    chat: "255 248 226",
    composer: "255 255 255",
    userMsg: "255 244 208",
    hover: "250 238 196",
    active: "255 236 170",
    border: "236 220 176",
    link: "89 16 53",
    send: "89 16 53",
    sendText: "255 248 212",
    danger: "221 65 71",
    success: "22 120 80",
    info: "76 109 194",
  });

  function current() {
    var name = "Aubergine";
    try {
      name = localStorage.getItem(KEY) || "Aubergine";
    } catch (err) {
      name = "Aubergine";
    }
    var i;
    for (i = 0; i < themes.length; i += 1) {
      if (themes[i].name === name) {
        return themes[i];
      }
    }
    return themes.filter(function (theme) {
      return theme.name === "Aubergine";
    })[0];
  }

  function apply(theme) {
    var root = document.documentElement;
    root.dataset.slackTheme = theme.name;
    root.style.setProperty("--slack-window", theme.gradient);
    var map = {
      "--text-primary": theme.text,
      "--text-secondary": theme.text2,
      "--text-tertiary": theme.muted,
      "--text-muted": theme.muted,
      "--header-primary": theme.top,
      "--presentation": theme.canvas,
      "--surface-primary": theme.canvas,
      "--surface-secondary": theme.chat,
      "--surface-tertiary": theme.hover,
      "--surface-chat": theme.chat,
      "--surface-canvas": theme.canvas,
      "--surface-primary-alt": theme.chat,
      "--surface-dialog": theme.chat,
      "--surface-composer": theme.composer,
      "--surface-user-message": theme.userMsg,
      "--surface-hover": theme.hover,
      "--surface-hover-alt": theme.hover,
      "--surface-active": theme.active,
      "--surface-active-alt": theme.active,
      "--surface-nav-selected": theme.active,
      "--surface-nav-hover": theme.hover,
      "--border-light": theme.border,
      "--border-medium": theme.border,
      "--border-heavy": theme.muted,
      "--border-control": theme.border,
      "--border-field-focus": theme.link,
      "--link": theme.link,
      "--accent-primary": theme.link,
      "--ring-primary": theme.link,
      "--surface-inverted": theme.send,
      "--surface-inverted-hover": theme.send,
      "--surface-submit": theme.send,
      "--button-primary": theme.send,
      "--text-inverted": theme.sendText,
      "--field-fill": theme.composer,
      "--field-text": theme.text,
      "--dialog-title": theme.text,
      "--text-success": theme.success,
      "--text-error": theme.danger,
      "--text-info": theme.info,
    };
    Object.keys(map).forEach(function (key) {
      if (root.style.getPropertyValue(key) !== map[key]) {
        root.style.setProperty(key, map[key]);
      }
    });
    if (document.body) {
      document.body.classList.toggle("dark", theme.dark);
    }
    root.classList.toggle("dark", theme.dark);
    applyChrome(theme);
    refreshPickers(theme);
  }

  function chromeHex(rgb) {
    var parts = String(rgb).trim().split(/\s+/);
    if (parts.length !== 3) {
      return "";
    }
    return hexOf(parts.map(Number));
  }

  function applyChrome(theme) {
    var root = document.documentElement;
    var scheme = theme.dark ? "dark" : "light";
    if (root.style.colorScheme !== scheme) {
      root.style.colorScheme = scheme;
    }
    var color = chromeHex(theme.top);
    if (!color) {
      return;
    }
    var themeMeta = document.querySelector('meta[name="theme-color"]');
    if (themeMeta && themeMeta.getAttribute("content") !== color) {
      themeMeta.setAttribute("content", color);
    }
    var appleMeta = document.querySelector('meta[name="apple-mobile-web-app-status-bar-style"]');
    var appleStyle = theme.dark ? "black-translucent" : "default";
    if (appleMeta && appleMeta.getAttribute("content") !== appleStyle) {
      appleMeta.setAttribute("content", appleStyle);
    }
    try {
      localStorage.setItem(
        "slack-theme-chrome",
        JSON.stringify({ dark: theme.dark, color: color }),
      );
    } catch (err) {
      /* storage unavailable */
    }
  }

  function persist(theme) {
    try {
      localStorage.setItem(KEY, theme.name);
      localStorage.setItem("color-theme", theme.dark ? "dark" : "light");
    } catch (err) {
      /* storage unavailable */
    }
  }

  function choose(theme) {
    persist(theme);
    apply(theme);
  }

  function ensureStyle() {
    if (document.getElementById("slack-window-gradient")) {
      return;
    }
    var style = document.createElement("style");
    style.id = "slack-window-gradient";
    style.textContent = [
      "/* Window gradient lives on the frame, like Slack. */",
      "html, body, #root, #loading-container {",
      "  background-color: rgb(var(--surface-canvas)) !important;",
      "  background-image: var(--slack-window) !important;",
      "  background-attachment: fixed !important;",
      "  background-repeat: no-repeat !important;",
      "  background-size: cover !important;",
      "}",
      "/* Sidebar is glass over the gradient; the content pane stays flat. */",
      "#root aside[aria-label] .bg-surface-primary-alt,",
      "#root #chat-history-nav,",
      "#root .bg-surface-primary-alt.absolute.inset-y-0.left-0 {",
      "  background-color: transparent !important;",
      "}",
      "#root aside[aria-label] .bg-surface-primary-alt.border-r {",
      "  border-right-color: rgb(var(--border-light) / 0.6) !important;",
      "}",
      "#root .bg-surface-canvas.grow,",
      "#root .bg-surface-primary-alt.grow {",
      "  background-color: rgb(var(--surface-canvas) / 0.94) !important;",
      "}",
      ".slack-theme-picker { width: 100%; margin-top: 0.5rem; font: 600 13px/1.3 ui-sans-serif, system-ui, sans-serif; color: rgb(var(--text-primary)); }",
      ".slack-theme-current { width: 100%; text-align: left; border-radius: 0.75rem; border: 1px solid rgb(var(--border-medium)); background: rgb(var(--surface-secondary)); color: rgb(var(--text-primary)); padding: 0.5rem 0.75rem; cursor: pointer; }",
      ".slack-theme-panel { margin-top: 0.45rem; max-height: min(24rem, 50vh); overflow: auto; border: 1px solid rgb(var(--border-medium)); border-radius: 0.75rem; background: rgb(var(--surface-chat)); }",
      ".slack-theme-group { padding: 0.55rem 0.7rem 0.15rem; font-size: 11px; letter-spacing: 0.04em; text-transform: uppercase; color: rgb(var(--text-muted)); }",
      ".slack-theme-option { display: flex; width: 100%; align-items: center; gap: 0.5rem; border: 0; background: transparent; color: rgb(var(--text-primary)); text-align: left; padding: 0.38rem 0.7rem; cursor: pointer; font: inherit; }",
      ".slack-theme-option[aria-selected='true'] { background: rgb(var(--surface-hover)); box-shadow: inset 2px 0 0 rgb(var(--border-field-focus)); }",
      ".slack-theme-swatch { width: 0.85rem; height: 0.85rem; border-radius: 999px; flex: none; border: 1px solid rgb(var(--border-medium)); }",
      "html.dark { color-scheme: dark !important; }",
      "html:not(.dark) { color-scheme: light !important; }",
      "html, html * {",
      "  scrollbar-color: rgb(var(--accent-primary)) rgb(var(--surface-secondary)) !important;",
      "}",
      "::-webkit-scrollbar-thumb,",
      ".dark ::-webkit-scrollbar-thumb,",
      ".scrollbar-hover::-webkit-scrollbar-thumb,",
      ".scrollbar-hover:hover::-webkit-scrollbar-thumb {",
      "  background-color: rgb(var(--accent-primary) / 0.85) !important;",
      "}",
      "::-webkit-scrollbar-track { background-color: transparent !important; }",
      "input:not([type='checkbox']):not([type='radio']):not([type='range']):not([type='file']):not([type='button']):not([type='submit']),",
      "textarea, select {",
      "  color-scheme: inherit;",
      "  background-color: rgb(var(--field-fill)) !important;",
      "  color: rgb(var(--field-text)) !important;",
      "  caret-color: rgb(var(--accent-primary));",
      "  border-color: rgb(var(--border-control)) !important;",
      "}",
      "input:-webkit-autofill {",
      "  -webkit-text-fill-color: rgb(var(--field-text));",
      "  box-shadow: 0 0 0 1000px rgb(var(--field-fill)) inset;",
      "}",
    ].join("\n");
    document.head.appendChild(style);
  }

  function refreshPickers(theme) {
    var pickers = document.querySelectorAll(".slack-theme-picker");
    var i;
    for (i = 0; i < pickers.length; i += 1) {
      var button = pickers[i].querySelector(".slack-theme-current");
      if (button && button.textContent !== theme.name) {
        button.textContent = theme.name;
      }
      var options = pickers[i].querySelectorAll(".slack-theme-option");
      var j;
      for (j = 0; j < options.length; j += 1) {
        var selected = options[j].getAttribute("data-theme-name") === theme.name;
        if ((options[j].getAttribute("aria-selected") === "true") !== selected) {
          options[j].setAttribute("aria-selected", selected ? "true" : "false");
        }
      }
    }
  }

  function buildPicker() {
    var theme = current();
    var root = document.createElement("div");
    root.className = "slack-theme-picker";
    var button = document.createElement("button");
    button.type = "button";
    button.className = "slack-theme-current";
    button.textContent = theme.name;
    button.setAttribute("aria-haspopup", "listbox");
    button.setAttribute("aria-expanded", "false");
    var panel = document.createElement("div");
    panel.className = "slack-theme-panel";
    panel.hidden = true;
    panel.setAttribute("role", "listbox");
    panel.setAttribute("aria-label", "Slack themes");
    groups.forEach(function (group) {
      var heading = document.createElement("div");
      heading.className = "slack-theme-group";
      heading.textContent = group.name;
      panel.appendChild(heading);
      group.items.forEach(function (item) {
        var name = item[0];
        var option = document.createElement("button");
        option.type = "button";
        option.className = "slack-theme-option";
        option.setAttribute("role", "option");
        option.setAttribute("data-theme-name", name);
        option.setAttribute("aria-selected", name === theme.name ? "true" : "false");
        var swatch = document.createElement("span");
        swatch.className = "slack-theme-swatch";
        swatch.style.background = item[1].split(",")[0];
        var label = document.createElement("span");
        label.textContent = name;
        option.appendChild(swatch);
        option.appendChild(label);
        option.addEventListener("click", function (event) {
          event.preventDefault();
          event.stopPropagation();
          var next = themes.filter(function (entry) {
            return entry.name === name;
          })[0];
          choose(next);
        });
        panel.appendChild(option);
      });
    });
    button.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      var open = panel.hidden;
      panel.hidden = !open;
      button.setAttribute("aria-expanded", open ? "true" : "false");
    });
    root.appendChild(button);
    root.appendChild(panel);
    return root;
  }

  function hideNativeThemeControl(row) {
    var trigger = row.querySelector("[data-testid='theme-selector']");
    if (!trigger) {
      return;
    }
    var wrap = trigger.parentElement && trigger.parentElement !== row ? trigger.parentElement : trigger;
    if (wrap.style.display !== "none") {
      wrap.style.setProperty("display", "none", "important");
      wrap.setAttribute("aria-hidden", "true");
    }
  }

  function syncSettings() {
    var labels = document.querySelectorAll("#theme-selector-label");
    var i;
    for (i = 0; i < labels.length; i += 1) {
      var row = labels[i].parentElement;
      if (!row || row.querySelector(".slack-theme-picker")) {
        if (row) {
          hideNativeThemeControl(row);
        }
        continue;
      }
      hideNativeThemeControl(row);
      row.style.flexDirection = "column";
      row.style.alignItems = "stretch";
      row.appendChild(buildPicker());
    }
    refreshPickers(current());
  }

  function onPointerDown(event) {
    var pickers = document.querySelectorAll(".slack-theme-picker");
    var i;
    for (i = 0; i < pickers.length; i += 1) {
      if (pickers[i].contains(event.target)) {
        continue;
      }
      var panel = pickers[i].querySelector(".slack-theme-panel");
      var button = pickers[i].querySelector(".slack-theme-current");
      if (panel && !panel.hidden) {
        panel.hidden = true;
        if (button) {
          button.setAttribute("aria-expanded", "false");
        }
      }
    }
  }

  function onKeyDown(event) {
    if (event.key !== "Escape") {
      return;
    }
    var open = document.querySelector(".slack-theme-picker .slack-theme-panel:not([hidden])");
    if (!open) {
      return;
    }
    event.stopPropagation();
    event.preventDefault();
    var pickers = document.querySelectorAll(".slack-theme-picker");
    var i;
    for (i = 0; i < pickers.length; i += 1) {
      var panel = pickers[i].querySelector(".slack-theme-panel");
      var button = pickers[i].querySelector(".slack-theme-current");
      if (panel) {
        panel.hidden = true;
      }
      if (button) {
        button.setAttribute("aria-expanded", "false");
      }
    }
  }

  function mount() {
    ensureStyle();
    var theme = current();
    persist(theme);
    apply(theme);
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("keydown", onKeyDown, true);
    syncSettings();
    new MutationObserver(function () {
      apply(current());
    }).observe(document.documentElement, { attributes: true, attributeFilter: ["class", "style"] });
    var watchRoot = document.body || document.documentElement;
    new MutationObserver(function () {
      syncSettings();
    }).observe(watchRoot, { childList: true, subtree: true });
  }

  if (document.body) {
    mount();
  } else {
    document.addEventListener("DOMContentLoaded", mount);
  }
})();
