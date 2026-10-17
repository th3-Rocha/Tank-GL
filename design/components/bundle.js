/* @ds-bundle: {"format":4,"namespace":"Tank","components":[{"name":"Icon"},{"name":"Kbd"},{"name":"Button"},{"name":"SegmentedControl"},{"name":"Knob"},{"name":"Fader"},{"name":"Toggle"},{"name":"Select"},{"name":"LedColorPicker"},{"name":"LedDisplay"},{"name":"ConnectionStatus"},{"name":"Tag"},{"name":"ProgressBar"},{"name":"SearchField"},{"name":"CommandPalette"},{"name":"LibraryTable"},{"name":"PresetGrid"},{"name":"Panel"},{"name":"EffectBlock"},{"name":"SignalChain"},{"name":"PresetEditor"},{"name":"MidiMonitor"}]} */
(function () {
var ICONS={"usb":[["p","M12 3v14"],["p","M9.5 5.5L12 3l2.5 2.5"],["c",12,19,2],["p","M12 13.5L8 11V8.5"],["r",7,6.5,2,2,0.4],["p","M12 11.5l4-2V8"],["c",16,7,1]],"bluetooth":[["p","M7 7.5l10 9-5 4.5V3l5 4.5-10 9"]],"search":[["c",10.5,10.5,6],["p","M20 20l-5.2-5.2"]],"play":[["pf","M8 5.5v13l10.5-6.5z"]],"pause":[["p","M8.5 5v14M15.5 5v14"]],"stop":[["rf",6.5,6.5,11,11,1.5]],"send":[["p","M12 15V4"],["p","M7.5 8.5L12 4l4.5 4.5"],["p","M5 14v5h14v-5"]],"download":[["p","M12 4v11"],["p","M7.5 10.5L12 15l4.5-4.5"],["p","M5 14v5h14v-5"]],"swap":[["p","M7 4L4 7l3 3"],["p","M4 7h13"],["p","M17 14l3 3-3 3"],["p","M20 17H7"]],"save":[["p","M5 4h11l3 3v13H5z"],["p","M8 4v5h7V4"],["p","M8 20v-6h8v6"]],"waveform":[["p","M3 12h3l2-7 3 14 2-10 2 6 1.5-3H21"]],"amp":[["r",3,6,18,13,2],["p","M3 10.5h18"],["c",8,14.75,1.5],["c",12,14.75,1.5],["c",16,14.75,1.5],["p","M9 6V4.5h6V6"]],"grid":[["r",4,4,7,7,1.5],["r",13,4,7,7,1.5],["r",4,13,7,7,1.5],["r",13,13,7,7,1.5]],"knob":[["p","M6.34 17.66A8 8 0 1117.66 17.66"],["p","M12 12l3.2-3.2"]],"link":[["p","M10 14a4 4 0 005.66 0l3-3a4 4 0 00-5.66-5.66l-1 1"],["p","M14 10a4 4 0 00-5.66 0l-3 3a4 4 0 005.66 5.66l1-1"]],"star":[["p","M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.85L12 16.9l-5.25 2.75 1-5.85L3.5 9.7l5.9-.9z"]],"star-filled":[["pf","M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.85L12 16.9l-5.25 2.75 1-5.85L3.5 9.7l5.9-.9z"]],"plus":[["p","M12 5v14M5 12h14"]],"close":[["p","M6 6l12 12M18 6L6 18"]],"check":[["p","M5 12.5l4.5 4.5L19 7"]],"chevron-down":[["p","M6 9l6 6 6-6"]],"chevron-right":[["p","M9 6l6 6-6 6"]],"power":[["p","M12 3v8"],["p","M6.4 6.6a8 8 0 1011.2 0"]],"sliders":[["p","M6 4v16M12 4v16M18 4v16"],["p","M4 15h4M10 8h4M16 13h4"]],"midi":[["c",12,12,8.5],["c",7.8,12,0.9],["c",16.2,12,0.9],["c",9.2,8.6,0.9],["c",14.8,8.6,0.9],["c",12,7.3,0.9],["p","M10.5 17.5h3"]],"folder":[["p","M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"]],"trash":[["p","M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"]],"undo":[["p","M9 13L4 8l5-5"],["p","M4 8h10a6 6 0 010 12h-3"]],"redo":[["p","M15 13l5-5-5-5"],["p","M20 8H10a6 6 0 000 12h3"]],"tag":[["p","M3.5 12.5V4.5a1 1 0 011-1h8l8 8.5-8.5 8.5z"],["c",8,8,1.3]],"filter":[["p","M4 5h16l-6 7.5V19l-4-2v-4.5z"]],"setlist":[["p","M9 18V5.5l11-2V16"],["c",6.5,18,2.5],["c",17.5,16,2.5]],"headphones":[["p","M4 15v-3a8 8 0 0116 0v3"],["r",3,14,4,6,1.5],["r",17,14,4,6,1.5]],"alert":[["p","M12 4l9 16H3z"],["p","M12 10v4.5"],["cf",12,17.3,0.9]],"history":[["p","M4 12a8 8 0 102.3-5.7L4 8.5"],["p","M4 4v4.5h4.5"],["p","M12 8v4.5l3 2"]],"command":[["r",3,5,18,14,2.5],["p","M7 9.5h.01M11 9.5h.01M15 9.5h.01M17 9.5h.01M8 14.5h8"]]};
(function () {
  "use strict";
  var React = window.React;
  var h = React.createElement;
  var useState = React.useState, useRef = React.useRef, useEffect = React.useEffect,
      useMemo = React.useMemo, useCallback = React.useCallback, Fragment = React.Fragment;

  /* ---------- helpers ---------- */
  function cx() {
    var out = [];
    for (var i = 0; i < arguments.length; i++) { if (arguments[i]) out.push(arguments[i]); }
    return out.join(" ");
  }
  function clamp(v, a, b) { return Math.min(b, Math.max(a, v)); }
  function omit(obj, keys) {
    var o = {};
    for (var k in obj) { if (Object.prototype.hasOwnProperty.call(obj, k) && keys.indexOf(k) < 0) o[k] = obj[k]; }
    return o;
  }
  function useControlled(value, initial, onChange) {
    var s = useState(initial);
    var controlled = value !== undefined;
    var current = controlled ? value : s[0];
    var set = useCallback(function (next) {
      if (!controlled) s[1](next);
      if (onChange) onChange(next);
    }, [controlled, onChange]);
    return [current, set];
  }
  var uidCount = 0;
  function useUid(prefix) {
    var r = useRef(null);
    if (r.current === null) { uidCount += 1; r.current = (prefix || "tk") + "-" + uidCount; }
    return r.current;
  }
  function norm(s) {
    return String(s == null ? "" : s).normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
  }
  /* fuzzy subsequence match: returns {score, idx} or null */
  function fuzzy(query, text) {
    var q = norm(query).replace(/\s+/g, " ").trim();
    if (!q) return { score: 0, idx: [] };
    var t = norm(text);
    var direct = t.indexOf(q);
    if (direct >= 0) {
      var ids = [];
      for (var d = 0; d < q.length; d++) ids.push(direct + d);
      var startBonus = (direct === 0 || /[\s\-_/·.]/.test(t.charAt(direct - 1))) ? 40 : 0;
      return { score: 200 + startBonus - direct, idx: ids };
    }
    var idx = [], score = 0, ti = 0, prev = -2;
    for (var qi = 0; qi < q.length; qi++) {
      var ch = q.charAt(qi);
      if (ch === " ") continue;
      var found = t.indexOf(ch, ti);
      if (found < 0) return null;
      var boundary = found === 0 || /[\s\-_/·.]/.test(t.charAt(found - 1));
      score += 1 + (found === prev + 1 ? 6 : 0) + (boundary ? 8 : 0);
      score -= Math.min(found - ti, 6) * 0.5;
      idx.push(found); prev = found; ti = found + 1;
    }
    return { score: score, idx: idx };
  }
  function Highlight(props) {
    var text = String(props.text == null ? "" : props.text);
    var idx = props.idx || [];
    if (!idx.length) return h(Fragment, null, text);
    var set = {}; idx.forEach(function (i) { set[i] = true; });
    var parts = [], buf = "", on = false;
    for (var i = 0; i < text.length; i++) {
      var hit = !!set[i];
      if (hit !== on && buf) { parts.push(on ? h("mark", { key: i, className: "tk-hl" }, buf) : buf); buf = ""; }
      on = hit; buf += text.charAt(i);
    }
    if (buf) parts.push(on ? h("mark", { key: "end", className: "tk-hl" }, buf) : buf);
    return h(Fragment, null, parts);
  }

  /* ---------- Icon ---------- */
  function Icon(props) {
    var spec = ICONS[props.name];
    var size = props.size || 20;
    if (!spec) return null;
    var kids = spec.map(function (e, i) {
      var t = e[0], filled = t.length === 2 && t.charAt(1) === "f", base = t.charAt(0);
      var extra = filled ? { fill: "currentColor", stroke: "none" } : null;
      if (base === "p") return h("path", Object.assign({ key: i, d: e[1] }, extra));
      if (base === "c") return h("circle", Object.assign({ key: i, cx: e[1], cy: e[2], r: e[3] }, extra));
      return h("rect", Object.assign({ key: i, x: e[1], y: e[2], width: e[3], height: e[4], rx: e[5] }, extra));
    });
    return h("svg", {
      className: cx("tk-icon", props.className), width: size, height: size, viewBox: "0 0 24 24",
      fill: "none", stroke: "currentColor", strokeWidth: props.strokeWidth || 1.75,
      strokeLinecap: "round", strokeLinejoin: "round",
      role: props.title ? "img" : undefined, "aria-label": props.title || undefined,
      "aria-hidden": props.title ? undefined : true, focusable: "false"
    }, kids);
  }

  /* ---------- Kbd ---------- */
  function Kbd(props) {
    var keys = String(props.children || "").split(/\s*\+\s*|\s+/).filter(Boolean);
    return h("span", { className: cx("tk-kbd-group", props.className) }, keys.map(function (k, i) {
      return h("kbd", { key: i, className: "tk-kbd" }, k);
    }));
  }

  /* ---------- Button ---------- */
  function Button(props) {
    var variant = props.variant || "secondary";
    var size = props.size || "md";
    var rest = omit(props, ["variant", "size", "icon", "iconRight", "kbd", "block", "className", "children", "loading"]);
    var iconSize = size === "sm" ? 16 : 18;
    return h("button", Object.assign({ type: "button" }, rest, {
      className: cx("tk-btn", "tk-btn--" + variant, "tk-btn--" + size, props.block && "tk-btn--block",
        !props.children && "tk-btn--icon", props.loading && "is-loading", props.className),
      "aria-busy": props.loading || undefined
    }),
      props.icon && h(Icon, { name: props.icon, size: iconSize }),
      props.children != null && h("span", { className: "tk-btn__text" }, props.children),
      props.kbd && h(Kbd, { className: "tk-btn__kbd" }, props.kbd),
      props.iconRight && h(Icon, { name: props.iconRight, size: iconSize })
    );
  }

  /* ---------- SegmentedControl ---------- */
  function normOptions(options) {
    return (options || []).map(function (o) { return typeof o === "object" ? o : { value: o, label: String(o) }; });
  }
  function SegmentedControl(props) {
    var opts = normOptions(props.options);
    var st = useControlled(props.value, props.defaultValue !== undefined ? props.defaultValue : (opts[0] && opts[0].value), props.onChange);
    var refs = useRef([]);
    function move(i, dir) {
      var n = (i + dir + opts.length) % opts.length;
      st[1](opts[n].value);
      if (refs.current[n]) refs.current[n].focus();
    }
    return h("div", { className: cx("tk-seg", props.size === "sm" && "tk-seg--sm", props.className), role: "radiogroup", "aria-label": props.label },
      opts.map(function (o, i) {
        var on = o.value === st[0];
        return h("button", {
          key: String(o.value), type: "button", role: "radio", "aria-checked": on, tabIndex: on ? 0 : -1,
          className: cx("tk-seg__item", on && "is-on"), disabled: o.disabled,
          ref: function (el) { refs.current[i] = el; },
          onClick: function () { st[1](o.value); },
          onKeyDown: function (e) {
            if (e.key === "ArrowRight" || e.key === "ArrowDown") { e.preventDefault(); move(i, 1); }
            if (e.key === "ArrowLeft" || e.key === "ArrowUp") { e.preventDefault(); move(i, -1); }
          }
        }, o.icon && h(Icon, { name: o.icon, size: 16 }), o.label && h("span", null, o.label));
      }));
  }

  /* ---------- Knob ---------- */
  var KNOB_SIZES = { sm: 60, md: 88, lg: 116 };
  function polar(cx0, r, deg, cy0) { var a = deg * Math.PI / 180; return [cx0 + r * Math.cos(a), (cy0 === undefined ? cx0 : cy0) + r * Math.sin(a)]; }
  function arcPath(c, r, a0, a1) {
    if (a1 - a0 < 0.05) a1 = a0 + 0.05;
    var p0 = polar(c, r, a0), p1 = polar(c, r, a1);
    return "M" + p0[0].toFixed(2) + " " + p0[1].toFixed(2) + " A" + r + " " + r + " 0 " + ((a1 - a0) > 180 ? 1 : 0) + " 1 " + p1[0].toFixed(2) + " " + p1[1].toFixed(2);
  }
  function quantize(v, min, max, step) {
    var q = Math.round((v - min) / step) * step + min;
    return parseFloat(clamp(q, min, max).toFixed(4));
  }
  function Knob(props) {
    var min = props.min != null ? props.min : 0;
    var max = props.max != null ? props.max : 100;
    var range = max - min || 1;
    var step = props.step || (Number.isInteger(min) && Number.isInteger(max) ? 1 : range / 100);
    var reset = props.resetValue != null ? props.resetValue : (props.defaultValue != null ? props.defaultValue : min);
    var st = useControlled(props.value, props.defaultValue != null ? props.defaultValue : min, props.onChange);
    var v = clamp(st[0], min, max);
    var setV = st[1];
    var vRef = useRef(v); vRef.current = v;
    var drag = useRef(null);
    var dialRef = useRef(null);
    var ds = useState(false);
    var uid = useUid("kn");
    var sizeKey = props.size || "md";
    var size = KNOB_SIZES[sizeKey] || 88;
    var sw = size < 70 ? 5 : size < 100 ? 6.5 : 8;
    var c = size / 2;
    var r = c - sw / 2 - 7;
    var capR = r - sw / 2 - 5;
    var t = (v - min) / range;
    var a1 = 135 + 270 * t;
    var bead = polar(c, r, a1);
    var display = props.valueLabels ? (props.valueLabels[Math.round(v - min)] || String(v))
      : props.format ? props.format(v) : String(Math.round(v * 100) / 100);
    var disabled = !!props.disabled;
    var discrete = props.valueLabels || props.ticks;

    function commit(n) { n = quantize(n, min, max, step); if (n !== vRef.current) setV(n); }

    useEffect(function () {
      var el = dialRef.current;
      if (!el) return;
      function onWheel(e) {
        if (disabled) return;
        e.preventDefault();
        var dir = e.deltaY < 0 ? 1 : -1;
        commit(vRef.current + dir * step * (e.shiftKey ? 1 : Math.max(1, Math.round(range / step / 100))));
      }
      el.addEventListener("wheel", onWheel, { passive: false });
      return function () { el.removeEventListener("wheel", onWheel); };
    });

    /* outer LED ring: one dot per detent for discrete knobs, else a fixed count */
    var nd = discrete ? (props.valueLabels ? props.valueLabels.length : Math.round(range / step) + 1) : (size < 70 ? 13 : size < 100 ? 17 : 21);
    nd = clamp(nd, 2, 25);
    var dotR = size < 70 ? 1.3 : size < 100 ? 1.6 : 1.9;
    var dots = [];
    for (var i = 0; i < nd; i++) {
      var frac = i / (nd - 1);
      var pd = polar(c, c - dotR - 0.5, 135 + 270 * frac);
      var lit = t > 0.0001 && frac <= t + 1e-6;
      if (discrete) lit = Math.abs(frac - t) < 1e-6 || (frac < t);
      var isCur = discrete && Math.abs(frac - t) < 1e-6;
      dots.push(h("circle", { key: i, cx: pd[0], cy: pd[1], r: isCur ? dotR + 0.6 : dotR, className: cx("tk-knob__dot", lit && "is-lit", isCur && "is-cur") }));
    }
    var knurl = [];
    var kn = size < 70 ? 24 : 36;
    for (var k = 0; k < kn; k++) {
      var ka = 360 * k / kn;
      var q0 = polar(c, capR - (size < 70 ? 2 : 3), ka), q1 = polar(c, capR - 0.5, ka);
      knurl.push(h("line", { key: k, x1: q0[0], y1: q0[1], x2: q1[0], y2: q1[1] }));
    }
    var pIn = polar(c, capR * 0.32, a1), pOut = polar(c, capR * 0.86, a1);

    return h("div", {
      className: cx("tk-knob", "tk-knob--" + sizeKey, disabled && "is-disabled", ds[0] && "is-dragging", props.className),
      style: { width: Math.max(size, 64) }
    },
      h("div", {
        ref: dialRef, className: "tk-knob__dial", role: "slider", tabIndex: disabled ? -1 : 0,
        "aria-label": props.label, "aria-valuemin": min, "aria-valuemax": max, "aria-valuenow": v,
        "aria-valuetext": display + (props.unit ? " " + props.unit : ""), "aria-disabled": disabled || undefined,
        title: props.hint || "Arraste, role ou use as setas · Shift = ajuste fino · duplo clique = " + reset,
        style: { width: size, height: size },
        onPointerDown: function (e) {
          if (disabled || e.button !== 0) return;
          e.currentTarget.setPointerCapture(e.pointerId);
          drag.current = { x: e.clientX, y: e.clientY, v: vRef.current };
          ds[1](true);
        },
        onPointerMove: function (e) {
          var d = drag.current; if (!d) return;
          var delta = (d.y - e.clientY) + (e.clientX - d.x) * 0.6;
          commit(d.v + delta * range / (e.shiftKey ? 900 : 180));
        },
        onPointerUp: function () { drag.current = null; ds[1](false); },
        onPointerCancel: function () { drag.current = null; ds[1](false); },
        onDoubleClick: function () { if (!disabled) commit(reset); },
        onKeyDown: function (e) {
          if (disabled) return;
          var big = Math.max(step, Math.round(range / 10 / step) * step);
          var n = null;
          if (e.key === "ArrowUp" || e.key === "ArrowRight") n = vRef.current + (e.shiftKey ? big : step);
          else if (e.key === "ArrowDown" || e.key === "ArrowLeft") n = vRef.current - (e.shiftKey ? big : step);
          else if (e.key === "PageUp") n = vRef.current + big;
          else if (e.key === "PageDown") n = vRef.current - big;
          else if (e.key === "Home") n = min;
          else if (e.key === "End") n = max;
          else if (e.key === "Delete" || e.key === "Backspace") n = reset;
          if (n !== null) { e.preventDefault(); commit(n); }
        }
      },
        h("svg", { width: size, height: size, viewBox: "0 0 " + size + " " + size, "aria-hidden": true },
          h("defs", null,
            h("radialGradient", { id: uid + "-cap", cx: "38%", cy: "28%", r: "80%" },
              h("stop", { offset: "0", className: "tk-knob__g0" }), h("stop", { offset: "0.5", className: "tk-knob__g1" }), h("stop", { offset: "1", className: "tk-knob__g2" })),
            h("linearGradient", { id: uid + "-rim", x1: "0", y1: "0", x2: "0", y2: "1" },
              h("stop", { offset: "0", className: "tk-knob__r0" }), h("stop", { offset: "1", className: "tk-knob__r1" }))),
          h("g", { className: "tk-knob__dots" }, dots),
          h("path", { className: "tk-knob__track", d: arcPath(c, r, 135, 405), strokeWidth: sw }),
          t > 0.001 && h("path", { className: "tk-knob__value", d: arcPath(c, r, 135, a1), strokeWidth: sw }),
          h("circle", { className: "tk-knob__well", cx: c, cy: c, r: capR + 3.5 }),
          h("circle", { cx: c, cy: c + 1, r: capR + 1.5, fill: "url(#" + uid + "-rim)" }),
          h("circle", { className: "tk-knob__capface", cx: c, cy: c, r: capR, fill: "url(#" + uid + "-cap)" }),
          h("g", { className: "tk-knob__knurl" }, knurl),
          h("ellipse", { className: "tk-knob__spec", cx: c - capR * 0.22, cy: c - capR * 0.42, rx: capR * 0.52, ry: capR * 0.26 }),
          h("line", { className: "tk-knob__pointer", x1: pIn[0], y1: pIn[1], x2: pOut[0], y2: pOut[1], strokeWidth: size < 70 ? 2.5 : 3.2 }),
          h("circle", { className: "tk-knob__bead", cx: bead[0], cy: bead[1], r: sw / 2 + 0.8 })
        ),
        ds[0] && h("span", { className: "tk-knob__bubble", "aria-hidden": true }, display, props.unit && h("span", { className: "tk-knob__unit" }, props.unit))
      ),
      h("div", { className: "tk-knob__readout" },
        props.label && h("span", { className: "tk-knob__label" }, props.label),
        h("span", { className: "tk-knob__num" }, display, props.unit && h("span", { className: "tk-knob__unit" }, props.unit))
      )
    );
  }

  /* ---------- Fader ---------- */
  function Fader(props) {
    var min = props.min != null ? props.min : 0;
    var max = props.max != null ? props.max : 100;
    var range = max - min || 1;
    var step = props.step || (Number.isInteger(min) && Number.isInteger(max) ? 1 : range / 100);
    var st = useControlled(props.value, props.defaultValue != null ? props.defaultValue : min, props.onChange);
    var v = clamp(st[0], min, max);
    var vRef = useRef(v); vRef.current = v;
    var trackRef = useRef(null);
    var dragging = useRef(false);
    var id = useUid("fader");
    var reset = props.resetValue != null ? props.resetValue : (props.defaultValue != null ? props.defaultValue : min);
    function commit(n) { n = quantize(n, min, max, step); if (n !== vRef.current) st[1](n); }
    function fromX(clientX) {
      var rc = trackRef.current.getBoundingClientRect();
      commit(min + clamp((clientX - rc.left) / rc.width, 0, 1) * range);
    }
    var pct = ((v - min) / range) * 100;
    var display = props.format ? props.format(v) : String(Math.round(v * 100) / 100);
    return h("div", { className: cx("tk-fader", props.disabled && "is-disabled", props.className), style: props.width ? { width: props.width } : null },
      (props.label || props.showValue !== false) && h("div", { className: "tk-fader__head" },
        props.label && h("span", { className: "tk-fader__label", id: id }, props.label),
        props.showValue !== false && h("span", { className: "tk-fader__num" }, display, props.unit && h("span", { className: "tk-knob__unit" }, props.unit))),
      h("div", {
        className: "tk-fader__track", ref: trackRef, role: "slider", tabIndex: props.disabled ? -1 : 0,
        "aria-labelledby": props.label ? id : undefined, "aria-label": props.label ? undefined : props.ariaLabel,
        "aria-valuemin": min, "aria-valuemax": max, "aria-valuenow": v, "aria-valuetext": display + (props.unit ? " " + props.unit : ""),
        onPointerDown: function (e) { if (props.disabled) return; e.currentTarget.setPointerCapture(e.pointerId); dragging.current = true; fromX(e.clientX); },
        onPointerMove: function (e) { if (dragging.current) fromX(e.clientX); },
        onPointerUp: function () { dragging.current = false; },
        onDoubleClick: function () { commit(reset); },
        onKeyDown: function (e) {
          var big = Math.max(step, Math.round(range / 10 / step) * step), n = null;
          if (e.key === "ArrowRight" || e.key === "ArrowUp") n = v + (e.shiftKey ? big : step);
          else if (e.key === "ArrowLeft" || e.key === "ArrowDown") n = v - (e.shiftKey ? big : step);
          else if (e.key === "Home") n = min; else if (e.key === "End") n = max;
          if (n !== null) { e.preventDefault(); commit(n); }
        }
      },
        h("span", { className: "tk-fader__rail" }, h("span", { className: "tk-fader__fill", style: { width: pct + "%" } })),
        h("span", { className: "tk-fader__thumb", style: { left: pct + "%" } }))
    );
  }

  /* ---------- Toggle ---------- */
  function Toggle(props) {
    var st = useControlled(props.checked, !!props.defaultChecked, props.onChange);
    var on = !!st[0];
    return h("button", {
      type: "button", role: "switch", "aria-checked": on, disabled: props.disabled,
      className: cx("tk-toggle", on && "is-on", props.size === "sm" && "tk-toggle--sm", props.className),
      onClick: function () { st[1](!on); }
    },
      h("span", { className: "tk-toggle__track", "aria-hidden": true }, h("span", { className: "tk-toggle__thumb" })),
      props.label && h("span", { className: cx("tk-toggle__label", props.hideLabel && "tk-sr") }, props.label),
      props.showState && h("span", { className: "tk-toggle__state", "aria-hidden": true }, on ? (props.onText || "ON") : (props.offText || "OFF"))
    );
  }

  /* ---------- Select ---------- */
  function Select(props) {
    var opts = normOptions(props.options);
    var st = useControlled(props.value, props.defaultValue !== undefined ? props.defaultValue : (opts[0] && opts[0].value), props.onChange);
    var id = useUid("select");
    return h("div", { className: cx("tk-field", props.className) },
      props.label && h("label", { className: cx("tk-field__label", props.hideLabel && "tk-sr"), htmlFor: id }, props.label),
      h("div", { className: cx("tk-select", "tk-select--" + (props.size || "md")), style: props.width ? { width: props.width } : null },
        h("select", {
          id: id, value: String(st[0]), disabled: props.disabled,
          onChange: function (e) {
            var raw = e.target.value;
            var hit = opts.filter(function (o) { return String(o.value) === raw; })[0];
            st[1](hit ? hit.value : raw);
          }
        }, opts.map(function (o) { return h("option", { key: String(o.value), value: String(o.value), disabled: o.disabled }, o.label); })),
        h(Icon, { name: "chevron-down", size: 16, className: "tk-select__chev" }))
    );
  }

  /* ---------- Tag ---------- */
  function Tag(props) {
    var tone = props.tone || "neutral";
    var El = props.onClick ? "button" : "span";
    return h(El, {
      className: cx("tk-tag", "tk-tag--" + tone, props.size === "sm" && "tk-tag--sm", props.selected && "is-selected", props.onClick && "tk-tag--btn", props.className),
      type: props.onClick ? "button" : undefined, onClick: props.onClick,
      "aria-pressed": props.onClick && props.selected !== undefined ? !!props.selected : undefined
    },
      props.icon && h(Icon, { name: props.icon, size: 13 }),
      h("span", null, props.children),
      props.count != null && h("span", { className: "tk-tag__count" }, props.count),
      props.onRemove && h("span", {
        className: "tk-tag__x", role: "button", tabIndex: 0, "aria-label": "Remover " + props.children,
        onClick: function (e) { e.stopPropagation(); props.onRemove(); },
        onKeyDown: function (e) { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); props.onRemove(); } }
      }, h(Icon, { name: "close", size: 12, strokeWidth: 2.25 }))
    );
  }

  /* ---------- ConnectionStatus ---------- */
  var CONN_WORDS = { connected: "Conectado", connecting: "Conectando…", disconnected: "Desconectado", error: "Falha na conexão" };
  function ConnectionStatus(props) {
    var state = props.state || "disconnected";
    var transport = props.transport || "usb";
    var El = props.onClick ? "button" : "div";
    return h(El, {
      type: props.onClick ? "button" : undefined, onClick: props.onClick,
      className: cx("tk-conn", "tk-conn--" + state, props.onClick && "tk-conn--btn", props.className),
      role: props.onClick ? undefined : "status", "aria-live": "polite"
    },
      h(Icon, { name: transport === "ble" ? "bluetooth" : "usb", size: 16 }),
      h("span", { className: "tk-conn__dot", "aria-hidden": true }),
      h("span", { className: "tk-conn__word" }, CONN_WORDS[state] || state),
      props.device && h("span", { className: "tk-conn__dev" }, props.device),
      props.detail && h("span", { className: "tk-conn__detail" }, props.detail)
    );
  }

  /* ---------- LedDisplay (7 segments) ---------- */
  var SEG = {
    "0": "abcdef", "1": "bc", "2": "abged", "3": "abgcd", "4": "fgbc", "5": "afgcd", "6": "afgedc", "7": "abc", "8": "abcdefg", "9": "abcdfg",
    "A": "abcefg", "b": "fgedc", "C": "adef", "c": "ged", "d": "bcdeg", "E": "adefg", "F": "aefg", "G": "acdef", "H": "bcefg", "h": "cefg",
    "I": "bc", "i": "c", "J": "bcde", "L": "def", "n": "ceg", "o": "cdeg", "O": "abcdef", "P": "abefg", "r": "eg", "S": "afgcd",
    "t": "defg", "U": "bcdef", "u": "cde", "V": "bcdef", "v": "cde", "y": "bcdfg", "-": "g", "_": "d", " ": ""
  };
  function segFor(ch) {
    if (SEG[ch] !== undefined) return SEG[ch];
    var up = ch.toUpperCase(), lo = ch.toLowerCase();
    if (SEG[up] !== undefined) return SEG[up];
    if (SEG[lo] !== undefined) return SEG[lo];
    return "";
  }
  function Digit(p) {
    var H = p.h, W = Math.round(H * 0.56), T = Math.max(2, H * 0.13);
    var lit = segFor(p.ch);
    var g = 0.12 * T;
    var R = {
      a: [T * 0.62, 0, W - T * 1.24, T], g: [T * 0.62, (H - T) / 2, W - T * 1.24, T], d: [T * 0.62, H - T, W - T * 1.24, T],
      f: [0, T * 0.62 + g, T, H / 2 - T * 1.12 - g], b: [W - T, T * 0.62 + g, T, H / 2 - T * 1.12 - g],
      e: [0, H / 2 + T * 0.5 + g, T, H / 2 - T * 1.12 - g], c: [W - T, H / 2 + T * 0.5 + g, T, H / 2 - T * 1.12 - g]
    };
    return h("g", { transform: "translate(" + p.x + " 0) skewX(-6) translate(" + (H * 0.05) + " 0)" },
      "abcdefg".split("").map(function (s) {
        var q = R[s];
        return h("rect", { key: s, x: q[0], y: q[1], width: q[2], height: q[3], rx: T / 2, className: lit.indexOf(s) >= 0 ? "tk-led__on" : "tk-led__off" });
      }));
  }
  function LedDisplay(props) {
    var text = String(props.value == null ? "" : props.value);
    var digits = Math.max(props.digits || text.length || 1, text.length);
    while (text.length < digits) text = " " + text;
    var H = props.height || 40, W = Math.round(H * 0.56), gap = Math.round(H * 0.2);
    var total = digits * W + (digits - 1) * gap + H * 0.12;
    var style = props.color ? { "--tk-led": props.color } : null;
    return h("div", { className: cx("tk-led", props.className), style: style, role: "img", "aria-label": props.label || ("Display: " + text.trim()) },
      h("svg", { width: total, height: H, viewBox: "0 0 " + total + " " + H, "aria-hidden": true },
        text.split("").map(function (ch, i) { return h(Digit, { key: i, ch: ch, h: H, x: i * (W + gap) }); })),
      props.caption && h("span", { className: "tk-led__caption" }, props.caption)
    );
  }

  /* ---------- LED colors ---------- */
  var LED_COLORS = [
    { value: "red", label: "Vermelho" }, { value: "orange", label: "Laranja" }, { value: "yellow", label: "Amarelo" }, { value: "green", label: "Verde" },
    { value: "cyan", label: "Ciano" }, { value: "blue", label: "Azul" }, { value: "purple", label: "Roxo" }, { value: "white", label: "Branco" }
  ];
  function ledVar(name) { return name ? "var(--led-" + name + ")" : "transparent"; }
  function LedDot(props) {
    return h("span", { className: cx("tk-leddot", props.size === "sm" && "tk-leddot--sm"), style: { background: ledVar(props.color), boxShadow: props.color ? "0 0 6px " + ledVar(props.color) : "none" }, "aria-hidden": true });
  }
  function LedColorPicker(props) {
    var colors = props.colors || LED_COLORS;
    var st = useControlled(props.value, props.defaultValue || colors[0].value, props.onChange);
    var refs = useRef([]);
    var cur = colors.filter(function (c) { return c.value === st[0]; })[0];
    function pick(i) { st[1](colors[i].value); if (refs.current[i]) refs.current[i].focus(); }
    return h("div", { className: cx("tk-ledpick", props.className) },
      h("div", { className: "tk-ledpick__row", role: "radiogroup", "aria-label": props.label || "Cor do LED do preset" },
        colors.map(function (c, i) {
          var on = c.value === st[0];
          return h("button", {
            key: c.value, type: "button", role: "radio", "aria-checked": on, "aria-label": c.label, title: c.label,
            tabIndex: on ? 0 : -1, ref: function (el) { refs.current[i] = el; },
            className: cx("tk-ledpick__sw", on && "is-on"), style: { "--sw": ledVar(c.value) },
            onClick: function () { pick(i); },
            onKeyDown: function (e) {
              if (e.key === "ArrowRight" || e.key === "ArrowDown") { e.preventDefault(); pick((i + 1) % colors.length); }
              if (e.key === "ArrowLeft" || e.key === "ArrowUp") { e.preventDefault(); pick((i - 1 + colors.length) % colors.length); }
            }
          });
        })),
      props.showName !== false && h("span", { className: "tk-ledpick__name" }, cur ? cur.label : "")
    );
  }

  /* ---------- SearchField ---------- */
  function SearchField(props) {
    var st = useControlled(props.value, props.defaultValue || "", props.onChange);
    var ref = useRef(null);
    var shortcut = props.shortcut === undefined ? "Ctrl K" : props.shortcut;
    useEffect(function () {
      if (!props.hotkey) return;
      function onKey(e) {
        if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) { e.preventDefault(); if (ref.current) { ref.current.focus(); ref.current.select(); } }
      }
      window.addEventListener("keydown", onKey);
      return function () { window.removeEventListener("keydown", onKey); };
    }, [props.hotkey]);
    useEffect(function () { if (props.autoFocus && ref.current) ref.current.focus(); }, []);
    return h("div", { className: cx("tk-search", "tk-search--" + (props.size || "md"), props.className), style: props.width ? { width: props.width } : null },
      h(Icon, { name: "search", size: 16, className: "tk-search__icon" }),
      h("input", {
        ref: ref, type: "text", inputMode: "search", value: st[0], placeholder: props.placeholder || "Buscar presets, IRs, amps…",
        "aria-label": props.label || props.placeholder || "Buscar", spellCheck: false, autoComplete: "off",
        onChange: function (e) { st[1](e.target.value); },
        onKeyDown: function (e) {
          if (e.key === "Escape" && st[0]) { e.preventDefault(); st[1](""); }
          if (props.onKeyDown) props.onKeyDown(e);
        }
      }),
      st[0] ? h("button", { type: "button", className: "tk-search__clear", "aria-label": "Limpar busca", onClick: function () { st[1](""); if (ref.current) ref.current.focus(); } }, h(Icon, { name: "close", size: 14 }))
        : (shortcut && h(Kbd, { className: "tk-search__kbd" }, shortcut))
    );
  }

  /* ---------- CommandPalette ---------- */
  var KIND = {
    preset: { label: "Presets", icon: "grid" }, ir: { label: "IRs", icon: "waveform" }, amp: { label: "Amps", icon: "amp" },
    param: { label: "Parâmetros", icon: "knob" }, action: { label: "Ações", icon: "chevron-right" }, setlist: { label: "Setlists", icon: "setlist" }
  };
  var KIND_ORDER = ["action", "preset", "ir", "amp", "param", "setlist"];
  function CommandPalette(props) {
    var qs = useState(props.defaultQuery || "");
    var q = qs[0];
    var as = useState(0);
    var listRef = useRef(null);
    var inputRef = useRef(null);
    var id = useUid("cmd");
    var items = props.items || [];
    var limit = props.limitPerGroup || 6;
    var groups = useMemo(function () {
      var scored = [];
      items.forEach(function (it) {
        var m = fuzzy(q, it.title);
        var m2 = !m && it.keywords ? fuzzy(q, it.keywords) : null;
        var m3 = !m && !m2 && it.subtitle ? fuzzy(q, it.subtitle) : null;
        var best = m || m2 || m3;
        if (best) scored.push({ it: it, score: m ? best.score : best.score - 30, idx: m ? best.idx : [] });
      });
      var by = {};
      scored.forEach(function (s) { (by[s.it.kind] = by[s.it.kind] || []).push(s); });
      var out = [];
      KIND_ORDER.concat(Object.keys(by).filter(function (k) { return KIND_ORDER.indexOf(k) < 0; })).forEach(function (k) {
        if (!by[k]) return;
        var arr = by[k].sort(function (a, b) { return b.score - a.score; });
        out.push({ kind: k, total: arr.length, rows: arr.slice(0, q ? limit : Math.min(limit, 4)), best: arr[0].score });
      });
      if (q) out.sort(function (a, b) { return b.best - a.best; });
      return out;
    }, [q, items, limit]);
    var flat = [];
    groups.forEach(function (g) { g.rows.forEach(function (r) { flat.push(r); }); });
    var active = clamp(as[0], 0, Math.max(0, flat.length - 1));
    useEffect(function () { as[1](0); }, [q]);
    useEffect(function () {
      var el = listRef.current && listRef.current.querySelector("[data-active='true']");
      if (el && el.scrollIntoView) el.scrollIntoView({ block: "nearest" });
    }, [active]);
    useEffect(function () { if (props.autoFocus !== false && inputRef.current && !props.inline) inputRef.current.focus(); }, []);
    function choose(row) { if (row && props.onSelect) props.onSelect(row.it); }
    function onKey(e) {
      if (e.key === "ArrowDown") { e.preventDefault(); as[1]((active + 1) % Math.max(1, flat.length)); }
      else if (e.key === "ArrowUp") { e.preventDefault(); as[1]((active - 1 + flat.length) % Math.max(1, flat.length)); }
      else if (e.key === "Enter") { e.preventDefault(); choose(flat[active]); }
      else if (e.key === "Escape") { e.preventDefault(); if (q) qs[1](""); else if (props.onClose) props.onClose(); }
    }
    var n = -1;
    var panel = h("div", { className: cx("tk-cmd", props.inline && "tk-cmd--inline", props.className), role: "dialog", "aria-label": props.label || "Paleta de comandos" },
      h("div", { className: "tk-cmd__bar" },
        h(Icon, { name: "search", size: 18, className: "tk-cmd__icon" }),
        h("input", {
          ref: inputRef, className: "tk-cmd__input", value: q, placeholder: props.placeholder || "Buscar presets, IRs, amps, parâmetros e ações…",
          role: "combobox", "aria-expanded": true, "aria-controls": id + "-list", "aria-activedescendant": flat.length ? id + "-o" + active : undefined,
          "aria-autocomplete": "list", spellCheck: false, autoComplete: "off",
          onChange: function (e) { qs[1](e.target.value); }, onKeyDown: onKey
        }),
        h(Kbd, null, "Esc")),
      h("div", { className: "tk-cmd__list", id: id + "-list", role: "listbox", ref: listRef, style: props.maxHeight ? { maxHeight: props.maxHeight } : null },
        flat.length === 0 && h("div", { className: "tk-cmd__empty" },
          h("span", { className: "tk-cmd__empty-title" }, "Nada encontrado para “" + q + "”"),
          h("span", null, "Tente o nome do arquivo, uma tag (V30, 4x12) ou o slot (2D).")),
        groups.map(function (g) {
          var meta = KIND[g.kind] || { label: g.kind, icon: "chevron-right" };
          return h("div", { key: g.kind, className: "tk-cmd__group", role: "group", "aria-label": meta.label },
            h("div", { className: "tk-cmd__ghead", "aria-hidden": true }, h("span", null, meta.label), g.total > g.rows.length && h("span", { className: "tk-cmd__more" }, g.rows.length + " de " + g.total)),
            g.rows.map(function (row) {
              n += 1;
              var i = n, it = row.it, on = i === active;
              return h("div", {
                key: it.id, id: id + "-o" + i, role: "option", "aria-selected": on, "data-active": on ? "true" : "false",
                className: cx("tk-cmd__opt", on && "is-active"),
                onMouseMove: function () { if (active !== i) as[1](i); },
                onMouseDown: function (e) { e.preventDefault(); },
                onClick: function () { choose(row); }
              },
                it.color ? h(LedDot, { color: it.color }) : h(Icon, { name: it.icon || meta.icon, size: 16, className: "tk-cmd__kind" }),
                h("span", { className: "tk-cmd__text" },
                  h("span", { className: "tk-cmd__title" }, h(Highlight, { text: it.title, idx: row.idx })),
                  it.subtitle && h("span", { className: "tk-cmd__sub" }, it.subtitle)),
                it.meta && h("span", { className: "tk-cmd__meta" }, it.meta),
                on && h("span", { className: "tk-cmd__enter", "aria-hidden": true }, "↵")
              );
            }));
        })),
      h("div", { className: "tk-cmd__foot", "aria-hidden": true },
        h("span", null, h(Kbd, null, "↑ ↓"), " navegar"), h("span", null, h(Kbd, null, "Enter"), " abrir"), h("span", null, h(Kbd, null, "Esc"), q ? " limpar" : " fechar"),
        h("span", { className: "tk-cmd__count" }, flat.length + " resultados")));
    if (props.inline) return panel;
    if (props.open === false) return null;
    return h("div", { className: "tk-overlay", onMouseDown: function (e) { if (e.target === e.currentTarget && props.onClose) props.onClose(); } }, panel);
  }

  /* ---------- LibraryTable ---------- */
  function LibraryTable(props) {
    var items = props.items || [];
    var q = props.query || "";
    var tags = props.tags || [];
    var ss = useState(props.defaultSort || { key: "name", dir: 1 });
    var sort = ss[0];
    var sel = useControlled(props.selected, [], props.onSelectionChange);
    var selected = sel[0];
    var anchor = useRef(null);
    var rowsRef = useRef([]);
    var rows = useMemo(function () {
      var out = [];
      items.forEach(function (it) {
        if (tags.length && !tags.every(function (t) { return (it.tags || []).indexOf(t) >= 0; })) return;
        var m = q ? fuzzy(q, it.name) : { score: 0, idx: [] };
        var m2 = !m && q ? fuzzy(q, (it.tags || []).join(" ") + " " + (it.file || "")) : null;
        if (q && !m && !m2) return;
        out.push({ it: it, idx: m ? m.idx : [], score: m ? m.score : (m2 ? m2.score - 30 : 0) });
      });
      var k = sort.key, d = sort.dir;
      out.sort(function (a, b) {
        if (q && k === "relevance") return b.score - a.score;
        var A = a.it, B = b.it, x, y;
        if (k === "slot") { x = A.slot == null ? 99 : A.slot; y = B.slot == null ? 99 : B.slot; }
        else if (k === "fav") { x = A.favorite ? 0 : 1; y = B.favorite ? 0 : 1; }
        else { x = norm(A.name); y = norm(B.name); }
        return (x < y ? -1 : x > y ? 1 : 0) * d;
      });
      return out;
    }, [items, q, tags.join("|"), sort.key, sort.dir]);
    function isSel(id) { return selected.indexOf(id) >= 0; }
    function clickRow(e, i, id) {
      if (e.shiftKey && anchor.current != null) {
        var a = Math.min(anchor.current, i), b = Math.max(anchor.current, i);
        sel[1](rows.slice(a, b + 1).map(function (r) { return r.it.id; }));
      } else if (e.ctrlKey || e.metaKey) {
        sel[1](isSel(id) ? selected.filter(function (x) { return x !== id; }) : selected.concat([id]));
        anchor.current = i;
      } else { sel[1]([id]); anchor.current = i; }
    }
    function head(key, label, cls) {
      var on = sort.key === key;
      return h("th", { className: cls, scope: "col", "aria-sort": on ? (sort.dir > 0 ? "ascending" : "descending") : "none" },
        h("button", { type: "button", className: cx("tk-th", on && "is-on"), onClick: function () { ss[1]({ key: key, dir: on ? -sort.dir : 1 }); } },
          label, h("span", { className: "tk-th__arrow", "aria-hidden": true }, on ? (sort.dir > 0 ? "▲" : "▼") : "")));
    }
    return h("div", { className: cx("tk-lib", props.className) },
      h("div", { className: "tk-lib__scroll", style: props.maxHeight ? { maxHeight: props.maxHeight } : null },
        h("table", { className: "tk-lib__table", role: "grid", "aria-multiselectable": true, "aria-label": props.label || "Biblioteca" },
          h("thead", null, h("tr", null,
            head("fav", h(Icon, { name: "star", size: 14, title: "Favoritos" }), "tk-col-fav"),
            head("name", "NOME", "tk-col-name"),
            h("th", { className: "tk-col-tags", scope: "col" }, h("span", { className: "tk-th tk-th--static" }, "TAGS")),
            h("th", { className: "tk-col-fmt", scope: "col" }, h("span", { className: "tk-th tk-th--static" }, "FORMATO")),
            head("slot", "SLOT", "tk-col-slot"),
            h("th", { className: "tk-col-act", scope: "col" }, h("span", { className: "tk-sr" }, "Ações"))
          )),
          h("tbody", null,
            rows.length === 0 && h("tr", null, h("td", { colSpan: 6, className: "tk-lib__empty" }, q ? "Nenhuma IR bate com “" + q + "”." : "Biblioteca vazia. Arraste uma pasta de .wav para cá.")),
            rows.map(function (r, i) {
              var it = r.it, on = isSel(it.id), playing = props.playingId === it.id;
              return h("tr", {
                key: it.id, className: cx("tk-row", on && "is-selected", playing && "is-playing"), "aria-selected": on, tabIndex: i === 0 ? 0 : -1,
                ref: function (el) { rowsRef.current[i] = el; },
                onClick: function (e) { clickRow(e, i, it.id); },
                onDoubleClick: function () { if (props.onPreview) props.onPreview(it.id); },
                onKeyDown: function (e) {
                  var t = null;
                  if (e.key === "ArrowDown") t = Math.min(rows.length - 1, i + 1);
                  if (e.key === "ArrowUp") t = Math.max(0, i - 1);
                  if (t !== null) { e.preventDefault(); if (rowsRef.current[t]) rowsRef.current[t].focus(); if (e.shiftKey) clickRow({ shiftKey: true }, t, rows[t].it.id); }
                  if (e.key === " ") { e.preventDefault(); clickRow({ ctrlKey: true }, i, it.id); }
                  if (e.key === "Enter" && props.onPreview) { e.preventDefault(); props.onPreview(it.id); }
                }
              },
                h("td", { className: "tk-col-fav" }, h("button", {
                  type: "button", className: cx("tk-star", it.favorite && "is-on"), "aria-pressed": !!it.favorite, "aria-label": "Favorito",
                  onClick: function (e) { e.stopPropagation(); if (props.onToggleFavorite) props.onToggleFavorite(it.id); }
                }, h(Icon, { name: it.favorite ? "star-filled" : "star", size: 15 }))),
                h("td", { className: "tk-col-name" },
                  h("span", { className: "tk-lib__name" }, h(Highlight, { text: it.name, idx: r.idx })),
                  it.file && h("span", { className: "tk-lib__file" }, it.file)),
                h("td", { className: "tk-col-tags" }, h("span", { className: "tk-lib__tags" }, (it.tags || []).slice(0, 3).map(function (t) {
                  return h(Tag, { key: t, size: "sm", tone: tags.indexOf(t) >= 0 ? "amber" : "neutral" }, t);
                }))),
                h("td", { className: "tk-col-fmt" }, it.format || ""),
                h("td", { className: "tk-col-slot" }, it.slot != null
                  ? h("span", { className: "tk-slot is-on" }, h("span", { className: "tk-slot__n" }, it.slot), " no pedal")
                  : h("span", { className: "tk-slot" }, "só no PC")),
                h("td", { className: "tk-col-act" }, h("span", { className: "tk-lib__acts" },
                  h(Button, {
                    variant: "ghost", size: "sm", icon: playing ? "stop" : "headphones", "aria-label": playing ? "Parar pré-escuta" : "Pré-escutar " + it.name,
                    title: "Pré-escutar (Enter)", onClick: function (e) { e.stopPropagation(); if (props.onPreview) props.onPreview(it.id); }
                  }),
                  h(Button, {
                    variant: "ghost", size: "sm", icon: "send", "aria-label": "Gravar " + it.name + " no pedal", title: "Gravar no pedal",
                    onClick: function (e) { e.stopPropagation(); if (props.onSend) props.onSend(it.id); }
                  })))
              );
            })))),
      props.footer !== false && h("div", { className: "tk-lib__foot" },
        h("span", null, rows.length + " de " + items.length + " " + (props.noun || "IRs")),
        selected.length > 0 && h("span", { className: "tk-lib__selcount" }, selected.length + " selecionada" + (selected.length > 1 ? "s" : "")),
        props.footerActions)
    );
  }

  /* ---------- PresetGrid ---------- */
  var BANK_LETTERS = ["A", "B", "C", "D"];
  function PresetGrid(props) {
    var banks = props.banks || 9;
    var letters = props.letters || BANK_LETTERS;
    var map = {};
    (props.presets || []).forEach(function (p) { map[p.id] = p; });
    var st = useControlled(props.value, props.defaultValue || (banks > 0 ? "1A" : null), props.onSelect);
    var q = props.query || "";
    var refs = useRef({});
    function idAt(b, l) { return (b + 1) + letters[l]; }
    function focusAt(b, l) { var id = idAt(b, l); st[1](id); if (refs.current[id]) refs.current[id].focus(); }
    var rows = [];
    for (var b = 0; b < banks; b++) {
      (function (b) {
        rows.push(h("div", { key: b, className: "tk-pgrid__row", role: "row" },
          h("span", { className: "tk-pgrid__bank", role: "rowheader" }, b + 1),
          letters.map(function (L, l) {
            var id = idAt(b, l), p = map[id] || { id: id, name: "Vazio" };
            var on = st[0] === id, live = props.activeId === id;
            var match = !q || !!fuzzy(q, p.name + " " + id + " " + (p.ir || ""));
            return h("button", {
              key: id, type: "button", role: "gridcell", "aria-selected": on, tabIndex: on ? 0 : -1,
              ref: function (el) { refs.current[id] = el; },
              className: cx("tk-pcell", on && "is-on", live && "is-live", !match && "is-dim", p.modified && "is-mod"),
              title: p.name + (p.ir ? " · " + p.ir : ""),
              onClick: function () { st[1](id); },
              onDoubleClick: function () { if (props.onActivate) props.onActivate(id); },
              onKeyDown: function (e) {
                var nb = b, nl = l;
                if (e.key === "ArrowRight") nl = Math.min(letters.length - 1, l + 1);
                else if (e.key === "ArrowLeft") nl = Math.max(0, l - 1);
                else if (e.key === "ArrowDown") nb = Math.min(banks - 1, b + 1);
                else if (e.key === "ArrowUp") nb = Math.max(0, b - 1);
                else if (e.key === "Enter") { e.preventDefault(); if (props.onActivate) props.onActivate(id); return; }
                else return;
                e.preventDefault(); focusAt(nb, nl);
              }
            },
              h("span", { className: "tk-pcell__top" },
                h("span", { className: "tk-pcell__id" }, id),
                p.color && h(LedDot, { color: p.color, size: "sm" }),
                live && h("span", { className: "tk-pcell__live", title: "Ativo no pedal" }, h(Icon, { name: "play", size: 10 }), h("span", { className: "tk-sr" }, "ativo no pedal")),
                p.modified && h("span", { className: "tk-pcell__mod", title: "Modificado, não salvo" }, h("span", { className: "tk-sr" }, "modificado"))),
              h("span", { className: "tk-pcell__name" }, p.name));
          })));
      })(b);
    }
    return h("div", { className: cx("tk-pgrid", props.className), role: "grid", "aria-label": props.label || "Presets do pedal" },
      h("div", { className: "tk-pgrid__row tk-pgrid__head", role: "row", "aria-hidden": true },
        h("span", { className: "tk-pgrid__bank" }), letters.map(function (L) { return h("span", { key: L, className: "tk-pgrid__col" }, L); })),
      rows);
  }

  /* ---------- ProgressBar ---------- */
  function ProgressBar(props) {
    var indet = props.value == null;
    var pct = indet ? 0 : clamp(props.value, 0, 1) * 100;
    var tone = props.tone || "amber";
    return h("div", { className: cx("tk-progress", "tk-progress--" + tone, indet && "is-indet", props.className) },
      (props.label || !indet) && h("div", { className: "tk-progress__head" },
        props.label && h("span", { className: "tk-progress__label" }, props.label),
        !indet && h("span", { className: "tk-progress__pct" }, Math.round(pct) + "%")),
      h("div", {
        className: "tk-progress__track", role: "progressbar", "aria-label": props.label || "Progresso",
        "aria-valuemin": 0, "aria-valuemax": 100, "aria-valuenow": indet ? undefined : Math.round(pct), "aria-valuetext": props.detail
      }, h("span", { className: "tk-progress__fill", style: indet ? null : { width: pct + "%" } })),
      props.detail && h("div", { className: "tk-progress__detail" }, props.detail)
    );
  }

  /* ---------- MidiMonitor ---------- */
  function MidiMonitor(props) {
    var msgs = props.messages || [];
    var ref = useRef(null);
    useEffect(function () { if (!props.paused && ref.current) ref.current.scrollTop = ref.current.scrollHeight; }, [msgs.length, props.paused]);
    return h("div", { className: cx("tk-midi", props.className) },
      h("div", { className: "tk-midi__bar" },
        h(Icon, { name: "midi", size: 16 }),
        h("span", { className: "tk-midi__title" }, props.title || "Monitor MIDI"),
        h("span", { className: "tk-midi__count" }, msgs.length + " mensagens"),
        h("span", { className: "tk-midi__spacer" }),
        props.onTogglePause && h(Button, { size: "sm", variant: "ghost", icon: props.paused ? "play" : "pause", onClick: props.onTogglePause }, props.paused ? "Retomar" : "Pausar"),
        props.onClear && h(Button, { size: "sm", variant: "ghost", icon: "trash", onClick: props.onClear }, "Limpar")),
      h("div", { className: "tk-midi__log", ref: ref, role: "log", "aria-live": props.paused ? "off" : "polite", style: props.height ? { height: props.height } : null },
        msgs.length === 0 && h("div", { className: "tk-midi__empty" }, "Mexa um knob no pedal ou no app para ver as mensagens."),
        msgs.map(function (m, i) {
          return h("div", { key: i, className: cx("tk-midi__line", m.dir === "in" ? "is-in" : "is-out", m.highlight && "is-hl") },
            h("span", { className: "tk-midi__t" }, m.time),
            h("span", { className: "tk-midi__dir" }, m.dir === "in" ? "← IN" : "→ OUT"),
            h("span", { className: "tk-midi__bytes" }, m.bytes),
            m.text && h("span", { className: "tk-midi__text" }, m.text));
        })));
  }

  /* ---------- Panel ---------- */
  function Panel(props) {
    return h("section", { className: cx("tk-panel", props.flush && "tk-panel--flush", props.sunken && "tk-panel--sunken", props.className), "aria-label": typeof props.title === "string" ? props.title : undefined, style: props.style },
      (props.title || props.actions) && h("header", { className: "tk-panel__head" },
        props.icon && h(Icon, { name: props.icon, size: 18, className: "tk-panel__icon" }),
        props.title && h("h3", { className: "tk-panel__title" }, props.title),
        props.meta && h("span", { className: "tk-panel__meta" }, props.meta),
        h("span", { className: "tk-panel__spacer" }),
        props.actions && h("div", { className: "tk-panel__actions" }, props.actions)),
      h("div", { className: "tk-panel__body" }, props.children));
  }

  /* ---------- EffectBlock ---------- */
  function EffectBlock(props) {
    var toggleable = props.enabled !== undefined || props.defaultEnabled !== undefined || props.onToggle;
    var st = useControlled(props.enabled, props.defaultEnabled !== undefined ? props.defaultEnabled : true, props.onToggle);
    var on = toggleable ? !!st[0] : true;
    return h("section", {
      className: cx("tk-block", !on && "is-bypassed", props.className), "data-bypassed": on ? undefined : "true",
      "aria-label": props.title + (on ? "" : " (desligado)"), style: props.style
    },
      h("header", { className: "tk-block__head" },
        h("h4", { className: "tk-block__title" }, props.title),
        props.subtitle && h("span", { className: "tk-block__sub" }, props.subtitle),
        toggleable && h(Toggle, { size: "sm", checked: on, onChange: st[1], label: (on ? "Desligar " : "Ligar ") + props.title, hideLabel: true })),
      h("div", { className: "tk-block__body" }, props.children));
  }

  /* ---------- SignalChain ---------- */
  var CHAIN_DEFAULT = [
    { id: "gate", kind: "gate", label: "GATE", on: true, led: "green", sub: "0" },
    { id: "amp", kind: "amp", label: "AMP", on: true, sub: "Modern" },
    { id: "mod", kind: "stomp", label: "MOD", on: true, led: "purple", sub: "FX 8" },
    { id: "delay", kind: "stomp", label: "DELAY", on: false, led: "cyan", sub: "1 ms" },
    { id: "reverb", kind: "stomp", label: "REVERB", on: true, led: "blue", sub: "Decay 12" },
    { id: "cab", kind: "cab", label: "CAB · IR", on: true, sub: "Slot 3" }
  ];
  var CHAIN_W = { gate: 66, amp: 176, stomp: 86, cab: 176 };
  var CHAIN_H = { gate: 92, amp: 118, stomp: 124, cab: 176 };
  var CHAIN_GAP = { gate: 46, amp: 52, stomp: 34, cab: 52 };
  var FLOOR = 214, CABLE_Y = 154;
  function SignalChain(props) {
    var nodes = props.nodes || CHAIN_DEFAULT;
    var uid = useUid("sc");
    var sel = useControlled(props.selected, props.defaultSelected || "amp", props.onSelect);
    var live = props.signal !== false;
    /* layout: IN jack, nodes, OUT jack, centred in a 1000-wide stage */
    var total = 40;
    nodes.forEach(function (n, i) { total += CHAIN_W[n.kind] + (i ? CHAIN_GAP[n.kind] : 52); });
    total += 52 + 40;
    var scale = Math.min(1.3, 1000 / total);
    var x = (1000 - total * scale) / 2 / scale;
    var VW = 1000 / scale;
    var inX = x + 20; x += 40;
    var placed = nodes.map(function (n, i) {
      x += i ? CHAIN_GAP[n.kind] : 52;
      var p = { n: n, x: x, w: CHAIN_W[n.kind], h: CHAIN_H[n.kind] };
      p.y = FLOOR - p.h; p.cx = x + p.w / 2;
      x += p.w;
      return p;
    });
    var outX = x + 52 + 20;
    function cable(x1, x2, key, hot) {
      var sag = Math.min(26, (x2 - x1) * 0.45);
      var d = "M" + x1 + " " + CABLE_Y + " C" + (x1 + (x2 - x1) * 0.35) + " " + (CABLE_Y + sag) + " " + (x2 - (x2 - x1) * 0.35) + " " + (CABLE_Y + sag) + " " + x2 + " " + CABLE_Y;
      return h("g", { key: key, className: cx("tk-chain__cable", hot && live && "is-hot") },
        h("path", { d: d, className: "tk-chain__jacket" }),
        h("path", { d: d, className: "tk-chain__core" }),
        live && h("path", { d: d, className: "tk-chain__flow" }));
    }
    var cables = [];
    var prevX = inX + 9;
    placed.forEach(function (p, i) {
      cables.push(cable(prevX, p.x, "c" + i, true));
      prevX = p.x + p.w;
    });
    cables.push(cable(prevX, outX - 9, "cout", true));

    function plug(px, side) {
      return h("g", { className: "tk-chain__plug" },
        h("rect", { x: side < 0 ? px - 10 : px, y: CABLE_Y - 4, width: 10, height: 8, rx: 2 }));
    }
    function footswitch(p, cxp, cyp) {
      var n = p.n;
      return h("g", {
        className: "tk-chain__fsw", role: "switch", "aria-checked": !!n.on, "aria-label": (n.on ? "Desligar " : "Ligar ") + n.label, tabIndex: 0,
        onClick: function (e) { e.stopPropagation(); if (props.onToggle) props.onToggle(n.id, !n.on); },
        onKeyDown: function (e) { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); e.stopPropagation(); if (props.onToggle) props.onToggle(n.id, !n.on); } }
      },
        h("circle", { cx: cxp, cy: cyp, r: 12, className: "tk-chain__fsw-ring" }),
        h("circle", { cx: cxp, cy: cyp, r: 9, fill: "url(#" + uid + "-metal)" }),
        h("circle", { cx: cxp - 2.5, cy: cyp - 3, r: 3, className: "tk-chain__spec" }));
    }
    function led(cxp, cyp, color, on) {
      return h("circle", { cx: cxp, cy: cyp, r: 4.5, className: cx("tk-chain__led", on && "is-on"), style: { "--c": "var(--led-" + (color || "red") + ")" } });
    }
    function miniKnob(cxp, cyp, rr, ang) {
      var p = polar(cxp, rr * 0.8, ang, cyp);
      return h("g", { className: "tk-chain__mk" },
        h("circle", { cx: cxp, cy: cyp, r: rr }),
        h("line", { x1: cxp, y1: cyp, x2: p[0], y2: p[1] }));
    }
    function drawNode(p) {
      var n = p.n, X = p.x, Y = p.y, W = p.w, H = p.h;
      if (n.kind === "amp") {
        var knobs = [];
        for (var i = 0; i < 6; i++) knobs.push(h(React.Fragment, { key: i }, miniKnob(X + 22 + i * 21, Y + 27, 6.2, 150 + i * 40)));
        return h("g", null,
          h("rect", { x: X + W / 2 - 28, y: Y - 11, width: 56, height: 14, rx: 7, className: "tk-chain__handle" }),
          h("rect", { x: X, y: Y, width: W, height: H, rx: 10, className: "tk-chain__tolex" }),
          h("rect", { x: X + 8, y: Y + 10, width: W - 16, height: 34, rx: 4, fill: "url(#" + uid + "-panel)" }),
          knobs,
          h("circle", { cx: X + W - 22, cy: Y + 27, r: 6, className: cx("tk-chain__jewel", n.on && "is-on") }),
          h("rect", { x: X + 8, y: Y + 50, width: W - 16, height: H - 58, rx: 4, fill: "url(#" + uid + "-grille)", className: "tk-chain__grillebox" }),
          h("rect", { x: X + W / 2 - 40, y: Y + 66, width: 80, height: 22, rx: 3, className: "tk-chain__plate" }),
          h("text", { x: X + W / 2, y: Y + 81.5, textAnchor: "middle", className: cx("tk-chain__plate-t", n.on && "is-on") }, (n.sub || "AMP").toUpperCase().slice(0, 12)));
      }
      if (n.kind === "cab") {
        var cones = [[X + W * 0.29, Y + 70], [X + W * 0.71, Y + 70]];
        return h("g", null,
          h("rect", { x: X, y: Y, width: W, height: H, rx: 10, className: "tk-chain__tolex" }),
          h("rect", { x: X + 8, y: Y + 8, width: W - 16, height: H - 44, rx: 5, fill: "url(#" + uid + "-grille)", className: "tk-chain__grillebox" }),
          cones.map(function (cc, i) {
            return h("g", { key: i, className: cx("tk-chain__cone", n.on && live && "is-vibe"), style: { transformOrigin: cc[0] + "px " + cc[1] + "px", animationDelay: (i * 0.11) + "s" } },
              h("circle", { cx: cc[0], cy: cc[1], r: 33, className: "tk-chain__surround" }),
              h("circle", { cx: cc[0], cy: cc[1], r: 29, fill: "url(#" + uid + "-cone)" }),
              h("circle", { cx: cc[0], cy: cc[1], r: 20, className: "tk-chain__ring" }),
              h("circle", { cx: cc[0], cy: cc[1], r: 12, className: "tk-chain__ring" }),
              h("circle", { cx: cc[0], cy: cc[1], r: 9, fill: "url(#" + uid + "-dust)" }),
              n.on && live && h("circle", { cx: cc[0], cy: cc[1], r: 31, className: "tk-chain__halo" }));
          }),
          h("rect", { x: X + 14, y: Y + H - 30, width: W - 28, height: 20, rx: 3, fill: "url(#" + uid + "-panel)" }),
          h("text", { x: X + W / 2, y: Y + H - 16, textAnchor: "middle", className: "tk-chain__plate-t is-dark" }, ("IR · " + (n.sub || "")).toUpperCase().slice(0, 18)));
      }
      if (n.kind === "gate") {
        return h("g", null,
          h("rect", { x: X, y: Y, width: W, height: H, rx: 8, fill: "url(#" + uid + "-body)", className: "tk-chain__box" }),
          h("rect", { x: X, y: Y, width: W, height: 6, rx: 3, className: "tk-chain__stripe", style: { "--c": "var(--led-" + (n.led || "green") + ")" } }),
          led(X + W / 2, Y + 20, n.led, n.on),
          miniKnob(X + W / 2, Y + 46, 10, 135 + 270 * clamp((n.amount || 0) / 100, 0, 1)),
          h("text", { x: X + W / 2, y: Y + 80, textAnchor: "middle", className: "tk-chain__name" }, "GATE"));
      }
      return h("g", null,
        h("rect", { x: X, y: Y, width: W, height: H, rx: 10, fill: "url(#" + uid + "-body)", className: "tk-chain__box" }),
        h("rect", { x: X, y: Y, width: W, height: 7, rx: 3.5, className: "tk-chain__stripe", style: { "--c": "var(--led-" + (n.led || "red") + ")" } }),
        miniKnob(X + W * 0.3, Y + 28, 9, 200), miniKnob(X + W * 0.7, Y + 28, 9, 320),
        led(X + W / 2, Y + 50, n.led, n.on),
        h("text", { x: X + W / 2, y: Y + 74, textAnchor: "middle", className: "tk-chain__name" }, n.label),
        footswitch(p, X + W / 2, Y + H - 24));
    }
    return h("div", { className: cx("tk-chain", !live && "is-offline", props.className) },
      h("div", { className: "tk-chain__stage", style: { aspectRatio: (VW / 230).toFixed(4) } },
      h("svg", { viewBox: "0 28 " + VW + " 230", preserveAspectRatio: "xMidYMid meet", className: "tk-chain__svg", role: "group", "aria-label": props.label || "Cadeia de sinal" },
        h("defs", null,
          h("linearGradient", { id: uid + "-body", x1: 0, y1: 0, x2: 0, y2: 1 }, h("stop", { offset: 0, className: "tk-chain__b0" }), h("stop", { offset: 1, className: "tk-chain__b1" })),
          h("linearGradient", { id: uid + "-panel", x1: 0, y1: 0, x2: 0, y2: 1 }, h("stop", { offset: 0, className: "tk-chain__p0" }), h("stop", { offset: 0.5, className: "tk-chain__p1" }), h("stop", { offset: 1, className: "tk-chain__p2" })),
          h("radialGradient", { id: uid + "-metal", cx: "35%", cy: "30%", r: "75%" }, h("stop", { offset: 0, className: "tk-chain__m0" }), h("stop", { offset: 1, className: "tk-chain__m1" })),
          h("radialGradient", { id: uid + "-cone", cx: "50%", cy: "45%", r: "60%" }, h("stop", { offset: 0, className: "tk-chain__c0" }), h("stop", { offset: 0.7, className: "tk-chain__c1" }), h("stop", { offset: 1, className: "tk-chain__c2" })),
          h("radialGradient", { id: uid + "-dust", cx: "40%", cy: "35%", r: "70%" }, h("stop", { offset: 0, className: "tk-chain__d0" }), h("stop", { offset: 1, className: "tk-chain__d1" })),
          h("pattern", { id: uid + "-grille", width: 6, height: 6, patternUnits: "userSpaceOnUse", patternTransform: "rotate(45)" },
            h("rect", { width: 6, height: 6, className: "tk-chain__gbg" }), h("rect", { width: 3, height: 6, className: "tk-chain__gfg" }))),
        h("g", { className: "tk-chain__floor" }, h("line", { x1: 0, y1: FLOOR + 0.5, x2: VW, y2: FLOOR + 0.5 })),
        cables,
        h("g", { className: "tk-chain__jack" },
          h("circle", { cx: inX, cy: CABLE_Y, r: 11 }), h("circle", { cx: inX, cy: CABLE_Y, r: 5, className: "tk-chain__hole" }),
          h("text", { x: inX, y: 238, textAnchor: "middle", className: "tk-chain__lbl" }, "IN"),
          h("circle", { cx: outX, cy: CABLE_Y, r: 11 }), h("circle", { cx: outX, cy: CABLE_Y, r: 5, className: "tk-chain__hole" }),
          h("text", { x: outX, y: 238, textAnchor: "middle", className: "tk-chain__lbl" }, "OUT")),
        placed.map(function (p, i) {
          var n = p.n, on = sel[0] === n.id;
          return h("g", {
            key: n.id, className: cx("tk-chain__node", "tk-chain__node--" + n.kind, on && "is-sel", !n.on && "is-off"),
            role: "button", tabIndex: 0, "aria-pressed": on, "aria-label": n.label + (n.on ? "" : " (desligado)") + (n.sub ? ", " + n.sub : ""),
            onClick: function () { sel[1](n.id); },
            onKeyDown: function (e) {
              if (e.key === "Enter" || e.key === " ") { e.preventDefault(); sel[1](n.id); }
              if (e.key === "ArrowRight" && i < placed.length - 1) { e.preventDefault(); sel[1](placed[i + 1].n.id); var nx = e.currentTarget.parentNode.querySelectorAll(".tk-chain__node")[i + 1]; if (nx) nx.focus(); }
              if (e.key === "ArrowLeft" && i > 0) { e.preventDefault(); sel[1](placed[i - 1].n.id); var pv = e.currentTarget.parentNode.querySelectorAll(".tk-chain__node")[i - 1]; if (pv) pv.focus(); }
            }
          },
            h("rect", { x: p.x - 8, y: p.y - (n.kind === "amp" ? 20 : 9), width: p.w + 16, height: p.h + (n.kind === "amp" ? 28 : 17), rx: 14, className: "tk-chain__sel" }),
            plug(p.x, -1), plug(p.x + p.w, 1),
            h("g", { className: "tk-chain__art" }, drawNode(p)),
            h("text", { x: p.cx, y: 238, textAnchor: "middle", className: "tk-chain__lbl" }, n.label + (n.on ? "" : " · OFF")),
            n.sub && h("text", { x: p.cx, y: 252, textAnchor: "middle", className: "tk-chain__sub" }, n.sub));
        }))));
  }

  /* ---------- PresetEditor ---------- */
  var DEFAULT_PRESET = {
    name: "Crunch Britânico", color: "orange", ircab: 3, irlv: 0, gate: 0,
    reverb: { on: true, mix: 1, decay: 12 }, delay: { on: false, time: 1, mix: 1 }, mod: { on: true, speed: 1, fx: 8 },
    amp: { on: true, volume: 80, bass: 64, middle: 64, treble: 64, gain: 70, type: 8 }
  };
  var DEFAULT_AMPS = ["Clean US", "Clean UK", "Chime", "Plexi", "Crunch", "Brit Hi", "US Hi", "Modern", "Bass"];
  function clone(o) { return JSON.parse(JSON.stringify(o)); }
  function PresetEditor(props) {
    var initial = props.preset || DEFAULT_PRESET;
    var base = useState(function () { return clone(initial); });
    var cur = useState(function () { return clone(initial); });
    var p = cur[0];
    var pidS = useState(props.presetId || "2D");
    var selS = useState(props.defaultSelected || "amp");
    var dirty = JSON.stringify(p) !== JSON.stringify(base[0]);
    var ampTypes = props.ampTypes || DEFAULT_AMPS;
    var irNames = props.irNames || null;
    function get(obj, path) { path.split(".").forEach(function (x) { obj = obj[x]; }); return obj; }
    function set(path, val) {
      cur[1](function (prev) {
        var n = clone(prev), o = n, keys = path.split(".");
        for (var i = 0; i < keys.length - 1; i++) o = o[keys[i]];
        o[keys[keys.length - 1]] = val;
        if (props.onChange) props.onChange(n, path, val);
        return n;
      });
    }
    function k(path, label, opts) {
      var saved = get(base[0], path);
      return h(Knob, Object.assign({ key: path, label: label, value: get(p, path), resetValue: saved,
        hint: "Arraste, role ou use as setas · Shift = ajuste fino · duplo clique = valor salvo (" + saved + ")",
        onChange: function (v) { set(path, v); } }, opts || {}));
    }
    var irLabel = irNames ? (irNames[p.ircab - 1] || "Slot " + p.ircab) : "Slot " + p.ircab;
    var nodes = [
      { id: "gate", kind: "gate", label: "GATE", on: p.gate > 0, led: "green", sub: p.gate > 0 ? "Gate " + p.gate : "Aberto", amount: p.gate },
      { id: "amp", kind: "amp", label: "AMP", on: p.amp.on, sub: ampTypes[p.amp.type - 1] || ("Tipo " + p.amp.type) },
      { id: "mod", kind: "stomp", label: "MOD", on: p.mod.on, led: "purple", sub: "FX " + p.mod.fx },
      { id: "delay", kind: "stomp", label: "DELAY", on: p.delay.on, led: "cyan", sub: "Time " + p.delay.time },
      { id: "reverb", kind: "stomp", label: "REVERB", on: p.reverb.on, led: "blue", sub: "Decay " + p.reverb.decay },
      { id: "cab", kind: "cab", label: "CAB · IR", on: true, sub: "Slot " + p.ircab }
    ];
    var sel = selS[0];
    var selNode = nodes.filter(function (n) { return n.id === sel; })[0] || nodes[1];
    var detail;
    if (sel === "gate") detail = h(EffectBlock, { title: "NOISE GATE", subtitle: "antes do amp" }, k("gate", "NOISE GATE", { size: "lg", min: 0, max: 100 }));
    else if (sel === "amp") detail = h(EffectBlock, { title: "AMP", subtitle: ampTypes[p.amp.type - 1], enabled: p.amp.on, onToggle: function (v) { set("amp.on", v); } },
      k("amp.gain", "GAIN", { size: "lg" }), k("amp.volume", "VOLUME", { size: "lg" }),
      k("amp.bass", "BASS"), k("amp.middle", "MIDDLE"), k("amp.treble", "TREBLE"),
      k("amp.type", "AMP TYPE", { min: 1, max: ampTypes.length, valueLabels: ampTypes }));
    else if (sel === "mod") detail = h(EffectBlock, { title: "MOD", enabled: p.mod.on, onToggle: function (v) { set("mod.on", v); } },
      k("mod.speed", "MOD SPEED", { size: "lg" }), k("mod.fx", "MOD FX", { size: "lg", min: 1, max: 9, ticks: true }));
    else if (sel === "delay") detail = h(EffectBlock, { title: "DELAY", enabled: p.delay.on, onToggle: function (v) { set("delay.on", v); } },
      k("delay.time", "DLY TIME", { size: "lg" }), k("delay.mix", "DLY MIX", { size: "lg" }));
    else if (sel === "reverb") detail = h(EffectBlock, { title: "REVERB", enabled: p.reverb.on, onToggle: function (v) { set("reverb.on", v); } },
      k("reverb.mix", "RVB MIX", { size: "lg" }), k("reverb.decay", "RVB DECAY", { size: "lg" }));
    else detail = h(EffectBlock, { title: "CAB · IR", subtitle: irLabel },
      k("ircab", "IR CAB", { size: "lg", min: 1, max: props.irSlots || 8, ticks: true }),
      h("div", { className: "tk-editor__irside" },
        irNames && h(Select, { label: "IR no slot", size: "sm", width: 260, options: irNames.map(function (nm, i) { return { value: i + 1, label: (i + 1) + " · " + nm }; }), value: p.ircab, onChange: function (v) { set("ircab", v); } }),
        h(Fader, { label: "IR LV", value: p.irlv, onChange: function (v) { set("irlv", v); }, min: -12, max: 12, step: 0.5, unit: "dB", width: 260, resetValue: get(base[0], "irlv"), format: function (x) { return (x > 0 ? "+" : "") + x.toFixed(1); } })));
    return h("div", { className: cx("tk-editor", props.className) },
      h("div", { className: "tk-editor__top" },
        h(LedDisplay, { value: pidS[0], digits: 2, height: 46, label: "Preset " + pidS[0] }),
        h("div", { className: "tk-editor__title" },
          h("div", { className: "tk-editor__name" }, p.name),
          h("div", { className: "tk-editor__meta" },
            (props.presets || []).length ? h(Select, { size: "sm", label: "Preset", hideLabel: true, options: props.presets.map(function (x) { return { value: x.id, label: x.id + " · " + x.name }; }), value: pidS[0], onChange: pidS[1], width: 190 }) : null,
            h(LedColorPicker, { value: p.color, onChange: function (c) { set("color", c); }, showName: false }),
            dirty ? h(Tag, { tone: "warn", icon: "alert" }, "Modificado") : h(Tag, { tone: "ok", icon: "check" }, "Salvo no pedal"))),
        h("span", { className: "tk-editor__spacer" }),
        h("div", { className: "tk-editor__side" },
          props.connection !== false && h(ConnectionStatus, props.connection || { transport: "usb", state: "connected", device: "Tank-G" }),
          h("div", { className: "tk-editor__actions" },
            h(Button, { variant: "ghost", icon: "undo", disabled: !dirty, onClick: function () { cur[1](clone(base[0])); } }, "Reverter"),
            h(Button, { variant: "primary", icon: "send", kbd: "Ctrl S", disabled: !dirty, onClick: function () { base[1](clone(p)); if (props.onSave) props.onSave(p, pidS[0]); } }, "Gravar no pedal")))),
      h(SignalChain, {
        nodes: nodes, selected: sel, onSelect: selS[1], signal: !props.connection || props.connection.state === "connected",
        onToggle: function (id, on) {
          if (id === "gate") set("gate", on ? (get(base[0], "gate") || 30) : 0);
          else if (id !== "cab") set(id + ".on", on);
        }
      }),
      h("div", { className: "tk-editor__detail", key: sel }, detail));
  }

  var api = {
    Icon: Icon, Kbd: Kbd, Button: Button, SegmentedControl: SegmentedControl, Knob: Knob, Fader: Fader, Toggle: Toggle, Select: Select,
    LedColorPicker: LedColorPicker, LedDisplay: LedDisplay, ConnectionStatus: ConnectionStatus, Tag: Tag, ProgressBar: ProgressBar,
    SearchField: SearchField, CommandPalette: CommandPalette, LibraryTable: LibraryTable, PresetGrid: PresetGrid, Panel: Panel,
    EffectBlock: EffectBlock, SignalChain: SignalChain, PresetEditor: PresetEditor, MidiMonitor: MidiMonitor,
    LedDot: LedDot, Highlight: Highlight, fuzzy: fuzzy, ICON_NAMES: Object.keys(ICONS), LED_COLORS: LED_COLORS
  };
  window.Tank = Object.assign(window.Tank || {}, api);
})();

})();
