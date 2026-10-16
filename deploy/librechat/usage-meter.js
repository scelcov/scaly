(function () {
  var PEAK_KEY = "deepseek-balance-peak";

  function money(currency, amount) {
    var value = Number(amount);
    if (!Number.isFinite(value)) {
      return "—";
    }
    var symbol = currency === "CNY" ? "¥" : currency === "USD" ? "$" : "";
    var formatted = value.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
    return symbol ? symbol + formatted : formatted + " " + currency;
  }

  function peakRatio(total) {
    var peak = Number(localStorage.getItem(PEAK_KEY) || "0");
    if (!Number.isFinite(peak) || total > peak) {
      peak = total;
      localStorage.setItem(PEAK_KEY, String(peak));
    }
    if (peak <= 0) {
      return 0;
    }
    return Math.max(0, Math.min(100, (total / peak) * 100));
  }

  function ensureStyle() {
    if (document.getElementById("deepseek-usage-style")) {
      return;
    }
    var style = document.createElement("style");
    style.id = "deepseek-usage-style";
    style.textContent = [
      ".ds-usage{position:fixed;z-index:30;right:max(14px,env(safe-area-inset-right));bottom:max(14px,env(safe-area-inset-bottom));",
      "width:168px;padding:10px 12px 11px;border-radius:16px;",
      "background:rgb(var(--surface-primary,46 24 48) / 0.92);color:rgb(var(--text-primary,249 237 255));",
      "border:1px solid rgb(var(--border-light,70 48 72) / 0.9);box-shadow:0 10px 28px rgb(0 0 0 / 0.18);",
      "backdrop-filter:blur(16px);font:500 12px/1.2 Lato,ui-sans-serif,system-ui,sans-serif;pointer-events:none}",
      ".ds-usage-row{display:flex;align-items:baseline;justify-content:space-between;gap:8px}",
      ".ds-usage-name{letter-spacing:.04em;text-transform:uppercase;font-size:10px;color:rgb(var(--text-tertiary,176 150 186))}",
      ".ds-usage-value{font-size:15px;font-weight:700;font-variant-numeric:tabular-nums}",
      ".ds-usage-track{margin-top:8px;height:4px;border-radius:999px;overflow:hidden;background:rgb(var(--surface-active,78 40 80) / 0.85)}",
      ".ds-usage-fill{height:100%;width:0;border-radius:inherit;background:rgb(var(--accent-primary,234 189 251));transition:width .35s ease}",
      ".ds-usage.is-empty .ds-usage-fill{background:rgb(var(--text-error,205 37 83))}",
      "@media (max-width:640px){.ds-usage{bottom:calc(92px + env(safe-area-inset-bottom))}}",
    ].join("");
    document.head.appendChild(style);
  }

  function paint(root, fill, label, value, ratio, empty) {
    root.classList.toggle("is-empty", empty);
    root.querySelector(".ds-usage-name").textContent = label;
    root.querySelector(".ds-usage-value").textContent = value;
    fill.style.width = ratio + "%";
  }

  function mount() {
    if (document.querySelector(".ds-usage")) {
      return;
    }
    ensureStyle();
    var root = document.createElement("div");
    root.className = "ds-usage";
    root.setAttribute("role", "status");
    root.setAttribute("aria-live", "polite");
    var row = document.createElement("div");
    row.className = "ds-usage-row";
    var name = document.createElement("span");
    name.className = "ds-usage-name";
    name.textContent = "DeepSeek";
    var value = document.createElement("span");
    value.className = "ds-usage-value";
    value.textContent = "…";
    var track = document.createElement("div");
    track.className = "ds-usage-track";
    var fill = document.createElement("div");
    fill.className = "ds-usage-fill";
    row.appendChild(name);
    row.appendChild(value);
    track.appendChild(fill);
    root.appendChild(row);
    root.appendChild(track);
    document.body.appendChild(root);

    var endpoint = location.protocol + "//" + location.hostname + ":8790/balance";
    fetch(endpoint)
      .then(function (response) {
        if (!response.ok) {
          throw new Error("balance");
        }
        return response.json();
      })
      .then(function (data) {
        var total = Number(data.total);
        var ratio = Number.isFinite(total) ? peakRatio(total) : 0;
        var shown = money(data.currency, data.total);
        var empty = data.available === false || !Number.isFinite(total) || total <= 0;
        paint(root, fill, "DeepSeek", shown, empty ? 0 : ratio, empty);
        root.title =
          "Granted " +
          money(data.currency, data.granted) +
          " · Topped up " +
          money(data.currency, data.toppedUp);
        root.setAttribute("aria-label", "DeepSeek balance " + shown);
      })
      .catch(function () {
        paint(root, fill, "DeepSeek", "—", 0, true);
        root.setAttribute("aria-label", "DeepSeek balance unavailable");
      });
  }

  if (document.body) {
    mount();
  } else {
    document.addEventListener("DOMContentLoaded", mount);
  }
})();
