var g0 = (w, K, O) => new Promise((_e, Qe) => {
  var rt = ue => {
      try {
        ye(O.next(ue));
      } catch (Re) {
        Qe(Re);
      }
    },
    we = ue => {
      try {
        ye(O.throw(ue));
      } catch (Re) {
        Qe(Re);
      }
    },
    ye = ue => ue.done ? _e(ue.value) : Promise.resolve(ue.value).then(rt, we);
  ye((O = O.apply(w, K)).next());
});
function _slicedToArray(w, K) {
  return _arrayWithHoles(w) || _iterableToArrayLimit(w, K) || _unsupportedIterableToArray(w, K) || _nonIterableRest();
}
function _nonIterableRest() {
  throw new TypeError("Invalid attempt to destructure non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.");
}
function _unsupportedIterableToArray(w, K) {
  if (w) {
    if (typeof w == "string") return _arrayLikeToArray(w, K);
    var O = {}.toString.call(w).slice(8, -1);
    return O === "Object" && w.constructor && (O = w.constructor.name), O === "Map" || O === "Set" ? Array.from(w) : O === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(O) ? _arrayLikeToArray(w, K) : void 0;
  }
}
function _arrayLikeToArray(w, K) {
  (K == null || K > w.length) && (K = w.length);
  for (var O = 0, _e = Array(K); O < K; O++) _e[O] = w[O];
  return _e;
}
function _iterableToArrayLimit(w, K) {
  var O = w == null ? null : typeof Symbol < "u" && w[Symbol.iterator] || w["@@iterator"];
  if (O != null) {
    var _e,
      Qe,
      rt,
      we,
      ye = [],
      ue = !0,
      Re = !1;
    try {
      if (rt = (O = O.call(w)).next, K === 0) {
        if (Object(O) !== O) return;
        ue = !1;
      } else for (; !(ue = (_e = rt.call(O)).done) && (ye.push(_e.value), ye.length !== K); ue = !0);
    } catch (Pt) {
      Re = !0, Qe = Pt;
    } finally {
      try {
        if (!ue && O.return != null && (we = O.return(), Object(we) !== we)) return;
      } finally {
        if (Re) throw Qe;
      }
    }
    return ye;
  }
}
function _arrayWithHoles(w) {
  if (Array.isArray(w)) return w;
}
(() => {
  var w = Object.create,
    K = Object.defineProperty,
    O = Object.defineProperties,
    _e = Object.getOwnPropertyDescriptor,
    Qe = Object.getOwnPropertyDescriptors,
    rt = Object.getOwnPropertyNames,
    we = Object.getOwnPropertySymbols,
    ye = Object.getPrototypeOf,
    ue = Object.prototype.hasOwnProperty,
    Re = Object.prototype.propertyIsEnumerable,
    Pt = (e, t, n) => t in e ? K(e, t, {
      enumerable: !0,
      configurable: !0,
      writable: !0,
      value: n
    }) : e[t] = n,
    L = (e, t) => {
      for (var n in t || (t = {})) ue.call(t, n) && Pt(e, n, t[n]);
      if (we) for (var n of we(t)) Re.call(t, n) && Pt(e, n, t[n]);
      return e;
    },
    Ye = (e, t) => O(e, Qe(t)),
    Xu = (e, t) => () => {
      try {
        return t || e((t = {
          exports: {}
        }).exports, t), t.exports;
      } catch (n) {
        throw t = 0, n;
      }
    },
    le = (e, t) => {
      for (var n in t) K(e, n, {
        get: t[n],
        enumerable: !0
      });
    },
    Ju = (e, t, n, r) => {
      if (t && typeof t == "object" || typeof t == "function") for (let a of rt(t)) !ue.call(e, a) && a !== n && K(e, a, {
        get: () => t[a],
        enumerable: !(r = _e(t, a)) || r.enumerable
      });
      return e;
    },
    Xe = (e, t, n) => (n = e != null ? w(ye(e)) : {}, Ju(t || !e || !e.__esModule ? K(n, "default", {
      value: e,
      enumerable: !0
    }) : n, e)),
    Je = Xu((e, t) => {
      t.exports = {
        trueFunc: function () {
          return !0;
        },
        falseFunc: function () {
          return !1;
        }
      };
    }),
    T0 = {};
  le(T0, {
    contains: () => Rn,
    load: () => i1,
    merge: () => er
  });
  var F0 = {};
  le(F0, {
    contains: () => Rn,
    extract: () => za,
    html: () => qa,
    merge: () => er,
    parseHTML: () => ja,
    root: () => Za,
    text: () => pt,
    xml: () => Va
  });
  var ht = {};
  le(ht, {
    DocumentPosition: () => Ee,
    append: () => Sa,
    appendChild: () => ba,
    compareDocumentPosition: () => Z0,
    existsOne: () => V0,
    filter: () => Ct,
    find: () => vn,
    findAll: () => ka,
    findOne: () => Nn,
    findOneChild: () => Na,
    getAttributeValue: () => Ta,
    getChildren: () => Yt,
    getElementById: () => La,
    getElements: () => Oa,
    getElementsByClassName: () => Ma,
    getElementsByTagName: () => at,
    getElementsByTagType: () => Ha,
    getFeed: () => Ua,
    getInnerHTML: () => ga,
    getName: () => _a,
    getOuterHTML: () => X0,
    getParent: () => J0,
    getSiblings: () => q0,
    getText: () => Kt,
    hasAttrib: () => Fa,
    hasChildren: () => Q,
    innerText: () => Qt,
    isCDATA: () => Ut,
    isComment: () => Wt,
    isDocument: () => Oe,
    isTag: () => y,
    isText: () => Be,
    nextElementSibling: () => Sn,
    prepend: () => va,
    prependChild: () => Ia,
    prevElementSibling: () => In,
    removeElement: () => Ze,
    removeSubsets: () => Pa,
    replaceElement: () => ya,
    testElement: () => Ra,
    textContent: () => ut,
    uniqueSort: () => it
  });
  var qe = {};
  le(qe, {
    CDATA: () => w0,
    Comment: () => I0,
    Directive: () => S0,
    Doctype: () => R0,
    ElementType: () => N,
    Root: () => y0,
    Script: () => v0,
    Style: () => N0,
    Tag: () => k0,
    Text: () => b0,
    isTag: () => _0
  });
  var N;
  (function (e) {
    e.Root = "root", e.Text = "text", e.Directive = "directive", e.Comment = "comment", e.Script = "script", e.Style = "style", e.Tag = "tag", e.CDATA = "cdata", e.Doctype = "doctype";
  })(N || (N = {}));
  function _0(e) {
    return e.type === N.Tag || e.type === N.Script || e.type === N.Style;
  }
  var y0 = N.Root,
    b0 = N.Text,
    S0 = N.Directive,
    I0 = N.Comment,
    v0 = N.Script,
    N0 = N.Style,
    k0 = N.Tag,
    w0 = N.CDATA,
    R0 = N.Doctype,
    O0 = class {
      constructor() {
        this.parent = null, this.prev = null, this.next = null, this.startIndex = null, this.endIndex = null;
      }
      get parentNode() {
        return this.parent;
      }
      set parentNode(e) {
        this.parent = e;
      }
      get previousSibling() {
        return this.prev;
      }
      set previousSibling(e) {
        this.prev = e;
      }
      get nextSibling() {
        return this.next;
      }
      set nextSibling(e) {
        this.next = e;
      }
      cloneNode(e = !1) {
        return Et(this, e);
      }
    },
    fn = class extends O0 {
      constructor(e) {
        super(), this.data = e;
      }
      get nodeValue() {
        return this.data;
      }
      set nodeValue(e) {
        this.data = e;
      }
    },
    dt = class extends fn {
      constructor() {
        super(...arguments), this.type = N.Text;
      }
      get nodeType() {
        return 3;
      }
    },
    xn = class extends fn {
      constructor() {
        super(...arguments), this.type = N.Comment;
      }
      get nodeType() {
        return 8;
      }
    },
    Bn = class extends fn {
      constructor(e, t) {
        super(t), this.name = e, this.type = N.Directive;
      }
      get nodeType() {
        return 1;
      }
    },
    mn = class extends O0 {
      constructor(e) {
        super(), this.children = e;
      }
      get firstChild() {
        var e;
        return (e = this.children[0]) !== null && e !== void 0 ? e : null;
      }
      get lastChild() {
        return this.children.length > 0 ? this.children[this.children.length - 1] : null;
      }
      get childNodes() {
        return this.children;
      }
      set childNodes(e) {
        this.children = e;
      }
    },
    L0 = class extends mn {
      constructor() {
        super(...arguments), this.type = N.CDATA;
      }
      get nodeType() {
        return 4;
      }
    },
    Ve = class extends mn {
      constructor() {
        super(...arguments), this.type = N.Root;
      }
      get nodeType() {
        return 9;
      }
    },
    Dn = class extends mn {
      constructor(e, t, n = [], r = e === "script" ? N.Script : e === "style" ? N.Style : N.Tag) {
        super(n), this.name = e, this.attribs = t, this.type = r;
      }
      get nodeType() {
        return 1;
      }
      get tagName() {
        return this.name;
      }
      set tagName(e) {
        this.name = e;
      }
      get attributes() {
        return Object.keys(this.attribs).map(e => {
          var t, n;
          return {
            name: e,
            value: this.attribs[e],
            namespace: (t = this["x-attribsNamespace"]) === null || t === void 0 ? void 0 : t[e],
            prefix: (n = this["x-attribsPrefix"]) === null || n === void 0 ? void 0 : n[e]
          };
        });
      }
    };
  function y(e) {
    return _0(e);
  }
  function Ut(e) {
    return e.type === N.CDATA;
  }
  function Be(e) {
    return e.type === N.Text;
  }
  function Wt(e) {
    return e.type === N.Comment;
  }
  function gn(e) {
    return e.type === N.Directive;
  }
  function Oe(e) {
    return e.type === N.Root;
  }
  function Q(e) {
    return Object.prototype.hasOwnProperty.call(e, "children");
  }
  function Et(e, t = !1) {
    let n;
    if (Be(e)) n = new dt(e.data);else if (Wt(e)) n = new xn(e.data);else if (y(e)) {
      let r = t ? Tn(e.children) : [],
        a = new Dn(e.name, L({}, e.attribs), r);
      r.forEach(i => i.parent = a), e.namespace != null && (a.namespace = e.namespace), e["x-attribsNamespace"] && (a["x-attribsNamespace"] = L({}, e["x-attribsNamespace"])), e["x-attribsPrefix"] && (a["x-attribsPrefix"] = L({}, e["x-attribsPrefix"])), n = a;
    } else if (Ut(e)) {
      let r = t ? Tn(e.children) : [],
        a = new L0(r);
      r.forEach(i => i.parent = a), n = a;
    } else if (Oe(e)) {
      let r = t ? Tn(e.children) : [],
        a = new Ve(r);
      r.forEach(i => i.parent = a), e["x-mode"] && (a["x-mode"] = e["x-mode"]), n = a;
    } else if (gn(e)) {
      let r = new Bn(e.name, e.data);
      e["x-name"] != null && (r["x-name"] = e["x-name"], r["x-publicId"] = e["x-publicId"], r["x-systemId"] = e["x-systemId"]), n = r;
    } else throw new Error("Not implemented yet: ".concat(e.type));
    return n.startIndex = e.startIndex, n.endIndex = e.endIndex, e.sourceCodeLocation != null && (n.sourceCodeLocation = e.sourceCodeLocation), n;
  }
  function Tn(e) {
    let t = e.map(n => Et(n, !0));
    for (let n = 1; n < t.length; n++) t[n].prev = t[n - 1], t[n - 1].next = t[n];
    return t;
  }
  var M0 = {
      withStartIndices: !1,
      withEndIndices: !1,
      xmlMode: !1
    },
    qu = class {
      constructor(e, t, n) {
        this.dom = [], this.root = new Ve(this.dom), this.done = !1, this.tagStack = [this.root], this.lastNode = null, this.parser = null, typeof t == "function" && (n = t, t = M0), typeof e == "object" && (t = e, e = void 0), this.callback = e != null ? e : null, this.options = t != null ? t : M0, this.elementCB = n != null ? n : null;
      }
      onparserinit(e) {
        this.parser = e;
      }
      onreset() {
        this.dom = [], this.root = new Ve(this.dom), this.done = !1, this.tagStack = [this.root], this.lastNode = null, this.parser = null;
      }
      onend() {
        this.done || (this.done = !0, this.parser = null, this.handleCallback(null));
      }
      onerror(e) {
        this.handleCallback(e);
      }
      onclosetag() {
        this.lastNode = null;
        let e = this.tagStack.pop();
        this.options.withEndIndices && (e.endIndex = this.parser.endIndex), this.elementCB && this.elementCB(e);
      }
      onopentag(e, t) {
        let n = this.options.xmlMode ? N.Tag : void 0,
          r = new Dn(e, t, void 0, n);
        this.addNode(r), this.tagStack.push(r);
      }
      ontext(e) {
        let t = this.lastNode;
        if (t && t.type === N.Text) t.data += e, this.options.withEndIndices && (t.endIndex = this.parser.endIndex);else {
          let n = new dt(e);
          this.addNode(n), this.lastNode = n;
        }
      }
      oncomment(e) {
        if (this.lastNode && this.lastNode.type === N.Comment) {
          this.lastNode.data += e;
          return;
        }
        let t = new xn(e);
        this.addNode(t), this.lastNode = t;
      }
      oncommentend() {
        this.lastNode = null;
      }
      oncdatastart() {
        let e = new dt(""),
          t = new L0([e]);
        this.addNode(t), e.parent = t, this.lastNode = e;
      }
      oncdataend() {
        this.lastNode = null;
      }
      onprocessinginstruction(e, t) {
        let n = new Bn(e, t);
        this.addNode(n);
      }
      handleCallback(e) {
        if (typeof this.callback == "function") this.callback(e, this.dom);else if (e) throw e;
      }
      addNode(e) {
        let t = this.tagStack[this.tagStack.length - 1],
          n = t.children[t.children.length - 1];
        this.options.withStartIndices && (e.startIndex = this.parser.startIndex), this.options.withEndIndices && (e.endIndex = this.parser.endIndex), t.children.push(e), n && (e.prev = n, n.next = e), e.parent = t, this.lastNode = null;
      }
    },
    Vu = new Uint16Array('\u1D41<\xD5\u0131\u028A\u049D\u057B\u05D0\u0675\u06DE\u07A2\u07D6\u080F\u0A4A\u0A91\u0DA1\u0E6D\u0F09\u0F26\u10CA\u1228\u12E1\u1415\u149D\u14C3\u14DF\u1525\0\0\0\0\0\0\u156B\u16CD\u198D\u1C12\u1DDD\u1F7E\u2060\u21B0\u228D\u23C0\u23FB\u2442\u2824\u2912\u2D08\u2E48\u2FCE\u3016\u32BA\u3639\u37AC\u38FE\u3A28\u3A71\u3AE0\u3B2E\u0800EMabcfglmnoprstu\\bfms\x7F\x84\x8B\x90\x95\x98\xA6\xB3\xB9\xC8\xCFlig\u803B\xC6\u40C6P\u803B&\u4026cute\u803B\xC1\u40C1reve;\u4102\u0100iyx}rc\u803B\xC2\u40C2;\u4410r;\uC000\u{1D504}rave\u803B\xC0\u40C0pha;\u4391acr;\u4100d;\u6A53\u0100gp\x9D\xA1on;\u4104f;\uC000\u{1D538}plyFunction;\u6061ing\u803B\xC5\u40C5\u0100cs\xBE\xC3r;\uC000\u{1D49C}ign;\u6254ilde\u803B\xC3\u40C3ml\u803B\xC4\u40C4\u0400aceforsu\xE5\xFB\xFE\u0117\u011C\u0122\u0127\u012A\u0100cr\xEA\xF2kslash;\u6216\u0176\xF6\xF8;\u6AE7ed;\u6306y;\u4411\u0180crt\u0105\u010B\u0114ause;\u6235noullis;\u612Ca;\u4392r;\uC000\u{1D505}pf;\uC000\u{1D539}eve;\u42D8c\xF2\u0113mpeq;\u624E\u0700HOacdefhilorsu\u014D\u0151\u0156\u0180\u019E\u01A2\u01B5\u01B7\u01BA\u01DC\u0215\u0273\u0278\u027Ecy;\u4427PY\u803B\xA9\u40A9\u0180cpy\u015D\u0162\u017Aute;\u4106\u0100;i\u0167\u0168\u62D2talDifferentialD;\u6145leys;\u612D\u0200aeio\u0189\u018E\u0194\u0198ron;\u410Cdil\u803B\xC7\u40C7rc;\u4108nint;\u6230ot;\u410A\u0100dn\u01A7\u01ADilla;\u40B8terDot;\u40B7\xF2\u017Fi;\u43A7rcle\u0200DMPT\u01C7\u01CB\u01D1\u01D6ot;\u6299inus;\u6296lus;\u6295imes;\u6297o\u0100cs\u01E2\u01F8kwiseContourIntegral;\u6232eCurly\u0100DQ\u0203\u020FoubleQuote;\u601Duote;\u6019\u0200lnpu\u021E\u0228\u0247\u0255on\u0100;e\u0225\u0226\u6237;\u6A74\u0180git\u022F\u0236\u023Aruent;\u6261nt;\u622FourIntegral;\u622E\u0100fr\u024C\u024E;\u6102oduct;\u6210nterClockwiseContourIntegral;\u6233oss;\u6A2Fcr;\uC000\u{1D49E}p\u0100;C\u0284\u0285\u62D3ap;\u624D\u0580DJSZacefios\u02A0\u02AC\u02B0\u02B4\u02B8\u02CB\u02D7\u02E1\u02E6\u0333\u048D\u0100;o\u0179\u02A5trahd;\u6911cy;\u4402cy;\u4405cy;\u440F\u0180grs\u02BF\u02C4\u02C7ger;\u6021r;\u61A1hv;\u6AE4\u0100ay\u02D0\u02D5ron;\u410E;\u4414l\u0100;t\u02DD\u02DE\u6207a;\u4394r;\uC000\u{1D507}\u0100af\u02EB\u0327\u0100cm\u02F0\u0322ritical\u0200ADGT\u0300\u0306\u0316\u031Ccute;\u40B4o\u0174\u030B\u030D;\u42D9bleAcute;\u42DDrave;\u4060ilde;\u42DCond;\u62C4ferentialD;\u6146\u0470\u033D\0\0\0\u0342\u0354\0\u0405f;\uC000\u{1D53B}\u0180;DE\u0348\u0349\u034D\u40A8ot;\u60DCqual;\u6250ble\u0300CDLRUV\u0363\u0372\u0382\u03CF\u03E2\u03F8ontourIntegra\xEC\u0239o\u0274\u0379\0\0\u037B\xBB\u0349nArrow;\u61D3\u0100eo\u0387\u03A4ft\u0180ART\u0390\u0396\u03A1rrow;\u61D0ightArrow;\u61D4e\xE5\u02CAng\u0100LR\u03AB\u03C4eft\u0100AR\u03B3\u03B9rrow;\u67F8ightArrow;\u67FAightArrow;\u67F9ight\u0100AT\u03D8\u03DErrow;\u61D2ee;\u62A8p\u0241\u03E9\0\0\u03EFrrow;\u61D1ownArrow;\u61D5erticalBar;\u6225n\u0300ABLRTa\u0412\u042A\u0430\u045E\u047F\u037Crrow\u0180;BU\u041D\u041E\u0422\u6193ar;\u6913pArrow;\u61F5reve;\u4311eft\u02D2\u043A\0\u0446\0\u0450ightVector;\u6950eeVector;\u695Eector\u0100;B\u0459\u045A\u61BDar;\u6956ight\u01D4\u0467\0\u0471eeVector;\u695Fector\u0100;B\u047A\u047B\u61C1ar;\u6957ee\u0100;A\u0486\u0487\u62A4rrow;\u61A7\u0100ct\u0492\u0497r;\uC000\u{1D49F}rok;\u4110\u0800NTacdfglmopqstux\u04BD\u04C0\u04C4\u04CB\u04DE\u04E2\u04E7\u04EE\u04F5\u0521\u052F\u0536\u0552\u055D\u0560\u0565G;\u414AH\u803B\xD0\u40D0cute\u803B\xC9\u40C9\u0180aiy\u04D2\u04D7\u04DCron;\u411Arc\u803B\xCA\u40CA;\u442Dot;\u4116r;\uC000\u{1D508}rave\u803B\xC8\u40C8ement;\u6208\u0100ap\u04FA\u04FEcr;\u4112ty\u0253\u0506\0\0\u0512mallSquare;\u65FBerySmallSquare;\u65AB\u0100gp\u0526\u052Aon;\u4118f;\uC000\u{1D53C}silon;\u4395u\u0100ai\u053C\u0549l\u0100;T\u0542\u0543\u6A75ilde;\u6242librium;\u61CC\u0100ci\u0557\u055Ar;\u6130m;\u6A73a;\u4397ml\u803B\xCB\u40CB\u0100ip\u056A\u056Fsts;\u6203onentialE;\u6147\u0280cfios\u0585\u0588\u058D\u05B2\u05CCy;\u4424r;\uC000\u{1D509}lled\u0253\u0597\0\0\u05A3mallSquare;\u65FCerySmallSquare;\u65AA\u0370\u05BA\0\u05BF\0\0\u05C4f;\uC000\u{1D53D}All;\u6200riertrf;\u6131c\xF2\u05CB\u0600JTabcdfgorst\u05E8\u05EC\u05EF\u05FA\u0600\u0612\u0616\u061B\u061D\u0623\u066C\u0672cy;\u4403\u803B>\u403Emma\u0100;d\u05F7\u05F8\u4393;\u43DCreve;\u411E\u0180eiy\u0607\u060C\u0610dil;\u4122rc;\u411C;\u4413ot;\u4120r;\uC000\u{1D50A};\u62D9pf;\uC000\u{1D53E}eater\u0300EFGLST\u0635\u0644\u064E\u0656\u065B\u0666qual\u0100;L\u063E\u063F\u6265ess;\u62DBullEqual;\u6267reater;\u6AA2ess;\u6277lantEqual;\u6A7Eilde;\u6273cr;\uC000\u{1D4A2};\u626B\u0400Aacfiosu\u0685\u068B\u0696\u069B\u069E\u06AA\u06BE\u06CARDcy;\u442A\u0100ct\u0690\u0694ek;\u42C7;\u405Eirc;\u4124r;\u610ClbertSpace;\u610B\u01F0\u06AF\0\u06B2f;\u610DizontalLine;\u6500\u0100ct\u06C3\u06C5\xF2\u06A9rok;\u4126mp\u0144\u06D0\u06D8ownHum\xF0\u012Fqual;\u624F\u0700EJOacdfgmnostu\u06FA\u06FE\u0703\u0707\u070E\u071A\u071E\u0721\u0728\u0744\u0778\u078B\u078F\u0795cy;\u4415lig;\u4132cy;\u4401cute\u803B\xCD\u40CD\u0100iy\u0713\u0718rc\u803B\xCE\u40CE;\u4418ot;\u4130r;\u6111rave\u803B\xCC\u40CC\u0180;ap\u0720\u072F\u073F\u0100cg\u0734\u0737r;\u412AinaryI;\u6148lie\xF3\u03DD\u01F4\u0749\0\u0762\u0100;e\u074D\u074E\u622C\u0100gr\u0753\u0758ral;\u622Bsection;\u62C2isible\u0100CT\u076C\u0772omma;\u6063imes;\u6062\u0180gpt\u077F\u0783\u0788on;\u412Ef;\uC000\u{1D540}a;\u4399cr;\u6110ilde;\u4128\u01EB\u079A\0\u079Ecy;\u4406l\u803B\xCF\u40CF\u0280cfosu\u07AC\u07B7\u07BC\u07C2\u07D0\u0100iy\u07B1\u07B5rc;\u4134;\u4419r;\uC000\u{1D50D}pf;\uC000\u{1D541}\u01E3\u07C7\0\u07CCr;\uC000\u{1D4A5}rcy;\u4408kcy;\u4404\u0380HJacfos\u07E4\u07E8\u07EC\u07F1\u07FD\u0802\u0808cy;\u4425cy;\u440Cppa;\u439A\u0100ey\u07F6\u07FBdil;\u4136;\u441Ar;\uC000\u{1D50E}pf;\uC000\u{1D542}cr;\uC000\u{1D4A6}\u0580JTaceflmost\u0825\u0829\u082C\u0850\u0863\u09B3\u09B8\u09C7\u09CD\u0A37\u0A47cy;\u4409\u803B<\u403C\u0280cmnpr\u0837\u083C\u0841\u0844\u084Dute;\u4139bda;\u439Bg;\u67EAlacetrf;\u6112r;\u619E\u0180aey\u0857\u085C\u0861ron;\u413Ddil;\u413B;\u441B\u0100fs\u0868\u0970t\u0500ACDFRTUVar\u087E\u08A9\u08B1\u08E0\u08E6\u08FC\u092F\u095B\u0390\u096A\u0100nr\u0883\u088FgleBracket;\u67E8row\u0180;BR\u0899\u089A\u089E\u6190ar;\u61E4ightArrow;\u61C6eiling;\u6308o\u01F5\u08B7\0\u08C3bleBracket;\u67E6n\u01D4\u08C8\0\u08D2eeVector;\u6961ector\u0100;B\u08DB\u08DC\u61C3ar;\u6959loor;\u630Aight\u0100AV\u08EF\u08F5rrow;\u6194ector;\u694E\u0100er\u0901\u0917e\u0180;AV\u0909\u090A\u0910\u62A3rrow;\u61A4ector;\u695Aiangle\u0180;BE\u0924\u0925\u0929\u62B2ar;\u69CFqual;\u62B4p\u0180DTV\u0937\u0942\u094CownVector;\u6951eeVector;\u6960ector\u0100;B\u0956\u0957\u61BFar;\u6958ector\u0100;B\u0965\u0966\u61BCar;\u6952ight\xE1\u039Cs\u0300EFGLST\u097E\u098B\u0995\u099D\u09A2\u09ADqualGreater;\u62DAullEqual;\u6266reater;\u6276ess;\u6AA1lantEqual;\u6A7Dilde;\u6272r;\uC000\u{1D50F}\u0100;e\u09BD\u09BE\u62D8ftarrow;\u61DAidot;\u413F\u0180npw\u09D4\u0A16\u0A1Bg\u0200LRlr\u09DE\u09F7\u0A02\u0A10eft\u0100AR\u09E6\u09ECrrow;\u67F5ightArrow;\u67F7ightArrow;\u67F6eft\u0100ar\u03B3\u0A0Aight\xE1\u03BFight\xE1\u03CAf;\uC000\u{1D543}er\u0100LR\u0A22\u0A2CeftArrow;\u6199ightArrow;\u6198\u0180cht\u0A3E\u0A40\u0A42\xF2\u084C;\u61B0rok;\u4141;\u626A\u0400acefiosu\u0A5A\u0A5D\u0A60\u0A77\u0A7C\u0A85\u0A8B\u0A8Ep;\u6905y;\u441C\u0100dl\u0A65\u0A6FiumSpace;\u605Flintrf;\u6133r;\uC000\u{1D510}nusPlus;\u6213pf;\uC000\u{1D544}c\xF2\u0A76;\u439C\u0480Jacefostu\u0AA3\u0AA7\u0AAD\u0AC0\u0B14\u0B19\u0D91\u0D97\u0D9Ecy;\u440Acute;\u4143\u0180aey\u0AB4\u0AB9\u0ABEron;\u4147dil;\u4145;\u441D\u0180gsw\u0AC7\u0AF0\u0B0Eative\u0180MTV\u0AD3\u0ADF\u0AE8ediumSpace;\u600Bhi\u0100cn\u0AE6\u0AD8\xEB\u0AD9eryThi\xEE\u0AD9ted\u0100GL\u0AF8\u0B06reaterGreate\xF2\u0673essLes\xF3\u0A48Line;\u400Ar;\uC000\u{1D511}\u0200Bnpt\u0B22\u0B28\u0B37\u0B3Areak;\u6060BreakingSpace;\u40A0f;\u6115\u0680;CDEGHLNPRSTV\u0B55\u0B56\u0B6A\u0B7C\u0BA1\u0BEB\u0C04\u0C5E\u0C84\u0CA6\u0CD8\u0D61\u0D85\u6AEC\u0100ou\u0B5B\u0B64ngruent;\u6262pCap;\u626DoubleVerticalBar;\u6226\u0180lqx\u0B83\u0B8A\u0B9Bement;\u6209ual\u0100;T\u0B92\u0B93\u6260ilde;\uC000\u2242\u0338ists;\u6204reater\u0380;EFGLST\u0BB6\u0BB7\u0BBD\u0BC9\u0BD3\u0BD8\u0BE5\u626Fqual;\u6271ullEqual;\uC000\u2267\u0338reater;\uC000\u226B\u0338ess;\u6279lantEqual;\uC000\u2A7E\u0338ilde;\u6275ump\u0144\u0BF2\u0BFDownHump;\uC000\u224E\u0338qual;\uC000\u224F\u0338e\u0100fs\u0C0A\u0C27tTriangle\u0180;BE\u0C1A\u0C1B\u0C21\u62EAar;\uC000\u29CF\u0338qual;\u62ECs\u0300;EGLST\u0C35\u0C36\u0C3C\u0C44\u0C4B\u0C58\u626Equal;\u6270reater;\u6278ess;\uC000\u226A\u0338lantEqual;\uC000\u2A7D\u0338ilde;\u6274ested\u0100GL\u0C68\u0C79reaterGreater;\uC000\u2AA2\u0338essLess;\uC000\u2AA1\u0338recedes\u0180;ES\u0C92\u0C93\u0C9B\u6280qual;\uC000\u2AAF\u0338lantEqual;\u62E0\u0100ei\u0CAB\u0CB9verseElement;\u620CghtTriangle\u0180;BE\u0CCB\u0CCC\u0CD2\u62EBar;\uC000\u29D0\u0338qual;\u62ED\u0100qu\u0CDD\u0D0CuareSu\u0100bp\u0CE8\u0CF9set\u0100;E\u0CF0\u0CF3\uC000\u228F\u0338qual;\u62E2erset\u0100;E\u0D03\u0D06\uC000\u2290\u0338qual;\u62E3\u0180bcp\u0D13\u0D24\u0D4Eset\u0100;E\u0D1B\u0D1E\uC000\u2282\u20D2qual;\u6288ceeds\u0200;EST\u0D32\u0D33\u0D3B\u0D46\u6281qual;\uC000\u2AB0\u0338lantEqual;\u62E1ilde;\uC000\u227F\u0338erset\u0100;E\u0D58\u0D5B\uC000\u2283\u20D2qual;\u6289ilde\u0200;EFT\u0D6E\u0D6F\u0D75\u0D7F\u6241qual;\u6244ullEqual;\u6247ilde;\u6249erticalBar;\u6224cr;\uC000\u{1D4A9}ilde\u803B\xD1\u40D1;\u439D\u0700Eacdfgmoprstuv\u0DBD\u0DC2\u0DC9\u0DD5\u0DDB\u0DE0\u0DE7\u0DFC\u0E02\u0E20\u0E22\u0E32\u0E3F\u0E44lig;\u4152cute\u803B\xD3\u40D3\u0100iy\u0DCE\u0DD3rc\u803B\xD4\u40D4;\u441Eblac;\u4150r;\uC000\u{1D512}rave\u803B\xD2\u40D2\u0180aei\u0DEE\u0DF2\u0DF6cr;\u414Cga;\u43A9cron;\u439Fpf;\uC000\u{1D546}enCurly\u0100DQ\u0E0E\u0E1AoubleQuote;\u601Cuote;\u6018;\u6A54\u0100cl\u0E27\u0E2Cr;\uC000\u{1D4AA}ash\u803B\xD8\u40D8i\u016C\u0E37\u0E3Cde\u803B\xD5\u40D5es;\u6A37ml\u803B\xD6\u40D6er\u0100BP\u0E4B\u0E60\u0100ar\u0E50\u0E53r;\u603Eac\u0100ek\u0E5A\u0E5C;\u63DEet;\u63B4arenthesis;\u63DC\u0480acfhilors\u0E7F\u0E87\u0E8A\u0E8F\u0E92\u0E94\u0E9D\u0EB0\u0EFCrtialD;\u6202y;\u441Fr;\uC000\u{1D513}i;\u43A6;\u43A0usMinus;\u40B1\u0100ip\u0EA2\u0EADncareplan\xE5\u069Df;\u6119\u0200;eio\u0EB9\u0EBA\u0EE0\u0EE4\u6ABBcedes\u0200;EST\u0EC8\u0EC9\u0ECF\u0EDA\u627Aqual;\u6AAFlantEqual;\u627Cilde;\u627Eme;\u6033\u0100dp\u0EE9\u0EEEuct;\u620Fortion\u0100;a\u0225\u0EF9l;\u621D\u0100ci\u0F01\u0F06r;\uC000\u{1D4AB};\u43A8\u0200Ufos\u0F11\u0F16\u0F1B\u0F1FOT\u803B"\u4022r;\uC000\u{1D514}pf;\u611Acr;\uC000\u{1D4AC}\u0600BEacefhiorsu\u0F3E\u0F43\u0F47\u0F60\u0F73\u0FA7\u0FAA\u0FAD\u1096\u10A9\u10B4\u10BEarr;\u6910G\u803B\xAE\u40AE\u0180cnr\u0F4E\u0F53\u0F56ute;\u4154g;\u67EBr\u0100;t\u0F5C\u0F5D\u61A0l;\u6916\u0180aey\u0F67\u0F6C\u0F71ron;\u4158dil;\u4156;\u4420\u0100;v\u0F78\u0F79\u611Cerse\u0100EU\u0F82\u0F99\u0100lq\u0F87\u0F8Eement;\u620Builibrium;\u61CBpEquilibrium;\u696Fr\xBB\u0F79o;\u43A1ght\u0400ACDFTUVa\u0FC1\u0FEB\u0FF3\u1022\u1028\u105B\u1087\u03D8\u0100nr\u0FC6\u0FD2gleBracket;\u67E9row\u0180;BL\u0FDC\u0FDD\u0FE1\u6192ar;\u61E5eftArrow;\u61C4eiling;\u6309o\u01F5\u0FF9\0\u1005bleBracket;\u67E7n\u01D4\u100A\0\u1014eeVector;\u695Dector\u0100;B\u101D\u101E\u61C2ar;\u6955loor;\u630B\u0100er\u102D\u1043e\u0180;AV\u1035\u1036\u103C\u62A2rrow;\u61A6ector;\u695Biangle\u0180;BE\u1050\u1051\u1055\u62B3ar;\u69D0qual;\u62B5p\u0180DTV\u1063\u106E\u1078ownVector;\u694FeeVector;\u695Cector\u0100;B\u1082\u1083\u61BEar;\u6954ector\u0100;B\u1091\u1092\u61C0ar;\u6953\u0100pu\u109B\u109Ef;\u611DndImplies;\u6970ightarrow;\u61DB\u0100ch\u10B9\u10BCr;\u611B;\u61B1leDelayed;\u69F4\u0680HOacfhimoqstu\u10E4\u10F1\u10F7\u10FD\u1119\u111E\u1151\u1156\u1161\u1167\u11B5\u11BB\u11BF\u0100Cc\u10E9\u10EEHcy;\u4429y;\u4428FTcy;\u442Ccute;\u415A\u0280;aeiy\u1108\u1109\u110E\u1113\u1117\u6ABCron;\u4160dil;\u415Erc;\u415C;\u4421r;\uC000\u{1D516}ort\u0200DLRU\u112A\u1134\u113E\u1149ownArrow\xBB\u041EeftArrow\xBB\u089AightArrow\xBB\u0FDDpArrow;\u6191gma;\u43A3allCircle;\u6218pf;\uC000\u{1D54A}\u0272\u116D\0\0\u1170t;\u621Aare\u0200;ISU\u117B\u117C\u1189\u11AF\u65A1ntersection;\u6293u\u0100bp\u118F\u119Eset\u0100;E\u1197\u1198\u628Fqual;\u6291erset\u0100;E\u11A8\u11A9\u6290qual;\u6292nion;\u6294cr;\uC000\u{1D4AE}ar;\u62C6\u0200bcmp\u11C8\u11DB\u1209\u120B\u0100;s\u11CD\u11CE\u62D0et\u0100;E\u11CD\u11D5qual;\u6286\u0100ch\u11E0\u1205eeds\u0200;EST\u11ED\u11EE\u11F4\u11FF\u627Bqual;\u6AB0lantEqual;\u627Dilde;\u627FTh\xE1\u0F8C;\u6211\u0180;es\u1212\u1213\u1223\u62D1rset\u0100;E\u121C\u121D\u6283qual;\u6287et\xBB\u1213\u0580HRSacfhiors\u123E\u1244\u1249\u1255\u125E\u1271\u1276\u129F\u12C2\u12C8\u12D1ORN\u803B\xDE\u40DEADE;\u6122\u0100Hc\u124E\u1252cy;\u440By;\u4426\u0100bu\u125A\u125C;\u4009;\u43A4\u0180aey\u1265\u126A\u126Fron;\u4164dil;\u4162;\u4422r;\uC000\u{1D517}\u0100ei\u127B\u1289\u01F2\u1280\0\u1287efore;\u6234a;\u4398\u0100cn\u128E\u1298kSpace;\uC000\u205F\u200ASpace;\u6009lde\u0200;EFT\u12AB\u12AC\u12B2\u12BC\u623Cqual;\u6243ullEqual;\u6245ilde;\u6248pf;\uC000\u{1D54B}ipleDot;\u60DB\u0100ct\u12D6\u12DBr;\uC000\u{1D4AF}rok;\u4166\u0AE1\u12F7\u130E\u131A\u1326\0\u132C\u1331\0\0\0\0\0\u1338\u133D\u1377\u1385\0\u13FF\u1404\u140A\u1410\u0100cr\u12FB\u1301ute\u803B\xDA\u40DAr\u0100;o\u1307\u1308\u619Fcir;\u6949r\u01E3\u1313\0\u1316y;\u440Eve;\u416C\u0100iy\u131E\u1323rc\u803B\xDB\u40DB;\u4423blac;\u4170r;\uC000\u{1D518}rave\u803B\xD9\u40D9acr;\u416A\u0100di\u1341\u1369er\u0100BP\u1348\u135D\u0100ar\u134D\u1350r;\u405Fac\u0100ek\u1357\u1359;\u63DFet;\u63B5arenthesis;\u63DDon\u0100;P\u1370\u1371\u62C3lus;\u628E\u0100gp\u137B\u137Fon;\u4172f;\uC000\u{1D54C}\u0400ADETadps\u1395\u13AE\u13B8\u13C4\u03E8\u13D2\u13D7\u13F3rrow\u0180;BD\u1150\u13A0\u13A4ar;\u6912ownArrow;\u61C5ownArrow;\u6195quilibrium;\u696Eee\u0100;A\u13CB\u13CC\u62A5rrow;\u61A5own\xE1\u03F3er\u0100LR\u13DE\u13E8eftArrow;\u6196ightArrow;\u6197i\u0100;l\u13F9\u13FA\u43D2on;\u43A5ing;\u416Ecr;\uC000\u{1D4B0}ilde;\u4168ml\u803B\xDC\u40DC\u0480Dbcdefosv\u1427\u142C\u1430\u1433\u143E\u1485\u148A\u1490\u1496ash;\u62ABar;\u6AEBy;\u4412ash\u0100;l\u143B\u143C\u62A9;\u6AE6\u0100er\u1443\u1445;\u62C1\u0180bty\u144C\u1450\u147Aar;\u6016\u0100;i\u144F\u1455cal\u0200BLST\u1461\u1465\u146A\u1474ar;\u6223ine;\u407Ceparator;\u6758ilde;\u6240ThinSpace;\u600Ar;\uC000\u{1D519}pf;\uC000\u{1D54D}cr;\uC000\u{1D4B1}dash;\u62AA\u0280cefos\u14A7\u14AC\u14B1\u14B6\u14BCirc;\u4174dge;\u62C0r;\uC000\u{1D51A}pf;\uC000\u{1D54E}cr;\uC000\u{1D4B2}\u0200fios\u14CB\u14D0\u14D2\u14D8r;\uC000\u{1D51B};\u439Epf;\uC000\u{1D54F}cr;\uC000\u{1D4B3}\u0480AIUacfosu\u14F1\u14F5\u14F9\u14FD\u1504\u150F\u1514\u151A\u1520cy;\u442Fcy;\u4407cy;\u442Ecute\u803B\xDD\u40DD\u0100iy\u1509\u150Drc;\u4176;\u442Br;\uC000\u{1D51C}pf;\uC000\u{1D550}cr;\uC000\u{1D4B4}ml;\u4178\u0400Hacdefos\u1535\u1539\u153F\u154B\u154F\u155D\u1560\u1564cy;\u4416cute;\u4179\u0100ay\u1544\u1549ron;\u417D;\u4417ot;\u417B\u01F2\u1554\0\u155BoWidt\xE8\u0AD9a;\u4396r;\u6128pf;\u6124cr;\uC000\u{1D4B5}\u0BE1\u1583\u158A\u1590\0\u15B0\u15B6\u15BF\0\0\0\0\u15C6\u15DB\u15EB\u165F\u166D\0\u1695\u169B\u16B2\u16B9\0\u16BEcute\u803B\xE1\u40E1reve;\u4103\u0300;Ediuy\u159C\u159D\u15A1\u15A3\u15A8\u15AD\u623E;\uC000\u223E\u0333;\u623Frc\u803B\xE2\u40E2te\u80BB\xB4\u0306;\u4430lig\u803B\xE6\u40E6\u0100;r\xB2\u15BA;\uC000\u{1D51E}rave\u803B\xE0\u40E0\u0100ep\u15CA\u15D6\u0100fp\u15CF\u15D4sym;\u6135\xE8\u15D3ha;\u43B1\u0100ap\u15DFc\u0100cl\u15E4\u15E7r;\u4101g;\u6A3F\u0264\u15F0\0\0\u160A\u0280;adsv\u15FA\u15FB\u15FF\u1601\u1607\u6227nd;\u6A55;\u6A5Clope;\u6A58;\u6A5A\u0380;elmrsz\u1618\u1619\u161B\u161E\u163F\u164F\u1659\u6220;\u69A4e\xBB\u1619sd\u0100;a\u1625\u1626\u6221\u0461\u1630\u1632\u1634\u1636\u1638\u163A\u163C\u163E;\u69A8;\u69A9;\u69AA;\u69AB;\u69AC;\u69AD;\u69AE;\u69AFt\u0100;v\u1645\u1646\u621Fb\u0100;d\u164C\u164D\u62BE;\u699D\u0100pt\u1654\u1657h;\u6222\xBB\xB9arr;\u637C\u0100gp\u1663\u1667on;\u4105f;\uC000\u{1D552}\u0380;Eaeiop\u12C1\u167B\u167D\u1682\u1684\u1687\u168A;\u6A70cir;\u6A6F;\u624Ad;\u624Bs;\u4027rox\u0100;e\u12C1\u1692\xF1\u1683ing\u803B\xE5\u40E5\u0180cty\u16A1\u16A6\u16A8r;\uC000\u{1D4B6};\u402Amp\u0100;e\u12C1\u16AF\xF1\u0288ilde\u803B\xE3\u40E3ml\u803B\xE4\u40E4\u0100ci\u16C2\u16C8onin\xF4\u0272nt;\u6A11\u0800Nabcdefiklnoprsu\u16ED\u16F1\u1730\u173C\u1743\u1748\u1778\u177D\u17E0\u17E6\u1839\u1850\u170D\u193D\u1948\u1970ot;\u6AED\u0100cr\u16F6\u171Ek\u0200ceps\u1700\u1705\u170D\u1713ong;\u624Cpsilon;\u43F6rime;\u6035im\u0100;e\u171A\u171B\u623Dq;\u62CD\u0176\u1722\u1726ee;\u62BDed\u0100;g\u172C\u172D\u6305e\xBB\u172Drk\u0100;t\u135C\u1737brk;\u63B6\u0100oy\u1701\u1741;\u4431quo;\u601E\u0280cmprt\u1753\u175B\u1761\u1764\u1768aus\u0100;e\u010A\u0109ptyv;\u69B0s\xE9\u170Cno\xF5\u0113\u0180ahw\u176F\u1771\u1773;\u43B2;\u6136een;\u626Cr;\uC000\u{1D51F}g\u0380costuvw\u178D\u179D\u17B3\u17C1\u17D5\u17DB\u17DE\u0180aiu\u1794\u1796\u179A\xF0\u0760rc;\u65EFp\xBB\u1371\u0180dpt\u17A4\u17A8\u17ADot;\u6A00lus;\u6A01imes;\u6A02\u0271\u17B9\0\0\u17BEcup;\u6A06ar;\u6605riangle\u0100du\u17CD\u17D2own;\u65BDp;\u65B3plus;\u6A04e\xE5\u1444\xE5\u14ADarow;\u690D\u0180ako\u17ED\u1826\u1835\u0100cn\u17F2\u1823k\u0180lst\u17FA\u05AB\u1802ozenge;\u69EBriangle\u0200;dlr\u1812\u1813\u1818\u181D\u65B4own;\u65BEeft;\u65C2ight;\u65B8k;\u6423\u01B1\u182B\0\u1833\u01B2\u182F\0\u1831;\u6592;\u65914;\u6593ck;\u6588\u0100eo\u183E\u184D\u0100;q\u1843\u1846\uC000=\u20E5uiv;\uC000\u2261\u20E5t;\u6310\u0200ptwx\u1859\u185E\u1867\u186Cf;\uC000\u{1D553}\u0100;t\u13CB\u1863om\xBB\u13CCtie;\u62C8\u0600DHUVbdhmptuv\u1885\u1896\u18AA\u18BB\u18D7\u18DB\u18EC\u18FF\u1905\u190A\u1910\u1921\u0200LRlr\u188E\u1890\u1892\u1894;\u6557;\u6554;\u6556;\u6553\u0280;DUdu\u18A1\u18A2\u18A4\u18A6\u18A8\u6550;\u6566;\u6569;\u6564;\u6567\u0200LRlr\u18B3\u18B5\u18B7\u18B9;\u655D;\u655A;\u655C;\u6559\u0380;HLRhlr\u18CA\u18CB\u18CD\u18CF\u18D1\u18D3\u18D5\u6551;\u656C;\u6563;\u6560;\u656B;\u6562;\u655Fox;\u69C9\u0200LRlr\u18E4\u18E6\u18E8\u18EA;\u6555;\u6552;\u6510;\u650C\u0280;DUdu\u06BD\u18F7\u18F9\u18FB\u18FD;\u6565;\u6568;\u652C;\u6534inus;\u629Flus;\u629Eimes;\u62A0\u0200LRlr\u1919\u191B\u191D\u191F;\u655B;\u6558;\u6518;\u6514\u0380;HLRhlr\u1930\u1931\u1933\u1935\u1937\u1939\u193B\u6502;\u656A;\u6561;\u655E;\u653C;\u6524;\u651C\u0100ev\u0123\u1942bar\u803B\xA6\u40A6\u0200ceio\u1951\u1956\u195A\u1960r;\uC000\u{1D4B7}mi;\u604Fm\u0100;e\u171A\u171Cl\u0180;bh\u1968\u1969\u196B\u405C;\u69C5sub;\u67C8\u016C\u1974\u197El\u0100;e\u1979\u197A\u6022t\xBB\u197Ap\u0180;Ee\u012F\u1985\u1987;\u6AAE\u0100;q\u06DC\u06DB\u0CE1\u19A7\0\u19E8\u1A11\u1A15\u1A32\0\u1A37\u1A50\0\0\u1AB4\0\0\u1AC1\0\0\u1B21\u1B2E\u1B4D\u1B52\0\u1BFD\0\u1C0C\u0180cpr\u19AD\u19B2\u19DDute;\u4107\u0300;abcds\u19BF\u19C0\u19C4\u19CA\u19D5\u19D9\u6229nd;\u6A44rcup;\u6A49\u0100au\u19CF\u19D2p;\u6A4Bp;\u6A47ot;\u6A40;\uC000\u2229\uFE00\u0100eo\u19E2\u19E5t;\u6041\xEE\u0693\u0200aeiu\u19F0\u19FB\u1A01\u1A05\u01F0\u19F5\0\u19F8s;\u6A4Don;\u410Ddil\u803B\xE7\u40E7rc;\u4109ps\u0100;s\u1A0C\u1A0D\u6A4Cm;\u6A50ot;\u410B\u0180dmn\u1A1B\u1A20\u1A26il\u80BB\xB8\u01ADptyv;\u69B2t\u8100\xA2;e\u1A2D\u1A2E\u40A2r\xE4\u01B2r;\uC000\u{1D520}\u0180cei\u1A3D\u1A40\u1A4Dy;\u4447ck\u0100;m\u1A47\u1A48\u6713ark\xBB\u1A48;\u43C7r\u0380;Ecefms\u1A5F\u1A60\u1A62\u1A6B\u1AA4\u1AAA\u1AAE\u65CB;\u69C3\u0180;el\u1A69\u1A6A\u1A6D\u42C6q;\u6257e\u0261\u1A74\0\0\u1A88rrow\u0100lr\u1A7C\u1A81eft;\u61BAight;\u61BB\u0280RSacd\u1A92\u1A94\u1A96\u1A9A\u1A9F\xBB\u0F47;\u64C8st;\u629Birc;\u629Aash;\u629Dnint;\u6A10id;\u6AEFcir;\u69C2ubs\u0100;u\u1ABB\u1ABC\u6663it\xBB\u1ABC\u02EC\u1AC7\u1AD4\u1AFA\0\u1B0Aon\u0100;e\u1ACD\u1ACE\u403A\u0100;q\xC7\xC6\u026D\u1AD9\0\0\u1AE2a\u0100;t\u1ADE\u1ADF\u402C;\u4040\u0180;fl\u1AE8\u1AE9\u1AEB\u6201\xEE\u1160e\u0100mx\u1AF1\u1AF6ent\xBB\u1AE9e\xF3\u024D\u01E7\u1AFE\0\u1B07\u0100;d\u12BB\u1B02ot;\u6A6Dn\xF4\u0246\u0180fry\u1B10\u1B14\u1B17;\uC000\u{1D554}o\xE4\u0254\u8100\xA9;s\u0155\u1B1Dr;\u6117\u0100ao\u1B25\u1B29rr;\u61B5ss;\u6717\u0100cu\u1B32\u1B37r;\uC000\u{1D4B8}\u0100bp\u1B3C\u1B44\u0100;e\u1B41\u1B42\u6ACF;\u6AD1\u0100;e\u1B49\u1B4A\u6AD0;\u6AD2dot;\u62EF\u0380delprvw\u1B60\u1B6C\u1B77\u1B82\u1BAC\u1BD4\u1BF9arr\u0100lr\u1B68\u1B6A;\u6938;\u6935\u0270\u1B72\0\0\u1B75r;\u62DEc;\u62DFarr\u0100;p\u1B7F\u1B80\u61B6;\u693D\u0300;bcdos\u1B8F\u1B90\u1B96\u1BA1\u1BA5\u1BA8\u622Arcap;\u6A48\u0100au\u1B9B\u1B9Ep;\u6A46p;\u6A4Aot;\u628Dr;\u6A45;\uC000\u222A\uFE00\u0200alrv\u1BB5\u1BBF\u1BDE\u1BE3rr\u0100;m\u1BBC\u1BBD\u61B7;\u693Cy\u0180evw\u1BC7\u1BD4\u1BD8q\u0270\u1BCE\0\0\u1BD2re\xE3\u1B73u\xE3\u1B75ee;\u62CEedge;\u62CFen\u803B\xA4\u40A4earrow\u0100lr\u1BEE\u1BF3eft\xBB\u1B80ight\xBB\u1BBDe\xE4\u1BDD\u0100ci\u1C01\u1C07onin\xF4\u01F7nt;\u6231lcty;\u632D\u0980AHabcdefhijlorstuwz\u1C38\u1C3B\u1C3F\u1C5D\u1C69\u1C75\u1C8A\u1C9E\u1CAC\u1CB7\u1CFB\u1CFF\u1D0D\u1D7B\u1D91\u1DAB\u1DBB\u1DC6\u1DCDr\xF2\u0381ar;\u6965\u0200glrs\u1C48\u1C4D\u1C52\u1C54ger;\u6020eth;\u6138\xF2\u1133h\u0100;v\u1C5A\u1C5B\u6010\xBB\u090A\u016B\u1C61\u1C67arow;\u690Fa\xE3\u0315\u0100ay\u1C6E\u1C73ron;\u410F;\u4434\u0180;ao\u0332\u1C7C\u1C84\u0100gr\u02BF\u1C81r;\u61CAtseq;\u6A77\u0180glm\u1C91\u1C94\u1C98\u803B\xB0\u40B0ta;\u43B4ptyv;\u69B1\u0100ir\u1CA3\u1CA8sht;\u697F;\uC000\u{1D521}ar\u0100lr\u1CB3\u1CB5\xBB\u08DC\xBB\u101E\u0280aegsv\u1CC2\u0378\u1CD6\u1CDC\u1CE0m\u0180;os\u0326\u1CCA\u1CD4nd\u0100;s\u0326\u1CD1uit;\u6666amma;\u43DDin;\u62F2\u0180;io\u1CE7\u1CE8\u1CF8\u40F7de\u8100\xF7;o\u1CE7\u1CF0ntimes;\u62C7n\xF8\u1CF7cy;\u4452c\u026F\u1D06\0\0\u1D0Arn;\u631Eop;\u630D\u0280lptuw\u1D18\u1D1D\u1D22\u1D49\u1D55lar;\u4024f;\uC000\u{1D555}\u0280;emps\u030B\u1D2D\u1D37\u1D3D\u1D42q\u0100;d\u0352\u1D33ot;\u6251inus;\u6238lus;\u6214quare;\u62A1blebarwedg\xE5\xFAn\u0180adh\u112E\u1D5D\u1D67ownarrow\xF3\u1C83arpoon\u0100lr\u1D72\u1D76ef\xF4\u1CB4igh\xF4\u1CB6\u0162\u1D7F\u1D85karo\xF7\u0F42\u026F\u1D8A\0\0\u1D8Ern;\u631Fop;\u630C\u0180cot\u1D98\u1DA3\u1DA6\u0100ry\u1D9D\u1DA1;\uC000\u{1D4B9};\u4455l;\u69F6rok;\u4111\u0100dr\u1DB0\u1DB4ot;\u62F1i\u0100;f\u1DBA\u1816\u65BF\u0100ah\u1DC0\u1DC3r\xF2\u0429a\xF2\u0FA6angle;\u69A6\u0100ci\u1DD2\u1DD5y;\u445Fgrarr;\u67FF\u0900Dacdefglmnopqrstux\u1E01\u1E09\u1E19\u1E38\u0578\u1E3C\u1E49\u1E61\u1E7E\u1EA5\u1EAF\u1EBD\u1EE1\u1F2A\u1F37\u1F44\u1F4E\u1F5A\u0100Do\u1E06\u1D34o\xF4\u1C89\u0100cs\u1E0E\u1E14ute\u803B\xE9\u40E9ter;\u6A6E\u0200aioy\u1E22\u1E27\u1E31\u1E36ron;\u411Br\u0100;c\u1E2D\u1E2E\u6256\u803B\xEA\u40EAlon;\u6255;\u444Dot;\u4117\u0100Dr\u1E41\u1E45ot;\u6252;\uC000\u{1D522}\u0180;rs\u1E50\u1E51\u1E57\u6A9Aave\u803B\xE8\u40E8\u0100;d\u1E5C\u1E5D\u6A96ot;\u6A98\u0200;ils\u1E6A\u1E6B\u1E72\u1E74\u6A99nters;\u63E7;\u6113\u0100;d\u1E79\u1E7A\u6A95ot;\u6A97\u0180aps\u1E85\u1E89\u1E97cr;\u4113ty\u0180;sv\u1E92\u1E93\u1E95\u6205et\xBB\u1E93p\u01001;\u1E9D\u1EA4\u0133\u1EA1\u1EA3;\u6004;\u6005\u6003\u0100gs\u1EAA\u1EAC;\u414Bp;\u6002\u0100gp\u1EB4\u1EB8on;\u4119f;\uC000\u{1D556}\u0180als\u1EC4\u1ECE\u1ED2r\u0100;s\u1ECA\u1ECB\u62D5l;\u69E3us;\u6A71i\u0180;lv\u1EDA\u1EDB\u1EDF\u43B5on\xBB\u1EDB;\u43F5\u0200csuv\u1EEA\u1EF3\u1F0B\u1F23\u0100io\u1EEF\u1E31rc\xBB\u1E2E\u0269\u1EF9\0\0\u1EFB\xED\u0548ant\u0100gl\u1F02\u1F06tr\xBB\u1E5Dess\xBB\u1E7A\u0180aei\u1F12\u1F16\u1F1Als;\u403Dst;\u625Fv\u0100;D\u0235\u1F20D;\u6A78parsl;\u69E5\u0100Da\u1F2F\u1F33ot;\u6253rr;\u6971\u0180cdi\u1F3E\u1F41\u1EF8r;\u612Fo\xF4\u0352\u0100ah\u1F49\u1F4B;\u43B7\u803B\xF0\u40F0\u0100mr\u1F53\u1F57l\u803B\xEB\u40EBo;\u60AC\u0180cip\u1F61\u1F64\u1F67l;\u4021s\xF4\u056E\u0100eo\u1F6C\u1F74ctatio\xEE\u0559nential\xE5\u0579\u09E1\u1F92\0\u1F9E\0\u1FA1\u1FA7\0\0\u1FC6\u1FCC\0\u1FD3\0\u1FE6\u1FEA\u2000\0\u2008\u205Allingdotse\xF1\u1E44y;\u4444male;\u6640\u0180ilr\u1FAD\u1FB3\u1FC1lig;\u8000\uFB03\u0269\u1FB9\0\0\u1FBDg;\u8000\uFB00ig;\u8000\uFB04;\uC000\u{1D523}lig;\u8000\uFB01lig;\uC000fj\u0180alt\u1FD9\u1FDC\u1FE1t;\u666Dig;\u8000\uFB02ns;\u65B1of;\u4192\u01F0\u1FEE\0\u1FF3f;\uC000\u{1D557}\u0100ak\u05BF\u1FF7\u0100;v\u1FFC\u1FFD\u62D4;\u6AD9artint;\u6A0D\u0100ao\u200C\u2055\u0100cs\u2011\u2052\u03B1\u201A\u2030\u2038\u2045\u2048\0\u2050\u03B2\u2022\u2025\u2027\u202A\u202C\0\u202E\u803B\xBD\u40BD;\u6153\u803B\xBC\u40BC;\u6155;\u6159;\u615B\u01B3\u2034\0\u2036;\u6154;\u6156\u02B4\u203E\u2041\0\0\u2043\u803B\xBE\u40BE;\u6157;\u615C5;\u6158\u01B6\u204C\0\u204E;\u615A;\u615D8;\u615El;\u6044wn;\u6322cr;\uC000\u{1D4BB}\u0880Eabcdefgijlnorstv\u2082\u2089\u209F\u20A5\u20B0\u20B4\u20F0\u20F5\u20FA\u20FF\u2103\u2112\u2138\u0317\u213E\u2152\u219E\u0100;l\u064D\u2087;\u6A8C\u0180cmp\u2090\u2095\u209Dute;\u41F5ma\u0100;d\u209C\u1CDA\u43B3;\u6A86reve;\u411F\u0100iy\u20AA\u20AErc;\u411D;\u4433ot;\u4121\u0200;lqs\u063E\u0642\u20BD\u20C9\u0180;qs\u063E\u064C\u20C4lan\xF4\u0665\u0200;cdl\u0665\u20D2\u20D5\u20E5c;\u6AA9ot\u0100;o\u20DC\u20DD\u6A80\u0100;l\u20E2\u20E3\u6A82;\u6A84\u0100;e\u20EA\u20ED\uC000\u22DB\uFE00s;\u6A94r;\uC000\u{1D524}\u0100;g\u0673\u061Bmel;\u6137cy;\u4453\u0200;Eaj\u065A\u210C\u210E\u2110;\u6A92;\u6AA5;\u6AA4\u0200Eaes\u211B\u211D\u2129\u2134;\u6269p\u0100;p\u2123\u2124\u6A8Arox\xBB\u2124\u0100;q\u212E\u212F\u6A88\u0100;q\u212E\u211Bim;\u62E7pf;\uC000\u{1D558}\u0100ci\u2143\u2146r;\u610Am\u0180;el\u066B\u214E\u2150;\u6A8E;\u6A90\u8300>;cdlqr\u05EE\u2160\u216A\u216E\u2173\u2179\u0100ci\u2165\u2167;\u6AA7r;\u6A7Aot;\u62D7Par;\u6995uest;\u6A7C\u0280adels\u2184\u216A\u2190\u0656\u219B\u01F0\u2189\0\u218Epro\xF8\u209Er;\u6978q\u0100lq\u063F\u2196les\xF3\u2088i\xED\u066B\u0100en\u21A3\u21ADrtneqq;\uC000\u2269\uFE00\xC5\u21AA\u0500Aabcefkosy\u21C4\u21C7\u21F1\u21F5\u21FA\u2218\u221D\u222F\u2268\u227Dr\xF2\u03A0\u0200ilmr\u21D0\u21D4\u21D7\u21DBrs\xF0\u1484f\xBB\u2024il\xF4\u06A9\u0100dr\u21E0\u21E4cy;\u444A\u0180;cw\u08F4\u21EB\u21EFir;\u6948;\u61ADar;\u610Firc;\u4125\u0180alr\u2201\u220E\u2213rts\u0100;u\u2209\u220A\u6665it\xBB\u220Alip;\u6026con;\u62B9r;\uC000\u{1D525}s\u0100ew\u2223\u2229arow;\u6925arow;\u6926\u0280amopr\u223A\u223E\u2243\u225E\u2263rr;\u61FFtht;\u623Bk\u0100lr\u2249\u2253eftarrow;\u61A9ightarrow;\u61AAf;\uC000\u{1D559}bar;\u6015\u0180clt\u226F\u2274\u2278r;\uC000\u{1D4BD}as\xE8\u21F4rok;\u4127\u0100bp\u2282\u2287ull;\u6043hen\xBB\u1C5B\u0AE1\u22A3\0\u22AA\0\u22B8\u22C5\u22CE\0\u22D5\u22F3\0\0\u22F8\u2322\u2367\u2362\u237F\0\u2386\u23AA\u23B4cute\u803B\xED\u40ED\u0180;iy\u0771\u22B0\u22B5rc\u803B\xEE\u40EE;\u4438\u0100cx\u22BC\u22BFy;\u4435cl\u803B\xA1\u40A1\u0100fr\u039F\u22C9;\uC000\u{1D526}rave\u803B\xEC\u40EC\u0200;ino\u073E\u22DD\u22E9\u22EE\u0100in\u22E2\u22E6nt;\u6A0Ct;\u622Dfin;\u69DCta;\u6129lig;\u4133\u0180aop\u22FE\u231A\u231D\u0180cgt\u2305\u2308\u2317r;\u412B\u0180elp\u071F\u230F\u2313in\xE5\u078Ear\xF4\u0720h;\u4131f;\u62B7ed;\u41B5\u0280;cfot\u04F4\u232C\u2331\u233D\u2341are;\u6105in\u0100;t\u2338\u2339\u621Eie;\u69DDdo\xF4\u2319\u0280;celp\u0757\u234C\u2350\u235B\u2361al;\u62BA\u0100gr\u2355\u2359er\xF3\u1563\xE3\u234Darhk;\u6A17rod;\u6A3C\u0200cgpt\u236F\u2372\u2376\u237By;\u4451on;\u412Ff;\uC000\u{1D55A}a;\u43B9uest\u803B\xBF\u40BF\u0100ci\u238A\u238Fr;\uC000\u{1D4BE}n\u0280;Edsv\u04F4\u239B\u239D\u23A1\u04F3;\u62F9ot;\u62F5\u0100;v\u23A6\u23A7\u62F4;\u62F3\u0100;i\u0777\u23AElde;\u4129\u01EB\u23B8\0\u23BCcy;\u4456l\u803B\xEF\u40EF\u0300cfmosu\u23CC\u23D7\u23DC\u23E1\u23E7\u23F5\u0100iy\u23D1\u23D5rc;\u4135;\u4439r;\uC000\u{1D527}ath;\u4237pf;\uC000\u{1D55B}\u01E3\u23EC\0\u23F1r;\uC000\u{1D4BF}rcy;\u4458kcy;\u4454\u0400acfghjos\u240B\u2416\u2422\u2427\u242D\u2431\u2435\u243Bppa\u0100;v\u2413\u2414\u43BA;\u43F0\u0100ey\u241B\u2420dil;\u4137;\u443Ar;\uC000\u{1D528}reen;\u4138cy;\u4445cy;\u445Cpf;\uC000\u{1D55C}cr;\uC000\u{1D4C0}\u0B80ABEHabcdefghjlmnoprstuv\u2470\u2481\u2486\u248D\u2491\u250E\u253D\u255A\u2580\u264E\u265E\u2665\u2679\u267D\u269A\u26B2\u26D8\u275D\u2768\u278B\u27C0\u2801\u2812\u0180art\u2477\u247A\u247Cr\xF2\u09C6\xF2\u0395ail;\u691Barr;\u690E\u0100;g\u0994\u248B;\u6A8Bar;\u6962\u0963\u24A5\0\u24AA\0\u24B1\0\0\0\0\0\u24B5\u24BA\0\u24C6\u24C8\u24CD\0\u24F9ute;\u413Amptyv;\u69B4ra\xEE\u084Cbda;\u43BBg\u0180;dl\u088E\u24C1\u24C3;\u6991\xE5\u088E;\u6A85uo\u803B\xAB\u40ABr\u0400;bfhlpst\u0899\u24DE\u24E6\u24E9\u24EB\u24EE\u24F1\u24F5\u0100;f\u089D\u24E3s;\u691Fs;\u691D\xEB\u2252p;\u61ABl;\u6939im;\u6973l;\u61A2\u0180;ae\u24FF\u2500\u2504\u6AABil;\u6919\u0100;s\u2509\u250A\u6AAD;\uC000\u2AAD\uFE00\u0180abr\u2515\u2519\u251Drr;\u690Crk;\u6772\u0100ak\u2522\u252Cc\u0100ek\u2528\u252A;\u407B;\u405B\u0100es\u2531\u2533;\u698Bl\u0100du\u2539\u253B;\u698F;\u698D\u0200aeuy\u2546\u254B\u2556\u2558ron;\u413E\u0100di\u2550\u2554il;\u413C\xEC\u08B0\xE2\u2529;\u443B\u0200cqrs\u2563\u2566\u256D\u257Da;\u6936uo\u0100;r\u0E19\u1746\u0100du\u2572\u2577har;\u6967shar;\u694Bh;\u61B2\u0280;fgqs\u258B\u258C\u0989\u25F3\u25FF\u6264t\u0280ahlrt\u2598\u25A4\u25B7\u25C2\u25E8rrow\u0100;t\u0899\u25A1a\xE9\u24F6arpoon\u0100du\u25AF\u25B4own\xBB\u045Ap\xBB\u0966eftarrows;\u61C7ight\u0180ahs\u25CD\u25D6\u25DErrow\u0100;s\u08F4\u08A7arpoon\xF3\u0F98quigarro\xF7\u21F0hreetimes;\u62CB\u0180;qs\u258B\u0993\u25FAlan\xF4\u09AC\u0280;cdgs\u09AC\u260A\u260D\u261D\u2628c;\u6AA8ot\u0100;o\u2614\u2615\u6A7F\u0100;r\u261A\u261B\u6A81;\u6A83\u0100;e\u2622\u2625\uC000\u22DA\uFE00s;\u6A93\u0280adegs\u2633\u2639\u263D\u2649\u264Bppro\xF8\u24C6ot;\u62D6q\u0100gq\u2643\u2645\xF4\u0989gt\xF2\u248C\xF4\u099Bi\xED\u09B2\u0180ilr\u2655\u08E1\u265Asht;\u697C;\uC000\u{1D529}\u0100;E\u099C\u2663;\u6A91\u0161\u2669\u2676r\u0100du\u25B2\u266E\u0100;l\u0965\u2673;\u696Alk;\u6584cy;\u4459\u0280;acht\u0A48\u2688\u268B\u2691\u2696r\xF2\u25C1orne\xF2\u1D08ard;\u696Bri;\u65FA\u0100io\u269F\u26A4dot;\u4140ust\u0100;a\u26AC\u26AD\u63B0che\xBB\u26AD\u0200Eaes\u26BB\u26BD\u26C9\u26D4;\u6268p\u0100;p\u26C3\u26C4\u6A89rox\xBB\u26C4\u0100;q\u26CE\u26CF\u6A87\u0100;q\u26CE\u26BBim;\u62E6\u0400abnoptwz\u26E9\u26F4\u26F7\u271A\u272F\u2741\u2747\u2750\u0100nr\u26EE\u26F1g;\u67ECr;\u61FDr\xEB\u08C1g\u0180lmr\u26FF\u270D\u2714eft\u0100ar\u09E6\u2707ight\xE1\u09F2apsto;\u67FCight\xE1\u09FDparrow\u0100lr\u2725\u2729ef\xF4\u24EDight;\u61AC\u0180afl\u2736\u2739\u273Dr;\u6985;\uC000\u{1D55D}us;\u6A2Dimes;\u6A34\u0161\u274B\u274Fst;\u6217\xE1\u134E\u0180;ef\u2757\u2758\u1800\u65CAnge\xBB\u2758ar\u0100;l\u2764\u2765\u4028t;\u6993\u0280achmt\u2773\u2776\u277C\u2785\u2787r\xF2\u08A8orne\xF2\u1D8Car\u0100;d\u0F98\u2783;\u696D;\u600Eri;\u62BF\u0300achiqt\u2798\u279D\u0A40\u27A2\u27AE\u27BBquo;\u6039r;\uC000\u{1D4C1}m\u0180;eg\u09B2\u27AA\u27AC;\u6A8D;\u6A8F\u0100bu\u252A\u27B3o\u0100;r\u0E1F\u27B9;\u601Arok;\u4142\u8400<;cdhilqr\u082B\u27D2\u2639\u27DC\u27E0\u27E5\u27EA\u27F0\u0100ci\u27D7\u27D9;\u6AA6r;\u6A79re\xE5\u25F2mes;\u62C9arr;\u6976uest;\u6A7B\u0100Pi\u27F5\u27F9ar;\u6996\u0180;ef\u2800\u092D\u181B\u65C3r\u0100du\u2807\u280Dshar;\u694Ahar;\u6966\u0100en\u2817\u2821rtneqq;\uC000\u2268\uFE00\xC5\u281E\u0700Dacdefhilnopsu\u2840\u2845\u2882\u288E\u2893\u28A0\u28A5\u28A8\u28DA\u28E2\u28E4\u0A83\u28F3\u2902Dot;\u623A\u0200clpr\u284E\u2852\u2863\u287Dr\u803B\xAF\u40AF\u0100et\u2857\u2859;\u6642\u0100;e\u285E\u285F\u6720se\xBB\u285F\u0100;s\u103B\u2868to\u0200;dlu\u103B\u2873\u2877\u287Bow\xEE\u048Cef\xF4\u090F\xF0\u13D1ker;\u65AE\u0100oy\u2887\u288Cmma;\u6A29;\u443Cash;\u6014asuredangle\xBB\u1626r;\uC000\u{1D52A}o;\u6127\u0180cdn\u28AF\u28B4\u28C9ro\u803B\xB5\u40B5\u0200;acd\u1464\u28BD\u28C0\u28C4s\xF4\u16A7ir;\u6AF0ot\u80BB\xB7\u01B5us\u0180;bd\u28D2\u1903\u28D3\u6212\u0100;u\u1D3C\u28D8;\u6A2A\u0163\u28DE\u28E1p;\u6ADB\xF2\u2212\xF0\u0A81\u0100dp\u28E9\u28EEels;\u62A7f;\uC000\u{1D55E}\u0100ct\u28F8\u28FDr;\uC000\u{1D4C2}pos\xBB\u159D\u0180;lm\u2909\u290A\u290D\u43BCtimap;\u62B8\u0C00GLRVabcdefghijlmoprstuvw\u2942\u2953\u297E\u2989\u2998\u29DA\u29E9\u2A15\u2A1A\u2A58\u2A5D\u2A83\u2A95\u2AA4\u2AA8\u2B04\u2B07\u2B44\u2B7F\u2BAE\u2C34\u2C67\u2C7C\u2CE9\u0100gt\u2947\u294B;\uC000\u22D9\u0338\u0100;v\u2950\u0BCF\uC000\u226B\u20D2\u0180elt\u295A\u2972\u2976ft\u0100ar\u2961\u2967rrow;\u61CDightarrow;\u61CE;\uC000\u22D8\u0338\u0100;v\u297B\u0C47\uC000\u226A\u20D2ightarrow;\u61CF\u0100Dd\u298E\u2993ash;\u62AFash;\u62AE\u0280bcnpt\u29A3\u29A7\u29AC\u29B1\u29CCla\xBB\u02DEute;\u4144g;\uC000\u2220\u20D2\u0280;Eiop\u0D84\u29BC\u29C0\u29C5\u29C8;\uC000\u2A70\u0338d;\uC000\u224B\u0338s;\u4149ro\xF8\u0D84ur\u0100;a\u29D3\u29D4\u666El\u0100;s\u29D3\u0B38\u01F3\u29DF\0\u29E3p\u80BB\xA0\u0B37mp\u0100;e\u0BF9\u0C00\u0280aeouy\u29F4\u29FE\u2A03\u2A10\u2A13\u01F0\u29F9\0\u29FB;\u6A43on;\u4148dil;\u4146ng\u0100;d\u0D7E\u2A0Aot;\uC000\u2A6D\u0338p;\u6A42;\u443Dash;\u6013\u0380;Aadqsx\u0B92\u2A29\u2A2D\u2A3B\u2A41\u2A45\u2A50rr;\u61D7r\u0100hr\u2A33\u2A36k;\u6924\u0100;o\u13F2\u13F0ot;\uC000\u2250\u0338ui\xF6\u0B63\u0100ei\u2A4A\u2A4Ear;\u6928\xED\u0B98ist\u0100;s\u0BA0\u0B9Fr;\uC000\u{1D52B}\u0200Eest\u0BC5\u2A66\u2A79\u2A7C\u0180;qs\u0BBC\u2A6D\u0BE1\u0180;qs\u0BBC\u0BC5\u2A74lan\xF4\u0BE2i\xED\u0BEA\u0100;r\u0BB6\u2A81\xBB\u0BB7\u0180Aap\u2A8A\u2A8D\u2A91r\xF2\u2971rr;\u61AEar;\u6AF2\u0180;sv\u0F8D\u2A9C\u0F8C\u0100;d\u2AA1\u2AA2\u62FC;\u62FAcy;\u445A\u0380AEadest\u2AB7\u2ABA\u2ABE\u2AC2\u2AC5\u2AF6\u2AF9r\xF2\u2966;\uC000\u2266\u0338rr;\u619Ar;\u6025\u0200;fqs\u0C3B\u2ACE\u2AE3\u2AEFt\u0100ar\u2AD4\u2AD9rro\xF7\u2AC1ightarro\xF7\u2A90\u0180;qs\u0C3B\u2ABA\u2AEAlan\xF4\u0C55\u0100;s\u0C55\u2AF4\xBB\u0C36i\xED\u0C5D\u0100;r\u0C35\u2AFEi\u0100;e\u0C1A\u0C25i\xE4\u0D90\u0100pt\u2B0C\u2B11f;\uC000\u{1D55F}\u8180\xAC;in\u2B19\u2B1A\u2B36\u40ACn\u0200;Edv\u0B89\u2B24\u2B28\u2B2E;\uC000\u22F9\u0338ot;\uC000\u22F5\u0338\u01E1\u0B89\u2B33\u2B35;\u62F7;\u62F6i\u0100;v\u0CB8\u2B3C\u01E1\u0CB8\u2B41\u2B43;\u62FE;\u62FD\u0180aor\u2B4B\u2B63\u2B69r\u0200;ast\u0B7B\u2B55\u2B5A\u2B5Flle\xEC\u0B7Bl;\uC000\u2AFD\u20E5;\uC000\u2202\u0338lint;\u6A14\u0180;ce\u0C92\u2B70\u2B73u\xE5\u0CA5\u0100;c\u0C98\u2B78\u0100;e\u0C92\u2B7D\xF1\u0C98\u0200Aait\u2B88\u2B8B\u2B9D\u2BA7r\xF2\u2988rr\u0180;cw\u2B94\u2B95\u2B99\u619B;\uC000\u2933\u0338;\uC000\u219D\u0338ghtarrow\xBB\u2B95ri\u0100;e\u0CCB\u0CD6\u0380chimpqu\u2BBD\u2BCD\u2BD9\u2B04\u0B78\u2BE4\u2BEF\u0200;cer\u0D32\u2BC6\u0D37\u2BC9u\xE5\u0D45;\uC000\u{1D4C3}ort\u026D\u2B05\0\0\u2BD6ar\xE1\u2B56m\u0100;e\u0D6E\u2BDF\u0100;q\u0D74\u0D73su\u0100bp\u2BEB\u2BED\xE5\u0CF8\xE5\u0D0B\u0180bcp\u2BF6\u2C11\u2C19\u0200;Ees\u2BFF\u2C00\u0D22\u2C04\u6284;\uC000\u2AC5\u0338et\u0100;e\u0D1B\u2C0Bq\u0100;q\u0D23\u2C00c\u0100;e\u0D32\u2C17\xF1\u0D38\u0200;Ees\u2C22\u2C23\u0D5F\u2C27\u6285;\uC000\u2AC6\u0338et\u0100;e\u0D58\u2C2Eq\u0100;q\u0D60\u2C23\u0200gilr\u2C3D\u2C3F\u2C45\u2C47\xEC\u0BD7lde\u803B\xF1\u40F1\xE7\u0C43iangle\u0100lr\u2C52\u2C5Ceft\u0100;e\u0C1A\u2C5A\xF1\u0C26ight\u0100;e\u0CCB\u2C65\xF1\u0CD7\u0100;m\u2C6C\u2C6D\u43BD\u0180;es\u2C74\u2C75\u2C79\u4023ro;\u6116p;\u6007\u0480DHadgilrs\u2C8F\u2C94\u2C99\u2C9E\u2CA3\u2CB0\u2CB6\u2CD3\u2CE3ash;\u62ADarr;\u6904p;\uC000\u224D\u20D2ash;\u62AC\u0100et\u2CA8\u2CAC;\uC000\u2265\u20D2;\uC000>\u20D2nfin;\u69DE\u0180Aet\u2CBD\u2CC1\u2CC5rr;\u6902;\uC000\u2264\u20D2\u0100;r\u2CCA\u2CCD\uC000<\u20D2ie;\uC000\u22B4\u20D2\u0100At\u2CD8\u2CDCrr;\u6903rie;\uC000\u22B5\u20D2im;\uC000\u223C\u20D2\u0180Aan\u2CF0\u2CF4\u2D02rr;\u61D6r\u0100hr\u2CFA\u2CFDk;\u6923\u0100;o\u13E7\u13E5ear;\u6927\u1253\u1A95\0\0\0\0\0\0\0\0\0\0\0\0\0\u2D2D\0\u2D38\u2D48\u2D60\u2D65\u2D72\u2D84\u1B07\0\0\u2D8D\u2DAB\0\u2DC8\u2DCE\0\u2DDC\u2E19\u2E2B\u2E3E\u2E43\u0100cs\u2D31\u1A97ute\u803B\xF3\u40F3\u0100iy\u2D3C\u2D45r\u0100;c\u1A9E\u2D42\u803B\xF4\u40F4;\u443E\u0280abios\u1AA0\u2D52\u2D57\u01C8\u2D5Alac;\u4151v;\u6A38old;\u69BClig;\u4153\u0100cr\u2D69\u2D6Dir;\u69BF;\uC000\u{1D52C}\u036F\u2D79\0\0\u2D7C\0\u2D82n;\u42DBave\u803B\xF2\u40F2;\u69C1\u0100bm\u2D88\u0DF4ar;\u69B5\u0200acit\u2D95\u2D98\u2DA5\u2DA8r\xF2\u1A80\u0100ir\u2D9D\u2DA0r;\u69BEoss;\u69BBn\xE5\u0E52;\u69C0\u0180aei\u2DB1\u2DB5\u2DB9cr;\u414Dga;\u43C9\u0180cdn\u2DC0\u2DC5\u01CDron;\u43BF;\u69B6pf;\uC000\u{1D560}\u0180ael\u2DD4\u2DD7\u01D2r;\u69B7rp;\u69B9\u0380;adiosv\u2DEA\u2DEB\u2DEE\u2E08\u2E0D\u2E10\u2E16\u6228r\xF2\u1A86\u0200;efm\u2DF7\u2DF8\u2E02\u2E05\u6A5Dr\u0100;o\u2DFE\u2DFF\u6134f\xBB\u2DFF\u803B\xAA\u40AA\u803B\xBA\u40BAgof;\u62B6r;\u6A56lope;\u6A57;\u6A5B\u0180clo\u2E1F\u2E21\u2E27\xF2\u2E01ash\u803B\xF8\u40F8l;\u6298i\u016C\u2E2F\u2E34de\u803B\xF5\u40F5es\u0100;a\u01DB\u2E3As;\u6A36ml\u803B\xF6\u40F6bar;\u633D\u0AE1\u2E5E\0\u2E7D\0\u2E80\u2E9D\0\u2EA2\u2EB9\0\0\u2ECB\u0E9C\0\u2F13\0\0\u2F2B\u2FBC\0\u2FC8r\u0200;ast\u0403\u2E67\u2E72\u0E85\u8100\xB6;l\u2E6D\u2E6E\u40B6le\xEC\u0403\u0269\u2E78\0\0\u2E7Bm;\u6AF3;\u6AFDy;\u443Fr\u0280cimpt\u2E8B\u2E8F\u2E93\u1865\u2E97nt;\u4025od;\u402Eil;\u6030enk;\u6031r;\uC000\u{1D52D}\u0180imo\u2EA8\u2EB0\u2EB4\u0100;v\u2EAD\u2EAE\u43C6;\u43D5ma\xF4\u0A76ne;\u660E\u0180;tv\u2EBF\u2EC0\u2EC8\u43C0chfork\xBB\u1FFD;\u43D6\u0100au\u2ECF\u2EDFn\u0100ck\u2ED5\u2EDDk\u0100;h\u21F4\u2EDB;\u610E\xF6\u21F4s\u0480;abcdemst\u2EF3\u2EF4\u1908\u2EF9\u2EFD\u2F04\u2F06\u2F0A\u2F0E\u402Bcir;\u6A23ir;\u6A22\u0100ou\u1D40\u2F02;\u6A25;\u6A72n\u80BB\xB1\u0E9Dim;\u6A26wo;\u6A27\u0180ipu\u2F19\u2F20\u2F25ntint;\u6A15f;\uC000\u{1D561}nd\u803B\xA3\u40A3\u0500;Eaceinosu\u0EC8\u2F3F\u2F41\u2F44\u2F47\u2F81\u2F89\u2F92\u2F7E\u2FB6;\u6AB3p;\u6AB7u\xE5\u0ED9\u0100;c\u0ECE\u2F4C\u0300;acens\u0EC8\u2F59\u2F5F\u2F66\u2F68\u2F7Eppro\xF8\u2F43urlye\xF1\u0ED9\xF1\u0ECE\u0180aes\u2F6F\u2F76\u2F7Approx;\u6AB9qq;\u6AB5im;\u62E8i\xED\u0EDFme\u0100;s\u2F88\u0EAE\u6032\u0180Eas\u2F78\u2F90\u2F7A\xF0\u2F75\u0180dfp\u0EEC\u2F99\u2FAF\u0180als\u2FA0\u2FA5\u2FAAlar;\u632Eine;\u6312urf;\u6313\u0100;t\u0EFB\u2FB4\xEF\u0EFBrel;\u62B0\u0100ci\u2FC0\u2FC5r;\uC000\u{1D4C5};\u43C8ncsp;\u6008\u0300fiopsu\u2FDA\u22E2\u2FDF\u2FE5\u2FEB\u2FF1r;\uC000\u{1D52E}pf;\uC000\u{1D562}rime;\u6057cr;\uC000\u{1D4C6}\u0180aeo\u2FF8\u3009\u3013t\u0100ei\u2FFE\u3005rnion\xF3\u06B0nt;\u6A16st\u0100;e\u3010\u3011\u403F\xF1\u1F19\xF4\u0F14\u0A80ABHabcdefhilmnoprstux\u3040\u3051\u3055\u3059\u30E0\u310E\u312B\u3147\u3162\u3172\u318E\u3206\u3215\u3224\u3229\u3258\u326E\u3272\u3290\u32B0\u32B7\u0180art\u3047\u304A\u304Cr\xF2\u10B3\xF2\u03DDail;\u691Car\xF2\u1C65ar;\u6964\u0380cdenqrt\u3068\u3075\u3078\u307F\u308F\u3094\u30CC\u0100eu\u306D\u3071;\uC000\u223D\u0331te;\u4155i\xE3\u116Emptyv;\u69B3g\u0200;del\u0FD1\u3089\u308B\u308D;\u6992;\u69A5\xE5\u0FD1uo\u803B\xBB\u40BBr\u0580;abcfhlpstw\u0FDC\u30AC\u30AF\u30B7\u30B9\u30BC\u30BE\u30C0\u30C3\u30C7\u30CAp;\u6975\u0100;f\u0FE0\u30B4s;\u6920;\u6933s;\u691E\xEB\u225D\xF0\u272El;\u6945im;\u6974l;\u61A3;\u619D\u0100ai\u30D1\u30D5il;\u691Ao\u0100;n\u30DB\u30DC\u6236al\xF3\u0F1E\u0180abr\u30E7\u30EA\u30EEr\xF2\u17E5rk;\u6773\u0100ak\u30F3\u30FDc\u0100ek\u30F9\u30FB;\u407D;\u405D\u0100es\u3102\u3104;\u698Cl\u0100du\u310A\u310C;\u698E;\u6990\u0200aeuy\u3117\u311C\u3127\u3129ron;\u4159\u0100di\u3121\u3125il;\u4157\xEC\u0FF2\xE2\u30FA;\u4440\u0200clqs\u3134\u3137\u313D\u3144a;\u6937dhar;\u6969uo\u0100;r\u020E\u020Dh;\u61B3\u0180acg\u314E\u315F\u0F44l\u0200;ips\u0F78\u3158\u315B\u109Cn\xE5\u10BBar\xF4\u0FA9t;\u65AD\u0180ilr\u3169\u1023\u316Esht;\u697D;\uC000\u{1D52F}\u0100ao\u3177\u3186r\u0100du\u317D\u317F\xBB\u047B\u0100;l\u1091\u3184;\u696C\u0100;v\u318B\u318C\u43C1;\u43F1\u0180gns\u3195\u31F9\u31FCht\u0300ahlrst\u31A4\u31B0\u31C2\u31D8\u31E4\u31EErrow\u0100;t\u0FDC\u31ADa\xE9\u30C8arpoon\u0100du\u31BB\u31BFow\xEE\u317Ep\xBB\u1092eft\u0100ah\u31CA\u31D0rrow\xF3\u0FEAarpoon\xF3\u0551ightarrows;\u61C9quigarro\xF7\u30CBhreetimes;\u62CCg;\u42DAingdotse\xF1\u1F32\u0180ahm\u320D\u3210\u3213r\xF2\u0FEAa\xF2\u0551;\u600Foust\u0100;a\u321E\u321F\u63B1che\xBB\u321Fmid;\u6AEE\u0200abpt\u3232\u323D\u3240\u3252\u0100nr\u3237\u323Ag;\u67EDr;\u61FEr\xEB\u1003\u0180afl\u3247\u324A\u324Er;\u6986;\uC000\u{1D563}us;\u6A2Eimes;\u6A35\u0100ap\u325D\u3267r\u0100;g\u3263\u3264\u4029t;\u6994olint;\u6A12ar\xF2\u31E3\u0200achq\u327B\u3280\u10BC\u3285quo;\u603Ar;\uC000\u{1D4C7}\u0100bu\u30FB\u328Ao\u0100;r\u0214\u0213\u0180hir\u3297\u329B\u32A0re\xE5\u31F8mes;\u62CAi\u0200;efl\u32AA\u1059\u1821\u32AB\u65B9tri;\u69CEluhar;\u6968;\u611E\u0D61\u32D5\u32DB\u32DF\u332C\u3338\u3371\0\u337A\u33A4\0\0\u33EC\u33F0\0\u3428\u3448\u345A\u34AD\u34B1\u34CA\u34F1\0\u3616\0\0\u3633cute;\u415Bqu\xEF\u27BA\u0500;Eaceinpsy\u11ED\u32F3\u32F5\u32FF\u3302\u330B\u330F\u331F\u3326\u3329;\u6AB4\u01F0\u32FA\0\u32FC;\u6AB8on;\u4161u\xE5\u11FE\u0100;d\u11F3\u3307il;\u415Frc;\u415D\u0180Eas\u3316\u3318\u331B;\u6AB6p;\u6ABAim;\u62E9olint;\u6A13i\xED\u1204;\u4441ot\u0180;be\u3334\u1D47\u3335\u62C5;\u6A66\u0380Aacmstx\u3346\u334A\u3357\u335B\u335E\u3363\u336Drr;\u61D8r\u0100hr\u3350\u3352\xEB\u2228\u0100;o\u0A36\u0A34t\u803B\xA7\u40A7i;\u403Bwar;\u6929m\u0100in\u3369\xF0nu\xF3\xF1t;\u6736r\u0100;o\u3376\u2055\uC000\u{1D530}\u0200acoy\u3382\u3386\u3391\u33A0rp;\u666F\u0100hy\u338B\u338Fcy;\u4449;\u4448rt\u026D\u3399\0\0\u339Ci\xE4\u1464ara\xEC\u2E6F\u803B\xAD\u40AD\u0100gm\u33A8\u33B4ma\u0180;fv\u33B1\u33B2\u33B2\u43C3;\u43C2\u0400;deglnpr\u12AB\u33C5\u33C9\u33CE\u33D6\u33DE\u33E1\u33E6ot;\u6A6A\u0100;q\u12B1\u12B0\u0100;E\u33D3\u33D4\u6A9E;\u6AA0\u0100;E\u33DB\u33DC\u6A9D;\u6A9Fe;\u6246lus;\u6A24arr;\u6972ar\xF2\u113D\u0200aeit\u33F8\u3408\u340F\u3417\u0100ls\u33FD\u3404lsetm\xE9\u336Ahp;\u6A33parsl;\u69E4\u0100dl\u1463\u3414e;\u6323\u0100;e\u341C\u341D\u6AAA\u0100;s\u3422\u3423\u6AAC;\uC000\u2AAC\uFE00\u0180flp\u342E\u3433\u3442tcy;\u444C\u0100;b\u3438\u3439\u402F\u0100;a\u343E\u343F\u69C4r;\u633Ff;\uC000\u{1D564}a\u0100dr\u344D\u0402es\u0100;u\u3454\u3455\u6660it\xBB\u3455\u0180csu\u3460\u3479\u349F\u0100au\u3465\u346Fp\u0100;s\u1188\u346B;\uC000\u2293\uFE00p\u0100;s\u11B4\u3475;\uC000\u2294\uFE00u\u0100bp\u347F\u348F\u0180;es\u1197\u119C\u3486et\u0100;e\u1197\u348D\xF1\u119D\u0180;es\u11A8\u11AD\u3496et\u0100;e\u11A8\u349D\xF1\u11AE\u0180;af\u117B\u34A6\u05B0r\u0165\u34AB\u05B1\xBB\u117Car\xF2\u1148\u0200cemt\u34B9\u34BE\u34C2\u34C5r;\uC000\u{1D4C8}tm\xEE\xF1i\xEC\u3415ar\xE6\u11BE\u0100ar\u34CE\u34D5r\u0100;f\u34D4\u17BF\u6606\u0100an\u34DA\u34EDight\u0100ep\u34E3\u34EApsilo\xEE\u1EE0h\xE9\u2EAFs\xBB\u2852\u0280bcmnp\u34FB\u355E\u1209\u358B\u358E\u0480;Edemnprs\u350E\u350F\u3511\u3515\u351E\u3523\u352C\u3531\u3536\u6282;\u6AC5ot;\u6ABD\u0100;d\u11DA\u351Aot;\u6AC3ult;\u6AC1\u0100Ee\u3528\u352A;\u6ACB;\u628Alus;\u6ABFarr;\u6979\u0180eiu\u353D\u3552\u3555t\u0180;en\u350E\u3545\u354Bq\u0100;q\u11DA\u350Feq\u0100;q\u352B\u3528m;\u6AC7\u0100bp\u355A\u355C;\u6AD5;\u6AD3c\u0300;acens\u11ED\u356C\u3572\u3579\u357B\u3326ppro\xF8\u32FAurlye\xF1\u11FE\xF1\u11F3\u0180aes\u3582\u3588\u331Bppro\xF8\u331Aq\xF1\u3317g;\u666A\u0680123;Edehlmnps\u35A9\u35AC\u35AF\u121C\u35B2\u35B4\u35C0\u35C9\u35D5\u35DA\u35DF\u35E8\u35ED\u803B\xB9\u40B9\u803B\xB2\u40B2\u803B\xB3\u40B3;\u6AC6\u0100os\u35B9\u35BCt;\u6ABEub;\u6AD8\u0100;d\u1222\u35C5ot;\u6AC4s\u0100ou\u35CF\u35D2l;\u67C9b;\u6AD7arr;\u697Bult;\u6AC2\u0100Ee\u35E4\u35E6;\u6ACC;\u628Blus;\u6AC0\u0180eiu\u35F4\u3609\u360Ct\u0180;en\u121C\u35FC\u3602q\u0100;q\u1222\u35B2eq\u0100;q\u35E7\u35E4m;\u6AC8\u0100bp\u3611\u3613;\u6AD4;\u6AD6\u0180Aan\u361C\u3620\u362Drr;\u61D9r\u0100hr\u3626\u3628\xEB\u222E\u0100;o\u0A2B\u0A29war;\u692Alig\u803B\xDF\u40DF\u0BE1\u3651\u365D\u3660\u12CE\u3673\u3679\0\u367E\u36C2\0\0\0\0\0\u36DB\u3703\0\u3709\u376C\0\0\0\u3787\u0272\u3656\0\0\u365Bget;\u6316;\u43C4r\xEB\u0E5F\u0180aey\u3666\u366B\u3670ron;\u4165dil;\u4163;\u4442lrec;\u6315r;\uC000\u{1D531}\u0200eiko\u3686\u369D\u36B5\u36BC\u01F2\u368B\0\u3691e\u01004f\u1284\u1281a\u0180;sv\u3698\u3699\u369B\u43B8ym;\u43D1\u0100cn\u36A2\u36B2k\u0100as\u36A8\u36AEppro\xF8\u12C1im\xBB\u12ACs\xF0\u129E\u0100as\u36BA\u36AE\xF0\u12C1rn\u803B\xFE\u40FE\u01EC\u031F\u36C6\u22E7es\u8180\xD7;bd\u36CF\u36D0\u36D8\u40D7\u0100;a\u190F\u36D5r;\u6A31;\u6A30\u0180eps\u36E1\u36E3\u3700\xE1\u2A4D\u0200;bcf\u0486\u36EC\u36F0\u36F4ot;\u6336ir;\u6AF1\u0100;o\u36F9\u36FC\uC000\u{1D565}rk;\u6ADA\xE1\u3362rime;\u6034\u0180aip\u370F\u3712\u3764d\xE5\u1248\u0380adempst\u3721\u374D\u3740\u3751\u3757\u375C\u375Fngle\u0280;dlqr\u3730\u3731\u3736\u3740\u3742\u65B5own\xBB\u1DBBeft\u0100;e\u2800\u373E\xF1\u092E;\u625Cight\u0100;e\u32AA\u374B\xF1\u105Aot;\u65ECinus;\u6A3Alus;\u6A39b;\u69CDime;\u6A3Bezium;\u63E2\u0180cht\u3772\u377D\u3781\u0100ry\u3777\u377B;\uC000\u{1D4C9};\u4446cy;\u445Brok;\u4167\u0100io\u378B\u378Ex\xF4\u1777head\u0100lr\u3797\u37A0eftarro\xF7\u084Fightarrow\xBB\u0F5D\u0900AHabcdfghlmoprstuw\u37D0\u37D3\u37D7\u37E4\u37F0\u37FC\u380E\u381C\u3823\u3834\u3851\u385D\u386B\u38A9\u38CC\u38D2\u38EA\u38F6r\xF2\u03EDar;\u6963\u0100cr\u37DC\u37E2ute\u803B\xFA\u40FA\xF2\u1150r\u01E3\u37EA\0\u37EDy;\u445Eve;\u416D\u0100iy\u37F5\u37FArc\u803B\xFB\u40FB;\u4443\u0180abh\u3803\u3806\u380Br\xF2\u13ADlac;\u4171a\xF2\u13C3\u0100ir\u3813\u3818sht;\u697E;\uC000\u{1D532}rave\u803B\xF9\u40F9\u0161\u3827\u3831r\u0100lr\u382C\u382E\xBB\u0957\xBB\u1083lk;\u6580\u0100ct\u3839\u384D\u026F\u383F\0\0\u384Arn\u0100;e\u3845\u3846\u631Cr\xBB\u3846op;\u630Fri;\u65F8\u0100al\u3856\u385Acr;\u416B\u80BB\xA8\u0349\u0100gp\u3862\u3866on;\u4173f;\uC000\u{1D566}\u0300adhlsu\u114B\u3878\u387D\u1372\u3891\u38A0own\xE1\u13B3arpoon\u0100lr\u3888\u388Cef\xF4\u382Digh\xF4\u382Fi\u0180;hl\u3899\u389A\u389C\u43C5\xBB\u13FAon\xBB\u389Aparrows;\u61C8\u0180cit\u38B0\u38C4\u38C8\u026F\u38B6\0\0\u38C1rn\u0100;e\u38BC\u38BD\u631Dr\xBB\u38BDop;\u630Eng;\u416Fri;\u65F9cr;\uC000\u{1D4CA}\u0180dir\u38D9\u38DD\u38E2ot;\u62F0lde;\u4169i\u0100;f\u3730\u38E8\xBB\u1813\u0100am\u38EF\u38F2r\xF2\u38A8l\u803B\xFC\u40FCangle;\u69A7\u0780ABDacdeflnoprsz\u391C\u391F\u3929\u392D\u39B5\u39B8\u39BD\u39DF\u39E4\u39E8\u39F3\u39F9\u39FD\u3A01\u3A20r\xF2\u03F7ar\u0100;v\u3926\u3927\u6AE8;\u6AE9as\xE8\u03E1\u0100nr\u3932\u3937grt;\u699C\u0380eknprst\u34E3\u3946\u394B\u3952\u395D\u3964\u3996app\xE1\u2415othin\xE7\u1E96\u0180hir\u34EB\u2EC8\u3959op\xF4\u2FB5\u0100;h\u13B7\u3962\xEF\u318D\u0100iu\u3969\u396Dgm\xE1\u33B3\u0100bp\u3972\u3984setneq\u0100;q\u397D\u3980\uC000\u228A\uFE00;\uC000\u2ACB\uFE00setneq\u0100;q\u398F\u3992\uC000\u228B\uFE00;\uC000\u2ACC\uFE00\u0100hr\u399B\u399Fet\xE1\u369Ciangle\u0100lr\u39AA\u39AFeft\xBB\u0925ight\xBB\u1051y;\u4432ash\xBB\u1036\u0180elr\u39C4\u39D2\u39D7\u0180;be\u2DEA\u39CB\u39CFar;\u62BBq;\u625Alip;\u62EE\u0100bt\u39DC\u1468a\xF2\u1469r;\uC000\u{1D533}tr\xE9\u39AEsu\u0100bp\u39EF\u39F1\xBB\u0D1C\xBB\u0D59pf;\uC000\u{1D567}ro\xF0\u0EFBtr\xE9\u39B4\u0100cu\u3A06\u3A0Br;\uC000\u{1D4CB}\u0100bp\u3A10\u3A18n\u0100Ee\u3980\u3A16\xBB\u397En\u0100Ee\u3992\u3A1E\xBB\u3990igzag;\u699A\u0380cefoprs\u3A36\u3A3B\u3A56\u3A5B\u3A54\u3A61\u3A6Airc;\u4175\u0100di\u3A40\u3A51\u0100bg\u3A45\u3A49ar;\u6A5Fe\u0100;q\u15FA\u3A4F;\u6259erp;\u6118r;\uC000\u{1D534}pf;\uC000\u{1D568}\u0100;e\u1479\u3A66at\xE8\u1479cr;\uC000\u{1D4CC}\u0AE3\u178E\u3A87\0\u3A8B\0\u3A90\u3A9B\0\0\u3A9D\u3AA8\u3AAB\u3AAF\0\0\u3AC3\u3ACE\0\u3AD8\u17DC\u17DFtr\xE9\u17D1r;\uC000\u{1D535}\u0100Aa\u3A94\u3A97r\xF2\u03C3r\xF2\u09F6;\u43BE\u0100Aa\u3AA1\u3AA4r\xF2\u03B8r\xF2\u09EBa\xF0\u2713is;\u62FB\u0180dpt\u17A4\u3AB5\u3ABE\u0100fl\u3ABA\u17A9;\uC000\u{1D569}im\xE5\u17B2\u0100Aa\u3AC7\u3ACAr\xF2\u03CEr\xF2\u0A01\u0100cq\u3AD2\u17B8r;\uC000\u{1D4CD}\u0100pt\u17D6\u3ADCr\xE9\u17D4\u0400acefiosu\u3AF0\u3AFD\u3B08\u3B0C\u3B11\u3B15\u3B1B\u3B21c\u0100uy\u3AF6\u3AFBte\u803B\xFD\u40FD;\u444F\u0100iy\u3B02\u3B06rc;\u4177;\u444Bn\u803B\xA5\u40A5r;\uC000\u{1D536}cy;\u4457pf;\uC000\u{1D56A}cr;\uC000\u{1D4CE}\u0100cm\u3B26\u3B29y;\u444El\u803B\xFF\u40FF\u0500acdefhiosw\u3B42\u3B48\u3B54\u3B58\u3B64\u3B69\u3B6D\u3B74\u3B7A\u3B80cute;\u417A\u0100ay\u3B4D\u3B52ron;\u417E;\u4437ot;\u417C\u0100et\u3B5D\u3B61tr\xE6\u155Fa;\u43B6r;\uC000\u{1D537}cy;\u4436grarr;\u61DDpf;\uC000\u{1D56B}cr;\uC000\u{1D4CF}\u0100jn\u3B85\u3B87;\u600Dj;\u600C'.split("").map(e => e.charCodeAt(0))),
    ju = new Uint16Array("\u0200aglq	\x1B\u026D\0\0p;\u4026os;\u4027t;\u403Et;\u403Cuot;\u4022".split("").map(e => e.charCodeAt(0))),
    Fn,
    Zu = new Map([[0, 65533], [128, 8364], [130, 8218], [131, 402], [132, 8222], [133, 8230], [134, 8224], [135, 8225], [136, 710], [137, 8240], [138, 352], [139, 8249], [140, 338], [142, 381], [145, 8216], [146, 8217], [147, 8220], [148, 8221], [149, 8226], [150, 8211], [151, 8212], [152, 732], [153, 8482], [154, 353], [155, 8250], [156, 339], [158, 382], [159, 376]]),
    zu = (Fn = String.fromCodePoint) !== null && Fn !== void 0 ? Fn : function (e) {
      let t = "";
      return e > 65535 && (e -= 65536, t += String.fromCharCode(e >>> 10 & 1023 | 55296), e = 56320 | e & 1023), t += String.fromCharCode(e), t;
    };
  function $u(e) {
    var t;
    return e >= 55296 && e <= 57343 || e > 1114111 ? 65533 : (t = Zu.get(e)) !== null && t !== void 0 ? t : e;
  }
  var V;
  (function (e) {
    e[e.NUM = 35] = "NUM", e[e.SEMI = 59] = "SEMI", e[e.EQUALS = 61] = "EQUALS", e[e.ZERO = 48] = "ZERO", e[e.NINE = 57] = "NINE", e[e.LOWER_A = 97] = "LOWER_A", e[e.LOWER_F = 102] = "LOWER_F", e[e.LOWER_X = 120] = "LOWER_X", e[e.LOWER_Z = 122] = "LOWER_Z", e[e.UPPER_A = 65] = "UPPER_A", e[e.UPPER_F = 70] = "UPPER_F", e[e.UPPER_Z = 90] = "UPPER_Z";
  })(V || (V = {}));
  var ea = 32,
    Le;
  (function (e) {
    e[e.VALUE_LENGTH = 49152] = "VALUE_LENGTH", e[e.BRANCH_LENGTH = 16256] = "BRANCH_LENGTH", e[e.JUMP_TABLE = 127] = "JUMP_TABLE";
  })(Le || (Le = {}));
  function _n(e) {
    return e >= V.ZERO && e <= V.NINE;
  }
  function ta(e) {
    return e >= V.UPPER_A && e <= V.UPPER_F || e >= V.LOWER_A && e <= V.LOWER_F;
  }
  function na(e) {
    return e >= V.UPPER_A && e <= V.UPPER_Z || e >= V.LOWER_A && e <= V.LOWER_Z || _n(e);
  }
  function ra(e) {
    return e === V.EQUALS || na(e);
  }
  var j;
  (function (e) {
    e[e.EntityStart = 0] = "EntityStart", e[e.NumericStart = 1] = "NumericStart", e[e.NumericDecimal = 2] = "NumericDecimal", e[e.NumericHex = 3] = "NumericHex", e[e.NamedEntity = 4] = "NamedEntity";
  })(j || (j = {}));
  var je;
  (function (e) {
    e[e.Legacy = 0] = "Legacy", e[e.Strict = 1] = "Strict", e[e.Attribute = 2] = "Attribute";
  })(je || (je = {}));
  var ua = class {
    constructor(e, t, n) {
      this.decodeTree = e, this.emitCodePoint = t, this.errors = n, this.state = j.EntityStart, this.consumed = 1, this.result = 0, this.treeIndex = 0, this.excess = 1, this.decodeMode = je.Strict;
    }
    startEntity(e) {
      this.decodeMode = e, this.state = j.EntityStart, this.result = 0, this.treeIndex = 0, this.excess = 1, this.consumed = 1;
    }
    write(e, t) {
      switch (this.state) {
        case j.EntityStart:
          return e.charCodeAt(t) === V.NUM ? (this.state = j.NumericStart, this.consumed += 1, this.stateNumericStart(e, t + 1)) : (this.state = j.NamedEntity, this.stateNamedEntity(e, t));
        case j.NumericStart:
          return this.stateNumericStart(e, t);
        case j.NumericDecimal:
          return this.stateNumericDecimal(e, t);
        case j.NumericHex:
          return this.stateNumericHex(e, t);
        case j.NamedEntity:
          return this.stateNamedEntity(e, t);
      }
    }
    stateNumericStart(e, t) {
      return t >= e.length ? -1 : (e.charCodeAt(t) | ea) === V.LOWER_X ? (this.state = j.NumericHex, this.consumed += 1, this.stateNumericHex(e, t + 1)) : (this.state = j.NumericDecimal, this.stateNumericDecimal(e, t));
    }
    addToNumericResult(e, t, n, r) {
      if (t !== n) {
        let a = n - t;
        this.result = this.result * Math.pow(r, a) + parseInt(e.substr(t, a), r), this.consumed += a;
      }
    }
    stateNumericHex(e, t) {
      let n = t;
      for (; t < e.length;) {
        let r = e.charCodeAt(t);
        if (_n(r) || ta(r)) t += 1;else return this.addToNumericResult(e, n, t, 16), this.emitNumericEntity(r, 3);
      }
      return this.addToNumericResult(e, n, t, 16), -1;
    }
    stateNumericDecimal(e, t) {
      let n = t;
      for (; t < e.length;) {
        let r = e.charCodeAt(t);
        if (_n(r)) t += 1;else return this.addToNumericResult(e, n, t, 10), this.emitNumericEntity(r, 2);
      }
      return this.addToNumericResult(e, n, t, 10), -1;
    }
    emitNumericEntity(e, t) {
      var n;
      if (this.consumed <= t) return (n = this.errors) === null || n === void 0 || n.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
      if (e === V.SEMI) this.consumed += 1;else if (this.decodeMode === je.Strict) return 0;
      return this.emitCodePoint($u(this.result), this.consumed), this.errors && (e !== V.SEMI && this.errors.missingSemicolonAfterCharacterReference(), this.errors.validateNumericCharacterReference(this.result)), this.consumed;
    }
    stateNamedEntity(e, t) {
      let n = this.decodeTree,
        r = n[this.treeIndex],
        a = (r & Le.VALUE_LENGTH) >> 14;
      for (; t < e.length; t++, this.excess++) {
        let i = e.charCodeAt(t);
        if (this.treeIndex = aa(n, r, this.treeIndex + Math.max(1, a), i), this.treeIndex < 0) return this.result === 0 || this.decodeMode === je.Attribute && (a === 0 || ra(i)) ? 0 : this.emitNotTerminatedNamedEntity();
        if (r = n[this.treeIndex], a = (r & Le.VALUE_LENGTH) >> 14, a !== 0) {
          if (i === V.SEMI) return this.emitNamedEntityData(this.treeIndex, a, this.consumed + this.excess);
          this.decodeMode !== je.Strict && (this.result = this.treeIndex, this.consumed += this.excess, this.excess = 0);
        }
      }
      return -1;
    }
    emitNotTerminatedNamedEntity() {
      var e;
      let t = this.result,
        n = this.decodeTree,
        r = (n[t] & Le.VALUE_LENGTH) >> 14;
      return this.emitNamedEntityData(t, r, this.consumed), (e = this.errors) === null || e === void 0 || e.missingSemicolonAfterCharacterReference(), this.consumed;
    }
    emitNamedEntityData(e, t, n) {
      let r = this.decodeTree;
      return this.emitCodePoint(t === 1 ? r[e] & ~Le.VALUE_LENGTH : r[e + 1], n), t === 3 && this.emitCodePoint(r[e + 2], n), n;
    }
    end() {
      var e;
      switch (this.state) {
        case j.NamedEntity:
          return this.result !== 0 && (this.decodeMode !== je.Attribute || this.result === this.treeIndex) ? this.emitNotTerminatedNamedEntity() : 0;
        case j.NumericDecimal:
          return this.emitNumericEntity(0, 2);
        case j.NumericHex:
          return this.emitNumericEntity(0, 3);
        case j.NumericStart:
          return (e = this.errors) === null || e === void 0 || e.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
        case j.EntityStart:
          return 0;
      }
    }
  };
  function H0(e) {
    let t = "",
      n = new ua(e, r => t += zu(r));
    return function (r, a) {
      let i = 0,
        s = 0;
      for (; (s = r.indexOf("&", s)) >= 0;) {
        t += r.slice(i, s), n.startEntity(a);
        let c = n.write(r, s + 1);
        if (c < 0) {
          i = s + n.end();
          break;
        }
        i = s + c, s = c === 0 ? i + 1 : i;
      }
      let l = t + r.slice(i);
      return t = "", l;
    };
  }
  function aa(e, t, n, r) {
    let a = (t & Le.BRANCH_LENGTH) >> 7,
      i = t & Le.JUMP_TABLE;
    if (a === 0) return i !== 0 && r === i ? n : -1;
    if (i) {
      let c = r - i;
      return c < 0 || c >= a ? -1 : e[n + c] - 1;
    }
    let s = n,
      l = s + a - 1;
    for (; s <= l;) {
      let c = s + l >>> 1,
        E = e[c];
      if (E < r) s = c + 1;else if (E > r) l = c - 1;else return e[c + a];
    }
    return -1;
  }
  var l1 = H0(Vu),
    h1 = H0(ju);
  function Gt(e) {
    for (let t = 1; t < e.length; t++) e[t][0] += e[t - 1][0] + 1;
    return e;
  }
  var d1 = new Map(Gt([[9, "&Tab;"], [0, "&NewLine;"], [22, "&excl;"], [0, "&quot;"], [0, "&num;"], [0, "&dollar;"], [0, "&percnt;"], [0, "&amp;"], [0, "&apos;"], [0, "&lpar;"], [0, "&rpar;"], [0, "&ast;"], [0, "&plus;"], [0, "&comma;"], [1, "&period;"], [0, "&sol;"], [10, "&colon;"], [0, "&semi;"], [0, {
      v: "&lt;",
      n: 8402,
      o: "&nvlt;"
    }], [0, {
      v: "&equals;",
      n: 8421,
      o: "&bne;"
    }], [0, {
      v: "&gt;",
      n: 8402,
      o: "&nvgt;"
    }], [0, "&quest;"], [0, "&commat;"], [26, "&lbrack;"], [0, "&bsol;"], [0, "&rbrack;"], [0, "&Hat;"], [0, "&lowbar;"], [0, "&DiacriticalGrave;"], [5, {
      n: 106,
      o: "&fjlig;"
    }], [20, "&lbrace;"], [0, "&verbar;"], [0, "&rbrace;"], [34, "&nbsp;"], [0, "&iexcl;"], [0, "&cent;"], [0, "&pound;"], [0, "&curren;"], [0, "&yen;"], [0, "&brvbar;"], [0, "&sect;"], [0, "&die;"], [0, "&copy;"], [0, "&ordf;"], [0, "&laquo;"], [0, "&not;"], [0, "&shy;"], [0, "&circledR;"], [0, "&macr;"], [0, "&deg;"], [0, "&PlusMinus;"], [0, "&sup2;"], [0, "&sup3;"], [0, "&acute;"], [0, "&micro;"], [0, "&para;"], [0, "&centerdot;"], [0, "&cedil;"], [0, "&sup1;"], [0, "&ordm;"], [0, "&raquo;"], [0, "&frac14;"], [0, "&frac12;"], [0, "&frac34;"], [0, "&iquest;"], [0, "&Agrave;"], [0, "&Aacute;"], [0, "&Acirc;"], [0, "&Atilde;"], [0, "&Auml;"], [0, "&angst;"], [0, "&AElig;"], [0, "&Ccedil;"], [0, "&Egrave;"], [0, "&Eacute;"], [0, "&Ecirc;"], [0, "&Euml;"], [0, "&Igrave;"], [0, "&Iacute;"], [0, "&Icirc;"], [0, "&Iuml;"], [0, "&ETH;"], [0, "&Ntilde;"], [0, "&Ograve;"], [0, "&Oacute;"], [0, "&Ocirc;"], [0, "&Otilde;"], [0, "&Ouml;"], [0, "&times;"], [0, "&Oslash;"], [0, "&Ugrave;"], [0, "&Uacute;"], [0, "&Ucirc;"], [0, "&Uuml;"], [0, "&Yacute;"], [0, "&THORN;"], [0, "&szlig;"], [0, "&agrave;"], [0, "&aacute;"], [0, "&acirc;"], [0, "&atilde;"], [0, "&auml;"], [0, "&aring;"], [0, "&aelig;"], [0, "&ccedil;"], [0, "&egrave;"], [0, "&eacute;"], [0, "&ecirc;"], [0, "&euml;"], [0, "&igrave;"], [0, "&iacute;"], [0, "&icirc;"], [0, "&iuml;"], [0, "&eth;"], [0, "&ntilde;"], [0, "&ograve;"], [0, "&oacute;"], [0, "&ocirc;"], [0, "&otilde;"], [0, "&ouml;"], [0, "&div;"], [0, "&oslash;"], [0, "&ugrave;"], [0, "&uacute;"], [0, "&ucirc;"], [0, "&uuml;"], [0, "&yacute;"], [0, "&thorn;"], [0, "&yuml;"], [0, "&Amacr;"], [0, "&amacr;"], [0, "&Abreve;"], [0, "&abreve;"], [0, "&Aogon;"], [0, "&aogon;"], [0, "&Cacute;"], [0, "&cacute;"], [0, "&Ccirc;"], [0, "&ccirc;"], [0, "&Cdot;"], [0, "&cdot;"], [0, "&Ccaron;"], [0, "&ccaron;"], [0, "&Dcaron;"], [0, "&dcaron;"], [0, "&Dstrok;"], [0, "&dstrok;"], [0, "&Emacr;"], [0, "&emacr;"], [2, "&Edot;"], [0, "&edot;"], [0, "&Eogon;"], [0, "&eogon;"], [0, "&Ecaron;"], [0, "&ecaron;"], [0, "&Gcirc;"], [0, "&gcirc;"], [0, "&Gbreve;"], [0, "&gbreve;"], [0, "&Gdot;"], [0, "&gdot;"], [0, "&Gcedil;"], [1, "&Hcirc;"], [0, "&hcirc;"], [0, "&Hstrok;"], [0, "&hstrok;"], [0, "&Itilde;"], [0, "&itilde;"], [0, "&Imacr;"], [0, "&imacr;"], [2, "&Iogon;"], [0, "&iogon;"], [0, "&Idot;"], [0, "&imath;"], [0, "&IJlig;"], [0, "&ijlig;"], [0, "&Jcirc;"], [0, "&jcirc;"], [0, "&Kcedil;"], [0, "&kcedil;"], [0, "&kgreen;"], [0, "&Lacute;"], [0, "&lacute;"], [0, "&Lcedil;"], [0, "&lcedil;"], [0, "&Lcaron;"], [0, "&lcaron;"], [0, "&Lmidot;"], [0, "&lmidot;"], [0, "&Lstrok;"], [0, "&lstrok;"], [0, "&Nacute;"], [0, "&nacute;"], [0, "&Ncedil;"], [0, "&ncedil;"], [0, "&Ncaron;"], [0, "&ncaron;"], [0, "&napos;"], [0, "&ENG;"], [0, "&eng;"], [0, "&Omacr;"], [0, "&omacr;"], [2, "&Odblac;"], [0, "&odblac;"], [0, "&OElig;"], [0, "&oelig;"], [0, "&Racute;"], [0, "&racute;"], [0, "&Rcedil;"], [0, "&rcedil;"], [0, "&Rcaron;"], [0, "&rcaron;"], [0, "&Sacute;"], [0, "&sacute;"], [0, "&Scirc;"], [0, "&scirc;"], [0, "&Scedil;"], [0, "&scedil;"], [0, "&Scaron;"], [0, "&scaron;"], [0, "&Tcedil;"], [0, "&tcedil;"], [0, "&Tcaron;"], [0, "&tcaron;"], [0, "&Tstrok;"], [0, "&tstrok;"], [0, "&Utilde;"], [0, "&utilde;"], [0, "&Umacr;"], [0, "&umacr;"], [0, "&Ubreve;"], [0, "&ubreve;"], [0, "&Uring;"], [0, "&uring;"], [0, "&Udblac;"], [0, "&udblac;"], [0, "&Uogon;"], [0, "&uogon;"], [0, "&Wcirc;"], [0, "&wcirc;"], [0, "&Ycirc;"], [0, "&ycirc;"], [0, "&Yuml;"], [0, "&Zacute;"], [0, "&zacute;"], [0, "&Zdot;"], [0, "&zdot;"], [0, "&Zcaron;"], [0, "&zcaron;"], [19, "&fnof;"], [34, "&imped;"], [63, "&gacute;"], [65, "&jmath;"], [142, "&circ;"], [0, "&caron;"], [16, "&breve;"], [0, "&DiacriticalDot;"], [0, "&ring;"], [0, "&ogon;"], [0, "&DiacriticalTilde;"], [0, "&dblac;"], [51, "&DownBreve;"], [127, "&Alpha;"], [0, "&Beta;"], [0, "&Gamma;"], [0, "&Delta;"], [0, "&Epsilon;"], [0, "&Zeta;"], [0, "&Eta;"], [0, "&Theta;"], [0, "&Iota;"], [0, "&Kappa;"], [0, "&Lambda;"], [0, "&Mu;"], [0, "&Nu;"], [0, "&Xi;"], [0, "&Omicron;"], [0, "&Pi;"], [0, "&Rho;"], [1, "&Sigma;"], [0, "&Tau;"], [0, "&Upsilon;"], [0, "&Phi;"], [0, "&Chi;"], [0, "&Psi;"], [0, "&ohm;"], [7, "&alpha;"], [0, "&beta;"], [0, "&gamma;"], [0, "&delta;"], [0, "&epsi;"], [0, "&zeta;"], [0, "&eta;"], [0, "&theta;"], [0, "&iota;"], [0, "&kappa;"], [0, "&lambda;"], [0, "&mu;"], [0, "&nu;"], [0, "&xi;"], [0, "&omicron;"], [0, "&pi;"], [0, "&rho;"], [0, "&sigmaf;"], [0, "&sigma;"], [0, "&tau;"], [0, "&upsi;"], [0, "&phi;"], [0, "&chi;"], [0, "&psi;"], [0, "&omega;"], [7, "&thetasym;"], [0, "&Upsi;"], [2, "&phiv;"], [0, "&piv;"], [5, "&Gammad;"], [0, "&digamma;"], [18, "&kappav;"], [0, "&rhov;"], [3, "&epsiv;"], [0, "&backepsilon;"], [10, "&IOcy;"], [0, "&DJcy;"], [0, "&GJcy;"], [0, "&Jukcy;"], [0, "&DScy;"], [0, "&Iukcy;"], [0, "&YIcy;"], [0, "&Jsercy;"], [0, "&LJcy;"], [0, "&NJcy;"], [0, "&TSHcy;"], [0, "&KJcy;"], [1, "&Ubrcy;"], [0, "&DZcy;"], [0, "&Acy;"], [0, "&Bcy;"], [0, "&Vcy;"], [0, "&Gcy;"], [0, "&Dcy;"], [0, "&IEcy;"], [0, "&ZHcy;"], [0, "&Zcy;"], [0, "&Icy;"], [0, "&Jcy;"], [0, "&Kcy;"], [0, "&Lcy;"], [0, "&Mcy;"], [0, "&Ncy;"], [0, "&Ocy;"], [0, "&Pcy;"], [0, "&Rcy;"], [0, "&Scy;"], [0, "&Tcy;"], [0, "&Ucy;"], [0, "&Fcy;"], [0, "&KHcy;"], [0, "&TScy;"], [0, "&CHcy;"], [0, "&SHcy;"], [0, "&SHCHcy;"], [0, "&HARDcy;"], [0, "&Ycy;"], [0, "&SOFTcy;"], [0, "&Ecy;"], [0, "&YUcy;"], [0, "&YAcy;"], [0, "&acy;"], [0, "&bcy;"], [0, "&vcy;"], [0, "&gcy;"], [0, "&dcy;"], [0, "&iecy;"], [0, "&zhcy;"], [0, "&zcy;"], [0, "&icy;"], [0, "&jcy;"], [0, "&kcy;"], [0, "&lcy;"], [0, "&mcy;"], [0, "&ncy;"], [0, "&ocy;"], [0, "&pcy;"], [0, "&rcy;"], [0, "&scy;"], [0, "&tcy;"], [0, "&ucy;"], [0, "&fcy;"], [0, "&khcy;"], [0, "&tscy;"], [0, "&chcy;"], [0, "&shcy;"], [0, "&shchcy;"], [0, "&hardcy;"], [0, "&ycy;"], [0, "&softcy;"], [0, "&ecy;"], [0, "&yucy;"], [0, "&yacy;"], [1, "&iocy;"], [0, "&djcy;"], [0, "&gjcy;"], [0, "&jukcy;"], [0, "&dscy;"], [0, "&iukcy;"], [0, "&yicy;"], [0, "&jsercy;"], [0, "&ljcy;"], [0, "&njcy;"], [0, "&tshcy;"], [0, "&kjcy;"], [1, "&ubrcy;"], [0, "&dzcy;"], [7074, "&ensp;"], [0, "&emsp;"], [0, "&emsp13;"], [0, "&emsp14;"], [1, "&numsp;"], [0, "&puncsp;"], [0, "&ThinSpace;"], [0, "&hairsp;"], [0, "&NegativeMediumSpace;"], [0, "&zwnj;"], [0, "&zwj;"], [0, "&lrm;"], [0, "&rlm;"], [0, "&dash;"], [2, "&ndash;"], [0, "&mdash;"], [0, "&horbar;"], [0, "&Verbar;"], [1, "&lsquo;"], [0, "&CloseCurlyQuote;"], [0, "&lsquor;"], [1, "&ldquo;"], [0, "&CloseCurlyDoubleQuote;"], [0, "&bdquo;"], [1, "&dagger;"], [0, "&Dagger;"], [0, "&bull;"], [2, "&nldr;"], [0, "&hellip;"], [9, "&permil;"], [0, "&pertenk;"], [0, "&prime;"], [0, "&Prime;"], [0, "&tprime;"], [0, "&backprime;"], [3, "&lsaquo;"], [0, "&rsaquo;"], [3, "&oline;"], [2, "&caret;"], [1, "&hybull;"], [0, "&frasl;"], [10, "&bsemi;"], [7, "&qprime;"], [7, {
      v: "&MediumSpace;",
      n: 8202,
      o: "&ThickSpace;"
    }], [0, "&NoBreak;"], [0, "&af;"], [0, "&InvisibleTimes;"], [0, "&ic;"], [72, "&euro;"], [46, "&tdot;"], [0, "&DotDot;"], [37, "&complexes;"], [2, "&incare;"], [4, "&gscr;"], [0, "&hamilt;"], [0, "&Hfr;"], [0, "&Hopf;"], [0, "&planckh;"], [0, "&hbar;"], [0, "&imagline;"], [0, "&Ifr;"], [0, "&lagran;"], [0, "&ell;"], [1, "&naturals;"], [0, "&numero;"], [0, "&copysr;"], [0, "&weierp;"], [0, "&Popf;"], [0, "&Qopf;"], [0, "&realine;"], [0, "&real;"], [0, "&reals;"], [0, "&rx;"], [3, "&trade;"], [1, "&integers;"], [2, "&mho;"], [0, "&zeetrf;"], [0, "&iiota;"], [2, "&bernou;"], [0, "&Cayleys;"], [1, "&escr;"], [0, "&Escr;"], [0, "&Fouriertrf;"], [1, "&Mellintrf;"], [0, "&order;"], [0, "&alefsym;"], [0, "&beth;"], [0, "&gimel;"], [0, "&daleth;"], [12, "&CapitalDifferentialD;"], [0, "&dd;"], [0, "&ee;"], [0, "&ii;"], [10, "&frac13;"], [0, "&frac23;"], [0, "&frac15;"], [0, "&frac25;"], [0, "&frac35;"], [0, "&frac45;"], [0, "&frac16;"], [0, "&frac56;"], [0, "&frac18;"], [0, "&frac38;"], [0, "&frac58;"], [0, "&frac78;"], [49, "&larr;"], [0, "&ShortUpArrow;"], [0, "&rarr;"], [0, "&darr;"], [0, "&harr;"], [0, "&updownarrow;"], [0, "&nwarr;"], [0, "&nearr;"], [0, "&LowerRightArrow;"], [0, "&LowerLeftArrow;"], [0, "&nlarr;"], [0, "&nrarr;"], [1, {
      v: "&rarrw;",
      n: 824,
      o: "&nrarrw;"
    }], [0, "&Larr;"], [0, "&Uarr;"], [0, "&Rarr;"], [0, "&Darr;"], [0, "&larrtl;"], [0, "&rarrtl;"], [0, "&LeftTeeArrow;"], [0, "&mapstoup;"], [0, "&map;"], [0, "&DownTeeArrow;"], [1, "&hookleftarrow;"], [0, "&hookrightarrow;"], [0, "&larrlp;"], [0, "&looparrowright;"], [0, "&harrw;"], [0, "&nharr;"], [1, "&lsh;"], [0, "&rsh;"], [0, "&ldsh;"], [0, "&rdsh;"], [1, "&crarr;"], [0, "&cularr;"], [0, "&curarr;"], [2, "&circlearrowleft;"], [0, "&circlearrowright;"], [0, "&leftharpoonup;"], [0, "&DownLeftVector;"], [0, "&RightUpVector;"], [0, "&LeftUpVector;"], [0, "&rharu;"], [0, "&DownRightVector;"], [0, "&dharr;"], [0, "&dharl;"], [0, "&RightArrowLeftArrow;"], [0, "&udarr;"], [0, "&LeftArrowRightArrow;"], [0, "&leftleftarrows;"], [0, "&upuparrows;"], [0, "&rightrightarrows;"], [0, "&ddarr;"], [0, "&leftrightharpoons;"], [0, "&Equilibrium;"], [0, "&nlArr;"], [0, "&nhArr;"], [0, "&nrArr;"], [0, "&DoubleLeftArrow;"], [0, "&DoubleUpArrow;"], [0, "&DoubleRightArrow;"], [0, "&dArr;"], [0, "&DoubleLeftRightArrow;"], [0, "&DoubleUpDownArrow;"], [0, "&nwArr;"], [0, "&neArr;"], [0, "&seArr;"], [0, "&swArr;"], [0, "&lAarr;"], [0, "&rAarr;"], [1, "&zigrarr;"], [6, "&larrb;"], [0, "&rarrb;"], [15, "&DownArrowUpArrow;"], [7, "&loarr;"], [0, "&roarr;"], [0, "&hoarr;"], [0, "&forall;"], [0, "&comp;"], [0, {
      v: "&part;",
      n: 824,
      o: "&npart;"
    }], [0, "&exist;"], [0, "&nexist;"], [0, "&empty;"], [1, "&Del;"], [0, "&Element;"], [0, "&NotElement;"], [1, "&ni;"], [0, "&notni;"], [2, "&prod;"], [0, "&coprod;"], [0, "&sum;"], [0, "&minus;"], [0, "&MinusPlus;"], [0, "&dotplus;"], [1, "&Backslash;"], [0, "&lowast;"], [0, "&compfn;"], [1, "&radic;"], [2, "&prop;"], [0, "&infin;"], [0, "&angrt;"], [0, {
      v: "&ang;",
      n: 8402,
      o: "&nang;"
    }], [0, "&angmsd;"], [0, "&angsph;"], [0, "&mid;"], [0, "&nmid;"], [0, "&DoubleVerticalBar;"], [0, "&NotDoubleVerticalBar;"], [0, "&and;"], [0, "&or;"], [0, {
      v: "&cap;",
      n: 65024,
      o: "&caps;"
    }], [0, {
      v: "&cup;",
      n: 65024,
      o: "&cups;"
    }], [0, "&int;"], [0, "&Int;"], [0, "&iiint;"], [0, "&conint;"], [0, "&Conint;"], [0, "&Cconint;"], [0, "&cwint;"], [0, "&ClockwiseContourIntegral;"], [0, "&awconint;"], [0, "&there4;"], [0, "&becaus;"], [0, "&ratio;"], [0, "&Colon;"], [0, "&dotminus;"], [1, "&mDDot;"], [0, "&homtht;"], [0, {
      v: "&sim;",
      n: 8402,
      o: "&nvsim;"
    }], [0, {
      v: "&backsim;",
      n: 817,
      o: "&race;"
    }], [0, {
      v: "&ac;",
      n: 819,
      o: "&acE;"
    }], [0, "&acd;"], [0, "&VerticalTilde;"], [0, "&NotTilde;"], [0, {
      v: "&eqsim;",
      n: 824,
      o: "&nesim;"
    }], [0, "&sime;"], [0, "&NotTildeEqual;"], [0, "&cong;"], [0, "&simne;"], [0, "&ncong;"], [0, "&ap;"], [0, "&nap;"], [0, "&ape;"], [0, {
      v: "&apid;",
      n: 824,
      o: "&napid;"
    }], [0, "&backcong;"], [0, {
      v: "&asympeq;",
      n: 8402,
      o: "&nvap;"
    }], [0, {
      v: "&bump;",
      n: 824,
      o: "&nbump;"
    }], [0, {
      v: "&bumpe;",
      n: 824,
      o: "&nbumpe;"
    }], [0, {
      v: "&doteq;",
      n: 824,
      o: "&nedot;"
    }], [0, "&doteqdot;"], [0, "&efDot;"], [0, "&erDot;"], [0, "&Assign;"], [0, "&ecolon;"], [0, "&ecir;"], [0, "&circeq;"], [1, "&wedgeq;"], [0, "&veeeq;"], [1, "&triangleq;"], [2, "&equest;"], [0, "&ne;"], [0, {
      v: "&Congruent;",
      n: 8421,
      o: "&bnequiv;"
    }], [0, "&nequiv;"], [1, {
      v: "&le;",
      n: 8402,
      o: "&nvle;"
    }], [0, {
      v: "&ge;",
      n: 8402,
      o: "&nvge;"
    }], [0, {
      v: "&lE;",
      n: 824,
      o: "&nlE;"
    }], [0, {
      v: "&gE;",
      n: 824,
      o: "&ngE;"
    }], [0, {
      v: "&lnE;",
      n: 65024,
      o: "&lvertneqq;"
    }], [0, {
      v: "&gnE;",
      n: 65024,
      o: "&gvertneqq;"
    }], [0, {
      v: "&ll;",
      n: new Map(Gt([[824, "&nLtv;"], [7577, "&nLt;"]]))
    }], [0, {
      v: "&gg;",
      n: new Map(Gt([[824, "&nGtv;"], [7577, "&nGt;"]]))
    }], [0, "&between;"], [0, "&NotCupCap;"], [0, "&nless;"], [0, "&ngt;"], [0, "&nle;"], [0, "&nge;"], [0, "&lesssim;"], [0, "&GreaterTilde;"], [0, "&nlsim;"], [0, "&ngsim;"], [0, "&LessGreater;"], [0, "&gl;"], [0, "&NotLessGreater;"], [0, "&NotGreaterLess;"], [0, "&pr;"], [0, "&sc;"], [0, "&prcue;"], [0, "&sccue;"], [0, "&PrecedesTilde;"], [0, {
      v: "&scsim;",
      n: 824,
      o: "&NotSucceedsTilde;"
    }], [0, "&NotPrecedes;"], [0, "&NotSucceeds;"], [0, {
      v: "&sub;",
      n: 8402,
      o: "&NotSubset;"
    }], [0, {
      v: "&sup;",
      n: 8402,
      o: "&NotSuperset;"
    }], [0, "&nsub;"], [0, "&nsup;"], [0, "&sube;"], [0, "&supe;"], [0, "&NotSubsetEqual;"], [0, "&NotSupersetEqual;"], [0, {
      v: "&subne;",
      n: 65024,
      o: "&varsubsetneq;"
    }], [0, {
      v: "&supne;",
      n: 65024,
      o: "&varsupsetneq;"
    }], [1, "&cupdot;"], [0, "&UnionPlus;"], [0, {
      v: "&sqsub;",
      n: 824,
      o: "&NotSquareSubset;"
    }], [0, {
      v: "&sqsup;",
      n: 824,
      o: "&NotSquareSuperset;"
    }], [0, "&sqsube;"], [0, "&sqsupe;"], [0, {
      v: "&sqcap;",
      n: 65024,
      o: "&sqcaps;"
    }], [0, {
      v: "&sqcup;",
      n: 65024,
      o: "&sqcups;"
    }], [0, "&CirclePlus;"], [0, "&CircleMinus;"], [0, "&CircleTimes;"], [0, "&osol;"], [0, "&CircleDot;"], [0, "&circledcirc;"], [0, "&circledast;"], [1, "&circleddash;"], [0, "&boxplus;"], [0, "&boxminus;"], [0, "&boxtimes;"], [0, "&dotsquare;"], [0, "&RightTee;"], [0, "&dashv;"], [0, "&DownTee;"], [0, "&bot;"], [1, "&models;"], [0, "&DoubleRightTee;"], [0, "&Vdash;"], [0, "&Vvdash;"], [0, "&VDash;"], [0, "&nvdash;"], [0, "&nvDash;"], [0, "&nVdash;"], [0, "&nVDash;"], [0, "&prurel;"], [1, "&LeftTriangle;"], [0, "&RightTriangle;"], [0, {
      v: "&LeftTriangleEqual;",
      n: 8402,
      o: "&nvltrie;"
    }], [0, {
      v: "&RightTriangleEqual;",
      n: 8402,
      o: "&nvrtrie;"
    }], [0, "&origof;"], [0, "&imof;"], [0, "&multimap;"], [0, "&hercon;"], [0, "&intcal;"], [0, "&veebar;"], [1, "&barvee;"], [0, "&angrtvb;"], [0, "&lrtri;"], [0, "&bigwedge;"], [0, "&bigvee;"], [0, "&bigcap;"], [0, "&bigcup;"], [0, "&diam;"], [0, "&sdot;"], [0, "&sstarf;"], [0, "&divideontimes;"], [0, "&bowtie;"], [0, "&ltimes;"], [0, "&rtimes;"], [0, "&leftthreetimes;"], [0, "&rightthreetimes;"], [0, "&backsimeq;"], [0, "&curlyvee;"], [0, "&curlywedge;"], [0, "&Sub;"], [0, "&Sup;"], [0, "&Cap;"], [0, "&Cup;"], [0, "&fork;"], [0, "&epar;"], [0, "&lessdot;"], [0, "&gtdot;"], [0, {
      v: "&Ll;",
      n: 824,
      o: "&nLl;"
    }], [0, {
      v: "&Gg;",
      n: 824,
      o: "&nGg;"
    }], [0, {
      v: "&leg;",
      n: 65024,
      o: "&lesg;"
    }], [0, {
      v: "&gel;",
      n: 65024,
      o: "&gesl;"
    }], [2, "&cuepr;"], [0, "&cuesc;"], [0, "&NotPrecedesSlantEqual;"], [0, "&NotSucceedsSlantEqual;"], [0, "&NotSquareSubsetEqual;"], [0, "&NotSquareSupersetEqual;"], [2, "&lnsim;"], [0, "&gnsim;"], [0, "&precnsim;"], [0, "&scnsim;"], [0, "&nltri;"], [0, "&NotRightTriangle;"], [0, "&nltrie;"], [0, "&NotRightTriangleEqual;"], [0, "&vellip;"], [0, "&ctdot;"], [0, "&utdot;"], [0, "&dtdot;"], [0, "&disin;"], [0, "&isinsv;"], [0, "&isins;"], [0, {
      v: "&isindot;",
      n: 824,
      o: "&notindot;"
    }], [0, "&notinvc;"], [0, "&notinvb;"], [1, {
      v: "&isinE;",
      n: 824,
      o: "&notinE;"
    }], [0, "&nisd;"], [0, "&xnis;"], [0, "&nis;"], [0, "&notnivc;"], [0, "&notnivb;"], [6, "&barwed;"], [0, "&Barwed;"], [1, "&lceil;"], [0, "&rceil;"], [0, "&LeftFloor;"], [0, "&rfloor;"], [0, "&drcrop;"], [0, "&dlcrop;"], [0, "&urcrop;"], [0, "&ulcrop;"], [0, "&bnot;"], [1, "&profline;"], [0, "&profsurf;"], [1, "&telrec;"], [0, "&target;"], [5, "&ulcorn;"], [0, "&urcorn;"], [0, "&dlcorn;"], [0, "&drcorn;"], [2, "&frown;"], [0, "&smile;"], [9, "&cylcty;"], [0, "&profalar;"], [7, "&topbot;"], [6, "&ovbar;"], [1, "&solbar;"], [60, "&angzarr;"], [51, "&lmoustache;"], [0, "&rmoustache;"], [2, "&OverBracket;"], [0, "&bbrk;"], [0, "&bbrktbrk;"], [37, "&OverParenthesis;"], [0, "&UnderParenthesis;"], [0, "&OverBrace;"], [0, "&UnderBrace;"], [2, "&trpezium;"], [4, "&elinters;"], [59, "&blank;"], [164, "&circledS;"], [55, "&boxh;"], [1, "&boxv;"], [9, "&boxdr;"], [3, "&boxdl;"], [3, "&boxur;"], [3, "&boxul;"], [3, "&boxvr;"], [7, "&boxvl;"], [7, "&boxhd;"], [7, "&boxhu;"], [7, "&boxvh;"], [19, "&boxH;"], [0, "&boxV;"], [0, "&boxdR;"], [0, "&boxDr;"], [0, "&boxDR;"], [0, "&boxdL;"], [0, "&boxDl;"], [0, "&boxDL;"], [0, "&boxuR;"], [0, "&boxUr;"], [0, "&boxUR;"], [0, "&boxuL;"], [0, "&boxUl;"], [0, "&boxUL;"], [0, "&boxvR;"], [0, "&boxVr;"], [0, "&boxVR;"], [0, "&boxvL;"], [0, "&boxVl;"], [0, "&boxVL;"], [0, "&boxHd;"], [0, "&boxhD;"], [0, "&boxHD;"], [0, "&boxHu;"], [0, "&boxhU;"], [0, "&boxHU;"], [0, "&boxvH;"], [0, "&boxVh;"], [0, "&boxVH;"], [19, "&uhblk;"], [3, "&lhblk;"], [3, "&block;"], [8, "&blk14;"], [0, "&blk12;"], [0, "&blk34;"], [13, "&square;"], [8, "&blacksquare;"], [0, "&EmptyVerySmallSquare;"], [1, "&rect;"], [0, "&marker;"], [2, "&fltns;"], [1, "&bigtriangleup;"], [0, "&blacktriangle;"], [0, "&triangle;"], [2, "&blacktriangleright;"], [0, "&rtri;"], [3, "&bigtriangledown;"], [0, "&blacktriangledown;"], [0, "&dtri;"], [2, "&blacktriangleleft;"], [0, "&ltri;"], [6, "&loz;"], [0, "&cir;"], [32, "&tridot;"], [2, "&bigcirc;"], [8, "&ultri;"], [0, "&urtri;"], [0, "&lltri;"], [0, "&EmptySmallSquare;"], [0, "&FilledSmallSquare;"], [8, "&bigstar;"], [0, "&star;"], [7, "&phone;"], [49, "&female;"], [1, "&male;"], [29, "&spades;"], [2, "&clubs;"], [1, "&hearts;"], [0, "&diamondsuit;"], [3, "&sung;"], [2, "&flat;"], [0, "&natural;"], [0, "&sharp;"], [163, "&check;"], [3, "&cross;"], [8, "&malt;"], [21, "&sext;"], [33, "&VerticalSeparator;"], [25, "&lbbrk;"], [0, "&rbbrk;"], [84, "&bsolhsub;"], [0, "&suphsol;"], [28, "&LeftDoubleBracket;"], [0, "&RightDoubleBracket;"], [0, "&lang;"], [0, "&rang;"], [0, "&Lang;"], [0, "&Rang;"], [0, "&loang;"], [0, "&roang;"], [7, "&longleftarrow;"], [0, "&longrightarrow;"], [0, "&longleftrightarrow;"], [0, "&DoubleLongLeftArrow;"], [0, "&DoubleLongRightArrow;"], [0, "&DoubleLongLeftRightArrow;"], [1, "&longmapsto;"], [2, "&dzigrarr;"], [258, "&nvlArr;"], [0, "&nvrArr;"], [0, "&nvHarr;"], [0, "&Map;"], [6, "&lbarr;"], [0, "&bkarow;"], [0, "&lBarr;"], [0, "&dbkarow;"], [0, "&drbkarow;"], [0, "&DDotrahd;"], [0, "&UpArrowBar;"], [0, "&DownArrowBar;"], [2, "&Rarrtl;"], [2, "&latail;"], [0, "&ratail;"], [0, "&lAtail;"], [0, "&rAtail;"], [0, "&larrfs;"], [0, "&rarrfs;"], [0, "&larrbfs;"], [0, "&rarrbfs;"], [2, "&nwarhk;"], [0, "&nearhk;"], [0, "&hksearow;"], [0, "&hkswarow;"], [0, "&nwnear;"], [0, "&nesear;"], [0, "&seswar;"], [0, "&swnwar;"], [8, {
      v: "&rarrc;",
      n: 824,
      o: "&nrarrc;"
    }], [1, "&cudarrr;"], [0, "&ldca;"], [0, "&rdca;"], [0, "&cudarrl;"], [0, "&larrpl;"], [2, "&curarrm;"], [0, "&cularrp;"], [7, "&rarrpl;"], [2, "&harrcir;"], [0, "&Uarrocir;"], [0, "&lurdshar;"], [0, "&ldrushar;"], [2, "&LeftRightVector;"], [0, "&RightUpDownVector;"], [0, "&DownLeftRightVector;"], [0, "&LeftUpDownVector;"], [0, "&LeftVectorBar;"], [0, "&RightVectorBar;"], [0, "&RightUpVectorBar;"], [0, "&RightDownVectorBar;"], [0, "&DownLeftVectorBar;"], [0, "&DownRightVectorBar;"], [0, "&LeftUpVectorBar;"], [0, "&LeftDownVectorBar;"], [0, "&LeftTeeVector;"], [0, "&RightTeeVector;"], [0, "&RightUpTeeVector;"], [0, "&RightDownTeeVector;"], [0, "&DownLeftTeeVector;"], [0, "&DownRightTeeVector;"], [0, "&LeftUpTeeVector;"], [0, "&LeftDownTeeVector;"], [0, "&lHar;"], [0, "&uHar;"], [0, "&rHar;"], [0, "&dHar;"], [0, "&luruhar;"], [0, "&ldrdhar;"], [0, "&ruluhar;"], [0, "&rdldhar;"], [0, "&lharul;"], [0, "&llhard;"], [0, "&rharul;"], [0, "&lrhard;"], [0, "&udhar;"], [0, "&duhar;"], [0, "&RoundImplies;"], [0, "&erarr;"], [0, "&simrarr;"], [0, "&larrsim;"], [0, "&rarrsim;"], [0, "&rarrap;"], [0, "&ltlarr;"], [1, "&gtrarr;"], [0, "&subrarr;"], [1, "&suplarr;"], [0, "&lfisht;"], [0, "&rfisht;"], [0, "&ufisht;"], [0, "&dfisht;"], [5, "&lopar;"], [0, "&ropar;"], [4, "&lbrke;"], [0, "&rbrke;"], [0, "&lbrkslu;"], [0, "&rbrksld;"], [0, "&lbrksld;"], [0, "&rbrkslu;"], [0, "&langd;"], [0, "&rangd;"], [0, "&lparlt;"], [0, "&rpargt;"], [0, "&gtlPar;"], [0, "&ltrPar;"], [3, "&vzigzag;"], [1, "&vangrt;"], [0, "&angrtvbd;"], [6, "&ange;"], [0, "&range;"], [0, "&dwangle;"], [0, "&uwangle;"], [0, "&angmsdaa;"], [0, "&angmsdab;"], [0, "&angmsdac;"], [0, "&angmsdad;"], [0, "&angmsdae;"], [0, "&angmsdaf;"], [0, "&angmsdag;"], [0, "&angmsdah;"], [0, "&bemptyv;"], [0, "&demptyv;"], [0, "&cemptyv;"], [0, "&raemptyv;"], [0, "&laemptyv;"], [0, "&ohbar;"], [0, "&omid;"], [0, "&opar;"], [1, "&operp;"], [1, "&olcross;"], [0, "&odsold;"], [1, "&olcir;"], [0, "&ofcir;"], [0, "&olt;"], [0, "&ogt;"], [0, "&cirscir;"], [0, "&cirE;"], [0, "&solb;"], [0, "&bsolb;"], [3, "&boxbox;"], [3, "&trisb;"], [0, "&rtriltri;"], [0, {
      v: "&LeftTriangleBar;",
      n: 824,
      o: "&NotLeftTriangleBar;"
    }], [0, {
      v: "&RightTriangleBar;",
      n: 824,
      o: "&NotRightTriangleBar;"
    }], [11, "&iinfin;"], [0, "&infintie;"], [0, "&nvinfin;"], [4, "&eparsl;"], [0, "&smeparsl;"], [0, "&eqvparsl;"], [5, "&blacklozenge;"], [8, "&RuleDelayed;"], [1, "&dsol;"], [9, "&bigodot;"], [0, "&bigoplus;"], [0, "&bigotimes;"], [1, "&biguplus;"], [1, "&bigsqcup;"], [5, "&iiiint;"], [0, "&fpartint;"], [2, "&cirfnint;"], [0, "&awint;"], [0, "&rppolint;"], [0, "&scpolint;"], [0, "&npolint;"], [0, "&pointint;"], [0, "&quatint;"], [0, "&intlarhk;"], [10, "&pluscir;"], [0, "&plusacir;"], [0, "&simplus;"], [0, "&plusdu;"], [0, "&plussim;"], [0, "&plustwo;"], [1, "&mcomma;"], [0, "&minusdu;"], [2, "&loplus;"], [0, "&roplus;"], [0, "&Cross;"], [0, "&timesd;"], [0, "&timesbar;"], [1, "&smashp;"], [0, "&lotimes;"], [0, "&rotimes;"], [0, "&otimesas;"], [0, "&Otimes;"], [0, "&odiv;"], [0, "&triplus;"], [0, "&triminus;"], [0, "&tritime;"], [0, "&intprod;"], [2, "&amalg;"], [0, "&capdot;"], [1, "&ncup;"], [0, "&ncap;"], [0, "&capand;"], [0, "&cupor;"], [0, "&cupcap;"], [0, "&capcup;"], [0, "&cupbrcap;"], [0, "&capbrcup;"], [0, "&cupcup;"], [0, "&capcap;"], [0, "&ccups;"], [0, "&ccaps;"], [2, "&ccupssm;"], [2, "&And;"], [0, "&Or;"], [0, "&andand;"], [0, "&oror;"], [0, "&orslope;"], [0, "&andslope;"], [1, "&andv;"], [0, "&orv;"], [0, "&andd;"], [0, "&ord;"], [1, "&wedbar;"], [6, "&sdote;"], [3, "&simdot;"], [2, {
      v: "&congdot;",
      n: 824,
      o: "&ncongdot;"
    }], [0, "&easter;"], [0, "&apacir;"], [0, {
      v: "&apE;",
      n: 824,
      o: "&napE;"
    }], [0, "&eplus;"], [0, "&pluse;"], [0, "&Esim;"], [0, "&Colone;"], [0, "&Equal;"], [1, "&ddotseq;"], [0, "&equivDD;"], [0, "&ltcir;"], [0, "&gtcir;"], [0, "&ltquest;"], [0, "&gtquest;"], [0, {
      v: "&leqslant;",
      n: 824,
      o: "&nleqslant;"
    }], [0, {
      v: "&geqslant;",
      n: 824,
      o: "&ngeqslant;"
    }], [0, "&lesdot;"], [0, "&gesdot;"], [0, "&lesdoto;"], [0, "&gesdoto;"], [0, "&lesdotor;"], [0, "&gesdotol;"], [0, "&lap;"], [0, "&gap;"], [0, "&lne;"], [0, "&gne;"], [0, "&lnap;"], [0, "&gnap;"], [0, "&lEg;"], [0, "&gEl;"], [0, "&lsime;"], [0, "&gsime;"], [0, "&lsimg;"], [0, "&gsiml;"], [0, "&lgE;"], [0, "&glE;"], [0, "&lesges;"], [0, "&gesles;"], [0, "&els;"], [0, "&egs;"], [0, "&elsdot;"], [0, "&egsdot;"], [0, "&el;"], [0, "&eg;"], [2, "&siml;"], [0, "&simg;"], [0, "&simlE;"], [0, "&simgE;"], [0, {
      v: "&LessLess;",
      n: 824,
      o: "&NotNestedLessLess;"
    }], [0, {
      v: "&GreaterGreater;",
      n: 824,
      o: "&NotNestedGreaterGreater;"
    }], [1, "&glj;"], [0, "&gla;"], [0, "&ltcc;"], [0, "&gtcc;"], [0, "&lescc;"], [0, "&gescc;"], [0, "&smt;"], [0, "&lat;"], [0, {
      v: "&smte;",
      n: 65024,
      o: "&smtes;"
    }], [0, {
      v: "&late;",
      n: 65024,
      o: "&lates;"
    }], [0, "&bumpE;"], [0, {
      v: "&PrecedesEqual;",
      n: 824,
      o: "&NotPrecedesEqual;"
    }], [0, {
      v: "&sce;",
      n: 824,
      o: "&NotSucceedsEqual;"
    }], [2, "&prE;"], [0, "&scE;"], [0, "&precneqq;"], [0, "&scnE;"], [0, "&prap;"], [0, "&scap;"], [0, "&precnapprox;"], [0, "&scnap;"], [0, "&Pr;"], [0, "&Sc;"], [0, "&subdot;"], [0, "&supdot;"], [0, "&subplus;"], [0, "&supplus;"], [0, "&submult;"], [0, "&supmult;"], [0, "&subedot;"], [0, "&supedot;"], [0, {
      v: "&subE;",
      n: 824,
      o: "&nsubE;"
    }], [0, {
      v: "&supE;",
      n: 824,
      o: "&nsupE;"
    }], [0, "&subsim;"], [0, "&supsim;"], [2, {
      v: "&subnE;",
      n: 65024,
      o: "&varsubsetneqq;"
    }], [0, {
      v: "&supnE;",
      n: 65024,
      o: "&varsupsetneqq;"
    }], [2, "&csub;"], [0, "&csup;"], [0, "&csube;"], [0, "&csupe;"], [0, "&subsup;"], [0, "&supsub;"], [0, "&subsub;"], [0, "&supsup;"], [0, "&suphsub;"], [0, "&supdsub;"], [0, "&forkv;"], [0, "&topfork;"], [0, "&mlcp;"], [8, "&Dashv;"], [1, "&Vdashl;"], [0, "&Barv;"], [0, "&vBar;"], [0, "&vBarv;"], [1, "&Vbar;"], [0, "&Not;"], [0, "&bNot;"], [0, "&rnmid;"], [0, "&cirmid;"], [0, "&midcir;"], [0, "&topcir;"], [0, "&nhpar;"], [0, "&parsim;"], [9, {
      v: "&parsl;",
      n: 8421,
      o: "&nparsl;"
    }], [44343, {
      n: new Map(Gt([[56476, "&Ascr;"], [1, "&Cscr;"], [0, "&Dscr;"], [2, "&Gscr;"], [2, "&Jscr;"], [0, "&Kscr;"], [2, "&Nscr;"], [0, "&Oscr;"], [0, "&Pscr;"], [0, "&Qscr;"], [1, "&Sscr;"], [0, "&Tscr;"], [0, "&Uscr;"], [0, "&Vscr;"], [0, "&Wscr;"], [0, "&Xscr;"], [0, "&Yscr;"], [0, "&Zscr;"], [0, "&ascr;"], [0, "&bscr;"], [0, "&cscr;"], [0, "&dscr;"], [1, "&fscr;"], [1, "&hscr;"], [0, "&iscr;"], [0, "&jscr;"], [0, "&kscr;"], [0, "&lscr;"], [0, "&mscr;"], [0, "&nscr;"], [1, "&pscr;"], [0, "&qscr;"], [0, "&rscr;"], [0, "&sscr;"], [0, "&tscr;"], [0, "&uscr;"], [0, "&vscr;"], [0, "&wscr;"], [0, "&xscr;"], [0, "&yscr;"], [0, "&zscr;"], [52, "&Afr;"], [0, "&Bfr;"], [1, "&Dfr;"], [0, "&Efr;"], [0, "&Ffr;"], [0, "&Gfr;"], [2, "&Jfr;"], [0, "&Kfr;"], [0, "&Lfr;"], [0, "&Mfr;"], [0, "&Nfr;"], [0, "&Ofr;"], [0, "&Pfr;"], [0, "&Qfr;"], [1, "&Sfr;"], [0, "&Tfr;"], [0, "&Ufr;"], [0, "&Vfr;"], [0, "&Wfr;"], [0, "&Xfr;"], [0, "&Yfr;"], [1, "&afr;"], [0, "&bfr;"], [0, "&cfr;"], [0, "&dfr;"], [0, "&efr;"], [0, "&ffr;"], [0, "&gfr;"], [0, "&hfr;"], [0, "&ifr;"], [0, "&jfr;"], [0, "&kfr;"], [0, "&lfr;"], [0, "&mfr;"], [0, "&nfr;"], [0, "&ofr;"], [0, "&pfr;"], [0, "&qfr;"], [0, "&rfr;"], [0, "&sfr;"], [0, "&tfr;"], [0, "&ufr;"], [0, "&vfr;"], [0, "&wfr;"], [0, "&xfr;"], [0, "&yfr;"], [0, "&zfr;"], [0, "&Aopf;"], [0, "&Bopf;"], [1, "&Dopf;"], [0, "&Eopf;"], [0, "&Fopf;"], [0, "&Gopf;"], [1, "&Iopf;"], [0, "&Jopf;"], [0, "&Kopf;"], [0, "&Lopf;"], [0, "&Mopf;"], [1, "&Oopf;"], [3, "&Sopf;"], [0, "&Topf;"], [0, "&Uopf;"], [0, "&Vopf;"], [0, "&Wopf;"], [0, "&Xopf;"], [0, "&Yopf;"], [1, "&aopf;"], [0, "&bopf;"], [0, "&copf;"], [0, "&dopf;"], [0, "&eopf;"], [0, "&fopf;"], [0, "&gopf;"], [0, "&hopf;"], [0, "&iopf;"], [0, "&jopf;"], [0, "&kopf;"], [0, "&lopf;"], [0, "&mopf;"], [0, "&nopf;"], [0, "&oopf;"], [0, "&popf;"], [0, "&qopf;"], [0, "&ropf;"], [0, "&sopf;"], [0, "&topf;"], [0, "&uopf;"], [0, "&vopf;"], [0, "&wopf;"], [0, "&xopf;"], [0, "&yopf;"], [0, "&zopf;"]]))
    }], [8906, "&fflig;"], [0, "&filig;"], [0, "&fllig;"], [0, "&ffilig;"], [0, "&ffllig;"]])),
    P0 = /["&'<>$\x80-\uFFFF]/g,
    U0 = new Map([[34, "&quot;"], [38, "&amp;"], [39, "&apos;"], [60, "&lt;"], [62, "&gt;"]]),
    ia = String.prototype.codePointAt != null ? (e, t) => e.codePointAt(t) : (e, t) => (e.charCodeAt(t) & 64512) === 55296 ? (e.charCodeAt(t) - 55296) * 1024 + e.charCodeAt(t + 1) - 56320 + 65536 : e.charCodeAt(t);
  function W0(e) {
    let t = "",
      n = 0,
      r;
    for (; (r = P0.exec(e)) !== null;) {
      let a = r.index,
        i = e.charCodeAt(a),
        s = U0.get(i);
      s !== void 0 ? (t += e.substring(n, a) + s, n = a + 1) : (t += "".concat(e.substring(n, a), "&#x").concat(ia(e, a).toString(16), ";"), n = P0.lastIndex += +((i & 64512) === 55296));
    }
    return t + e.substr(n);
  }
  function yn(e, t) {
    return function (n) {
      let r,
        a = 0,
        i = "";
      for (; r = e.exec(n);) a !== r.index && (i += n.substring(a, r.index)), i += t.get(r[0].charCodeAt(0)), a = r.index + 1;
      return i + n.substring(a);
    };
  }
  var E1 = yn(/[&<>'"]/g, U0),
    sa = yn(/["&\u00A0]/g, new Map([[34, "&quot;"], [38, "&amp;"], [160, "&nbsp;"]])),
    oa = yn(/[&<>\u00A0]/g, new Map([[38, "&amp;"], [60, "&lt;"], [62, "&gt;"], [160, "&nbsp;"]])),
    G0;
  (function (e) {
    e[e.XML = 0] = "XML", e[e.HTML = 1] = "HTML";
  })(G0 || (G0 = {}));
  var K0;
  (function (e) {
    e[e.UTF8 = 0] = "UTF8", e[e.ASCII = 1] = "ASCII", e[e.Extensive = 2] = "Extensive", e[e.Attribute = 3] = "Attribute", e[e.Text = 4] = "Text";
  })(K0 || (K0 = {}));
  var Aa = new Map(["altGlyph", "altGlyphDef", "altGlyphItem", "animateColor", "animateMotion", "animateTransform", "clipPath", "feBlend", "feColorMatrix", "feComponentTransfer", "feComposite", "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap", "feDistantLight", "feDropShadow", "feFlood", "feFuncA", "feFuncB", "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge", "feMergeNode", "feMorphology", "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight", "feTile", "feTurbulence", "foreignObject", "glyphRef", "linearGradient", "radialGradient", "textPath"].map(e => [e.toLowerCase(), e])),
    ca = new Map(["definitionURL", "attributeName", "attributeType", "baseFrequency", "baseProfile", "calcMode", "clipPathUnits", "diffuseConstant", "edgeMode", "filterUnits", "glyphRef", "gradientTransform", "gradientUnits", "kernelMatrix", "kernelUnitLength", "keyPoints", "keySplines", "keyTimes", "lengthAdjust", "limitingConeAngle", "markerHeight", "markerUnits", "markerWidth", "maskContentUnits", "maskUnits", "numOctaves", "pathLength", "patternContentUnits", "patternTransform", "patternUnits", "pointsAtX", "pointsAtY", "pointsAtZ", "preserveAlpha", "preserveAspectRatio", "primitiveUnits", "refX", "refY", "repeatCount", "repeatDur", "requiredExtensions", "requiredFeatures", "specularConstant", "specularExponent", "spreadMethod", "startOffset", "stdDeviation", "stitchTiles", "surfaceScale", "systemLanguage", "tableValues", "targetX", "targetY", "textLength", "viewBox", "viewTarget", "xChannelSelector", "yChannelSelector", "zoomAndPan"].map(e => [e.toLowerCase(), e])),
    la = new Set(["style", "script", "xmp", "iframe", "noembed", "noframes", "plaintext", "noscript"]);
  function ha(e) {
    return e.replace(/"/g, "&quot;");
  }
  function da(e, t) {
    var n;
    if (!e) return;
    let r = ((n = t.encodeEntities) !== null && n !== void 0 ? n : t.decodeEntities) === !1 ? ha : t.xmlMode || t.encodeEntities !== "utf8" ? W0 : sa;
    return Object.keys(e).map(a => {
      var i, s;
      let l = (i = e[a]) !== null && i !== void 0 ? i : "";
      return t.xmlMode === "foreign" && (a = (s = ca.get(a)) !== null && s !== void 0 ? s : a), !t.emptyAttrs && !t.xmlMode && l === "" ? a : "".concat(a, '="').concat(r(l), '"');
    }).join(" ");
  }
  var Q0 = new Set(["area", "base", "basefont", "br", "col", "command", "embed", "frame", "hr", "img", "input", "isindex", "keygen", "link", "meta", "param", "source", "track", "wbr"]);
  function bn(e, t = {}) {
    let n = "length" in e ? e : [e],
      r = "";
    for (let a = 0; a < n.length; a++) r += Ea(n[a], t);
    return r;
  }
  var Y0 = bn;
  function Ea(e, t) {
    switch (e.type) {
      case y0:
        return bn(e.children, t);
      case R0:
      case S0:
        return xa(e);
      case I0:
        return Da(e);
      case w0:
        return ma(e);
      case v0:
      case N0:
      case k0:
        return fa(e, t);
      case b0:
        return Ba(e, t);
    }
  }
  var Ca = new Set(["mi", "mo", "mn", "ms", "mtext", "annotation-xml", "foreignObject", "desc", "title"]),
    pa = new Set(["svg", "math"]);
  function fa(e, t) {
    var n;
    t.xmlMode === "foreign" && (e.name = (n = Aa.get(e.name)) !== null && n !== void 0 ? n : e.name, e.parent && Ca.has(e.parent.name) && (t = Ye(L({}, t), {
      xmlMode: !1
    }))), !t.xmlMode && pa.has(e.name) && (t = Ye(L({}, t), {
      xmlMode: "foreign"
    }));
    let r = "<".concat(e.name),
      a = da(e.attribs, t);
    return a && (r += " ".concat(a)), e.children.length === 0 && (t.xmlMode ? t.selfClosingTags !== !1 : t.selfClosingTags && Q0.has(e.name)) ? (t.xmlMode || (r += " "), r += "/>") : (r += ">", e.children.length > 0 && (r += bn(e.children, t)), (t.xmlMode || !Q0.has(e.name)) && (r += "</".concat(e.name, ">"))), r;
  }
  function xa(e) {
    return "<".concat(e.data, ">");
  }
  function Ba(e, t) {
    var n;
    let r = e.data || "";
    return ((n = t.encodeEntities) !== null && n !== void 0 ? n : t.decodeEntities) !== !1 && !(!t.xmlMode && e.parent && la.has(e.parent.name)) && (r = t.xmlMode || t.encodeEntities !== "utf8" ? W0(r) : oa(r)), r;
  }
  function ma(e) {
    return "<![CDATA[".concat(e.children[0].data, "]]>");
  }
  function Da(e) {
    return "<!--".concat(e.data, "-->");
  }
  function X0(e, t) {
    return Y0(e, t);
  }
  function ga(e, t) {
    return Q(e) ? e.children.map(n => X0(n, t)).join("") : "";
  }
  function Kt(e) {
    return Array.isArray(e) ? e.map(Kt).join("") : y(e) ? e.name === "br" ? "\n" : Kt(e.children) : Ut(e) ? Kt(e.children) : Be(e) ? e.data : "";
  }
  function ut(e) {
    return Array.isArray(e) ? e.map(ut).join("") : Q(e) && !Wt(e) ? ut(e.children) : Be(e) ? e.data : "";
  }
  function Qt(e) {
    return Array.isArray(e) ? e.map(Qt).join("") : Q(e) && (e.type === N.Tag || Ut(e)) ? Qt(e.children) : Be(e) ? e.data : "";
  }
  function Yt(e) {
    return Q(e) ? e.children : [];
  }
  function J0(e) {
    return e.parent || null;
  }
  function q0(e) {
    let t = J0(e);
    if (t != null) return Yt(t);
    let n = [e],
      r = e.prev,
      a = e.next;
    for (; r != null;) {
      var i;
      n.unshift(r), i = r, r = i.prev;
    }
    for (; a != null;) {
      var s;
      n.push(a), s = a, a = s.next;
    }
    return n;
  }
  function Ta(e, t) {
    var n;
    return (n = e.attribs) === null || n === void 0 ? void 0 : n[t];
  }
  function Fa(e, t) {
    return e.attribs != null && Object.prototype.hasOwnProperty.call(e.attribs, t) && e.attribs[t] != null;
  }
  function _a(e) {
    return e.name;
  }
  function Sn(e) {
    let t = e.next;
    for (; t !== null && !y(t);) {
      var n = t;
      t = n.next;
    }
    return t;
  }
  function In(e) {
    let t = e.prev;
    for (; t !== null && !y(t);) {
      var n = t;
      t = n.prev;
    }
    return t;
  }
  function Ze(e) {
    if (e.prev && (e.prev.next = e.next), e.next && (e.next.prev = e.prev), e.parent) {
      let t = e.parent.children,
        n = t.lastIndexOf(e);
      n >= 0 && t.splice(n, 1);
    }
    e.next = null, e.prev = null, e.parent = null;
  }
  function ya(e, t) {
    let n = t.prev = e.prev;
    n && (n.next = t);
    let r = t.next = e.next;
    r && (r.prev = t);
    let a = t.parent = e.parent;
    if (a) {
      let i = a.children;
      i[i.lastIndexOf(e)] = t, e.parent = null;
    }
  }
  function ba(e, t) {
    if (Ze(t), t.next = null, t.parent = e, e.children.push(t) > 1) {
      let n = e.children[e.children.length - 2];
      n.next = t, t.prev = n;
    } else t.prev = null;
  }
  function Sa(e, t) {
    Ze(t);
    let n = e.parent,
      r = e.next;
    if (t.next = r, t.prev = e, e.next = t, t.parent = n, r) {
      if (r.prev = t, n) {
        let a = n.children;
        a.splice(a.lastIndexOf(r), 0, t);
      }
    } else n && n.children.push(t);
  }
  function Ia(e, t) {
    if (Ze(t), t.parent = e, t.prev = null, e.children.unshift(t) !== 1) {
      let n = e.children[1];
      n.prev = t, t.next = n;
    } else t.next = null;
  }
  function va(e, t) {
    Ze(t);
    let n = e.parent;
    if (n) {
      let r = n.children;
      r.splice(r.indexOf(e), 0, t);
    }
    e.prev && (e.prev.next = t), t.parent = n, t.prev = e.prev, t.next = e, e.prev = t;
  }
  function Ct(e, t, n = !0, r = 1 / 0) {
    return vn(e, Array.isArray(t) ? t : [t], n, r);
  }
  function vn(e, t, n, r) {
    let a = [],
      i = [Array.isArray(t) ? t : [t]],
      s = [0];
    for (;;) {
      if (s[0] >= i[0].length) {
        if (s.length === 1) return a;
        i.shift(), s.shift();
        continue;
      }
      let l = i[0][s[0]++];
      if (e(l) && (a.push(l), --r <= 0)) return a;
      n && Q(l) && l.children.length > 0 && (s.unshift(0), i.unshift(l.children));
    }
  }
  function Na(e, t) {
    return t.find(e);
  }
  function Nn(e, t, n = !0) {
    let r = Array.isArray(t) ? t : [t];
    for (let a = 0; a < r.length; a++) {
      let i = r[a];
      if (y(i) && e(i)) return i;
      if (n && Q(i) && i.children.length > 0) {
        let s = Nn(e, i.children, !0);
        if (s) return s;
      }
    }
    return null;
  }
  function V0(e, t) {
    return (Array.isArray(t) ? t : [t]).some(n => y(n) && e(n) || Q(n) && V0(e, n.children));
  }
  function ka(e, t) {
    let n = [],
      r = [Array.isArray(t) ? t : [t]],
      a = [0];
    for (;;) {
      if (a[0] >= r[0].length) {
        if (r.length === 1) return n;
        r.shift(), a.shift();
        continue;
      }
      let i = r[0][a[0]++];
      y(i) && e(i) && n.push(i), Q(i) && i.children.length > 0 && (a.unshift(0), r.unshift(i.children));
    }
  }
  var Xt = {
    tag_name(e) {
      return typeof e == "function" ? t => y(t) && e(t.name) : e === "*" ? y : t => y(t) && t.name === e;
    },
    tag_type(e) {
      return typeof e == "function" ? t => e(t.type) : t => t.type === e;
    },
    tag_contains(e) {
      return typeof e == "function" ? t => Be(t) && e(t.data) : t => Be(t) && t.data === e;
    }
  };
  function kn(e, t) {
    return typeof t == "function" ? n => y(n) && t(n.attribs[e]) : n => y(n) && n.attribs[e] === t;
  }
  function wa(e, t) {
    return n => e(n) || t(n);
  }
  function j0(e) {
    let t = Object.keys(e).map(n => {
      let r = e[n];
      return Object.prototype.hasOwnProperty.call(Xt, n) ? Xt[n](r) : kn(n, r);
    });
    return t.length === 0 ? null : t.reduce(wa);
  }
  function Ra(e, t) {
    let n = j0(e);
    return n ? n(t) : !0;
  }
  function Oa(e, t, n, r = 1 / 0) {
    let a = j0(e);
    return a ? Ct(a, t, n, r) : [];
  }
  function La(e, t, n = !0) {
    return Array.isArray(t) || (t = [t]), Nn(kn("id", e), t, n);
  }
  function at(e, t, n = !0, r = 1 / 0) {
    return Ct(Xt.tag_name(e), t, n, r);
  }
  function Ma(e, t, n = !0, r = 1 / 0) {
    return Ct(kn("class", e), t, n, r);
  }
  function Ha(e, t, n = !0, r = 1 / 0) {
    return Ct(Xt.tag_type(e), t, n, r);
  }
  function Pa(e) {
    let t = e.length;
    for (; --t >= 0;) {
      let n = e[t];
      if (t > 0 && e.lastIndexOf(n, t - 1) >= 0) {
        e.splice(t, 1);
        continue;
      }
      for (let r = n.parent; r; r = r.parent) if (e.includes(r)) {
        e.splice(t, 1);
        break;
      }
    }
    return e;
  }
  var Ee;
  (function (e) {
    e[e.DISCONNECTED = 1] = "DISCONNECTED", e[e.PRECEDING = 2] = "PRECEDING", e[e.FOLLOWING = 4] = "FOLLOWING", e[e.CONTAINS = 8] = "CONTAINS", e[e.CONTAINED_BY = 16] = "CONTAINED_BY";
  })(Ee || (Ee = {}));
  function Z0(e, t) {
    let n = [],
      r = [];
    if (e === t) return 0;
    let a = Q(e) ? e : e.parent;
    for (; a;) n.unshift(a), a = a.parent;
    for (a = Q(t) ? t : t.parent; a;) r.unshift(a), a = a.parent;
    let i = Math.min(n.length, r.length),
      s = 0;
    for (; s < i && n[s] === r[s];) s++;
    if (s === 0) return Ee.DISCONNECTED;
    let l = n[s - 1],
      c = l.children,
      E = n[s],
      f = r[s];
    return c.indexOf(E) > c.indexOf(f) ? l === t ? Ee.FOLLOWING | Ee.CONTAINED_BY : Ee.FOLLOWING : l === e ? Ee.PRECEDING | Ee.CONTAINS : Ee.PRECEDING;
  }
  function it(e) {
    return e = e.filter((t, n, r) => !r.includes(t, n + 1)), e.sort((t, n) => {
      let r = Z0(t, n);
      return r & Ee.PRECEDING ? -1 : r & Ee.FOLLOWING ? 1 : 0;
    }), e;
  }
  function Ua(e) {
    let t = Jt(Ya, e);
    return t ? t.name === "feed" ? Wa(t) : Ga(t) : null;
  }
  function Wa(e) {
    var t;
    let n = e.children,
      r = {
        type: "atom",
        items: at("entry", n).map(s => {
          var l;
          let c = s.children,
            E = {
              media: z0(c)
            };
          se(E, "id", "id", c), se(E, "title", "title", c);
          let f = (l = Jt("link", c)) === null || l === void 0 ? void 0 : l.attribs.href;
          f && (E.link = f);
          let m = Me("summary", c) || Me("content", c);
          m && (E.description = m);
          let D = Me("updated", c);
          return D && (E.pubDate = new Date(D)), E;
        })
      };
    se(r, "id", "id", n), se(r, "title", "title", n);
    let a = (t = Jt("link", n)) === null || t === void 0 ? void 0 : t.attribs.href;
    a && (r.link = a), se(r, "description", "subtitle", n);
    let i = Me("updated", n);
    return i && (r.updated = new Date(i)), se(r, "author", "email", n, !0), r;
  }
  function Ga(e) {
    var t, n;
    let r = (n = (t = Jt("channel", e.children)) === null || t === void 0 ? void 0 : t.children) !== null && n !== void 0 ? n : [],
      a = {
        type: e.name.substr(0, 3),
        id: "",
        items: at("item", e.children).map(s => {
          let l = s.children,
            c = {
              media: z0(l)
            };
          se(c, "id", "guid", l), se(c, "title", "title", l), se(c, "link", "link", l), se(c, "description", "description", l);
          let E = Me("pubDate", l) || Me("dc:date", l);
          return E && (c.pubDate = new Date(E)), c;
        })
      };
    se(a, "title", "title", r), se(a, "link", "link", r), se(a, "description", "description", r);
    let i = Me("lastBuildDate", r);
    return i && (a.updated = new Date(i)), se(a, "author", "managingEditor", r, !0), a;
  }
  var Ka = ["url", "type", "lang"],
    Qa = ["fileSize", "bitrate", "framerate", "samplingrate", "channels", "duration", "height", "width"];
  function z0(e) {
    return at("media:content", e).map(t => {
      let n = t.attribs,
        r = {
          medium: n.medium,
          isDefault: !!n.isDefault
        };
      for (let a of Ka) n[a] && (r[a] = n[a]);
      for (let a of Qa) n[a] && (r[a] = parseInt(n[a], 10));
      return n.expression && (r.expression = n.expression), r;
    });
  }
  function Jt(e, t) {
    return at(e, t, !0, 1)[0];
  }
  function Me(e, t, n = !1) {
    return ut(at(e, t, n, 1)).trim();
  }
  function se(e, t, n, r, a = !1) {
    let i = Me(n, r, a);
    i && (e[t] = i);
  }
  function Ya(e) {
    return e === "rss" || e === "feed" || e === "rdf:RDF";
  }
  var Xa = {
    _useHtmlParser2: !1
  };
  function wn(e, t) {
    if (!e) return t != null ? t : Xa;
    let n = L(L({
      _useHtmlParser2: !!e.xmlMode
    }, t), e);
    return e.xml ? (n._useHtmlParser2 = !0, n.xmlMode = !0, e.xml !== !0 && Object.assign(n, e.xml)) : e.xmlMode && (n._useHtmlParser2 = !0), n;
  }
  function $0(e, t, n) {
    return e ? e(t != null ? t : e._root.children, null, void 0, n).toString() : "";
  }
  function Ja(e, t) {
    return !t && typeof e == "object" && e != null && !("length" in e) && !("type" in e);
  }
  function qa(e, t) {
    let n = Ja(e) ? (t = e, void 0) : e,
      r = L(L({}, this === null || this === void 0 ? void 0 : this._options), wn(t));
    return $0(this, n, r);
  }
  function Va(e) {
    let t = Ye(L({}, this._options), {
      xmlMode: !0
    });
    return $0(this, e, t);
  }
  function pt(e) {
    let t = e != null ? e : this ? this.root() : [],
      n = "";
    for (let r = 0; r < t.length; r++) n += ut(t[r]);
    return n;
  }
  function ja(e, t, n = typeof t == "boolean" ? t : !1) {
    if (!e || typeof e != "string") return null;
    typeof t == "boolean" && (n = t);
    let r = this.load(e, this._options, !1);
    return n || r("script").remove(), [...r.root()[0].children];
  }
  function Za() {
    return this(this._root);
  }
  function Rn(e, t) {
    if (t === e) return !1;
    let n = t;
    for (; n && n !== n.parent;) if (n = n.parent, n === e) return !0;
    return !1;
  }
  function za(e) {
    return this.root().extract(e);
  }
  function er(e, t) {
    if (!tr(e) || !tr(t)) return;
    let n = e.length,
      r = +t.length;
    for (let a = 0; a < r; a++) e[n++] = t[a];
    return e.length = n, e;
  }
  function tr(e) {
    if (Array.isArray(e)) return !0;
    if (typeof e != "object" || e === null || !("length" in e) || typeof e.length != "number" || e.length < 0) return !1;
    for (let t = 0; t < e.length; t++) if (!(t in e)) return !1;
    return !0;
  }
  var nr = {};
  le(nr, {
    addClass: () => Cr,
    attr: () => xi,
    data: () => gi,
    hasClass: () => _i,
    prop: () => Bi,
    removeAttr: () => Fi,
    removeClass: () => pr,
    toggleClass: () => fr,
    val: () => Ti
  });
  function be(e) {
    return e.cheerio != null;
  }
  function $a(e) {
    return e.replace(/[._-](\w|$)/g, (t, n) => n.toUpperCase());
  }
  function ei(e) {
    return e.replace(/[A-Z]/g, "-$&").toLowerCase();
  }
  function P(e, t) {
    let n = e.length;
    for (let r = 0; r < n; r++) t(e[r], r);
    return e;
  }
  var ze;
  (function (e) {
    e[e.LowerA = 97] = "LowerA", e[e.LowerZ = 122] = "LowerZ", e[e.UpperA = 65] = "UpperA", e[e.UpperZ = 90] = "UpperZ", e[e.Exclamation = 33] = "Exclamation";
  })(ze || (ze = {}));
  function On(e) {
    if (typeof e != "string") return !1;
    let t = e.indexOf("<");
    if (t === -1 || t > e.length - 3) return !1;
    let n = e.charCodeAt(t + 1);
    return (n >= ze.LowerA && n <= ze.LowerZ || n >= ze.UpperA && n <= ze.UpperZ || n === ze.Exclamation) && e.includes(">", t + 2);
  }
  var Ln,
    ti = new Map([[0, 65533], [128, 8364], [130, 8218], [131, 402], [132, 8222], [133, 8230], [134, 8224], [135, 8225], [136, 710], [137, 8240], [138, 352], [139, 8249], [140, 338], [142, 381], [145, 8216], [146, 8217], [147, 8220], [148, 8221], [149, 8226], [150, 8211], [151, 8212], [152, 732], [153, 8482], [154, 353], [155, 8250], [156, 339], [158, 382], [159, 376]]),
    rr = (Ln = String.fromCodePoint) !== null && Ln !== void 0 ? Ln : e => {
      let t = "";
      return e > 65535 && (e -= 65536, t += String.fromCharCode(e >>> 10 & 1023 | 55296), e = 56320 | e & 1023), t += String.fromCharCode(e), t;
    };
  function ni(e) {
    var t;
    return e >= 55296 && e <= 57343 || e > 1114111 ? 65533 : (t = ti.get(e)) !== null && t !== void 0 ? t : e;
  }
  function ur(e) {
    let t = typeof atob == "function" ? atob(e) : typeof Buffer.from == "function" ? Buffer.from(e, "base64").toString("binary") : new Buffer(e, "base64").toString("binary"),
      n = t.length & -2,
      r = new Uint16Array(n / 2);
    for (let a = 0, i = 0; a < n; a += 2) {
      let s = t.charCodeAt(a),
        l = t.charCodeAt(a + 1);
      r[i++] = s | l << 8;
    }
    return r;
  }
  var ri = ur("QR08ALkAAgH6AYsDNQR2BO0EPgXZBQEGLAbdBxMISQrvCmQLfQurDKQNLw4fD4YPpA+6D/IPAAAAAAAAAAAAAAAAKhBMEY8TmxUWF2EYLBkxGuAa3RsJHDscWR8YIC8jSCSIJcMl6ie3Ku8rEC0CLjoupS7kLgAIRU1hYmNmZ2xtbm9wcnN0dVQAWgBeAGUAaQBzAHcAfgCBAIQAhwCSAJoAoACsALMAbABpAGcAO4DGAMZAUAA7gCYAJkBjAHUAdABlADuAwQDBQHIiZXZlAAJhAAFpeW0AcgByAGMAO4DCAMJAEGRyAADgNdgE3XIAYQB2AGUAO4DAAMBA8CFoYZFj4SFjcgBhZAAAoFMqAAFncIsAjgBvAG4ABGFmAADgNdg43fAlbHlGdW5jdGlvbgCgYSBpAG4AZwA7gMUAxUAAAWNzpACoAHIAAOA12Jzc6SFnbgCgVCJpAGwAZABlADuAwwDDQG0AbAA7gMQAxEAABGFjZWZvcnN1xQDYANoA7QDxAPYA+QD8AAABY3LJAM8AayNzbGFzaAAAoBYidgHTANUAAKDnKmUAZAAAoAYjeQARZIABY3J0AOAA5QDrAGEidXNlAACgNSLuI291bGxpcwCgLCFhAJJjcgAA4DXYBd1wAGYAAOA12Dnd5SF2ZdhiYwDyAOoAbSJwZXEAAKBOIgAHSE9hY2RlZmhpbG9yc3UXARoBHwE6AVIBVQFiAWQBZgGCAakB6QHtAfIBYwB5ACdkUABZADuAqQCpQIABY3B5ACUBKAE1AfUhdGUGYWmg0iJ0KGFsRGlmZmVyZW50aWFsRAAAoEUhbCJleXMAAKAtIQACYWVpb0EBRAFKAU0B8iFvbgxhZABpAGwAO4DHAMdAcgBjAAhhbiJpbnQAAKAwIm8AdAAKYQABZG5ZAV0BaSJsbGEAuGB0I2VyRG90ALdg8gA5AWkAp2NyImNsZQAAAkRNUFRwAXQBeQF9AW8AdAAAoJkiaSJudXMAAKCWIuwhdXMAoJUiaSJtZXMAAKCXIm8AAAFjc4cBlAFrKndpc2VDb250b3VySW50ZWdyYWwAAKAyImUjQ3VybHkAAAFEUZwBpAFvJXVibGVRdW90ZQAAoB0gdSJvdGUAAKAZIAACbG5wdbABtgHNAdgBbwBuAGWgNyIAoHQqgAFnaXQAvAHBAcUB8iJ1ZW50AKBhIm4AdAAAoC8i7yV1ckludGVncmFsAKAuIgABZnLRAdMBAKACIe8iZHVjdACgECJuLnRlckNsb2Nrd2lzZUNvbnRvdXJJbnRlZ3JhbAAAoDMi7yFzcwCgLypjAHIAAOA12J7ccABDoNMiYQBwAACgTSKABURKU1phY2VmaW9zAAsCEgIVAhgCGwIsAjQCOQI9AnMCfwNvoEUh9CJyYWhkAKARKWMAeQACZGMAeQAFZGMAeQAPZIABZ3JzACECJQIoAuchZXIAoCEgcgAAoKEhaAB2AACg5CoAAWF5MAIzAvIhb24OYRRkbAB0oAciYQCUY3IAAOA12AfdAAFhZkECawIAAWNtRQJnAvIjaXRpY2FsAAJBREdUUAJUAl8CYwJjInV0ZQC0YG8AdAFZAloC2WJiJGxlQWN1dGUA3WJyImF2ZQBgYGkibGRlANxi7yFuZACgxCJmJWVyZW50aWFsRAAAoEYhcAR9AgAAAAAAAIECjgIAABoDZgAA4DXYO91EoagAhQKJAm8AdAAAoNwgcSJ1YWwAAKBQIuIhbGUAA0NETFJVVpkCqAK1Au8C/wIRA28AbgB0AG8AdQByAEkAbgB0AGUAZwByAGEA7ADEAW8AdAKvAgAAAACwAqhgbiNBcnJvdwAAoNMhAAFlb7kC0AJmAHQAgAFBUlQAwQLGAs0CciJyb3cAAKDQIekkZ2h0QXJyb3cAoNQhZQDlACsCbgBnAAABTFLWAugC5SFmdAABQVLcAuECciJyb3cAAKD4J+kkZ2h0QXJyb3cAoPon6SRnaHRBcnJvdwCg+SdpImdodAAAAUFU9gL7AnIicm93AACg0iFlAGUAAKCoInAAQQIGAwAAAAALA3Iicm93AACg0SFvJHduQXJyb3cAAKDVIWUlcnRpY2FsQmFyAACgJSJuAAADQUJMUlRhJAM2AzoDWgNxA3oDciJyb3cAAKGTIUJVLAMwA2EAcgAAoBMpcCNBcnJvdwAAoPUhciJldmUAEWPlIWZ00gJDAwAASwMAAFIDaSVnaHRWZWN0b3IAAKBQKWUkZVZlY3RvcgAAoF4p5SJjdG9yQqC9IWEAcgAAoFYpaSJnaHQA1AFiAwAAaQNlJGVWZWN0b3IAAKBfKeUiY3RvckKgwSFhAHIAAKBXKWUAZQBBoKQiciJyb3cAAKCnIXIAcgBvAPcAtAIAAWN0gwOHA3IAAOA12J/c8iFvaxBhAAhOVGFjZGZnbG1vcHFzdHV4owOlA6kDsAO/A8IDxgPNA9ID8gP9AwEEFAQeBCAEJQRHAEphSAA7gNAA0EBjAHUAdABlADuAyQDJQIABYWl5ALYDuQO+A/Ihb24aYXIAYwA7gMoAykAtZG8AdAAWYXIAAOA12AjdcgBhAHYAZQA7gMgAyEDlIm1lbnQAoAgiAAFhcNYD2QNjAHIAEmF0AHkAUwLhAwAAAADpA20lYWxsU3F1YXJlAACg+yVlJ3J5U21hbGxTcXVhcmUAAKCrJQABZ3D2A/kDbwBuABhhZgAA4DXYPN3zImlsb26VY3UAAAFhaQYEDgRsAFSgdSppImxkZQAAoEIi7CNpYnJpdW0AoMwhAAFjaRgEGwRyAACgMCFtAACgcyphAJdjbQBsADuAywDLQAABaXApBC0E8yF0cwCgAyLvJG5lbnRpYWxFAKBHIYACY2Zpb3MAPQQ/BEMEXQRyBHkAJGRyAADgNdgJ3WwibGVkAFMCTAQAAAAAVARtJWFsbFNxdWFyZQAAoPwlZSdyeVNtYWxsU3F1YXJlAACgqiVwA2UEAABpBAAAAABtBGYAAOA12D3dwSFsbACgACLyI2llcnRyZgCgMSFjAPIAcQQABkpUYWJjZGZnb3JzdIgEiwSOBJMElwSkBKcEqwStBLIE5QTqBGMAeQADZDuAPgA+QO0hbWFkoJMD3GNyImV2ZQAeYYABZWl5AJ0EoASjBOQhaWwiYXIAYwAcYRNkbwB0ACBhcgAA4DXYCt0AoNkicABmAADgNdg+3eUiYXRlcgADRUZHTFNUvwTIBM8E1QTZBOAEcSJ1YWwATKBlIuUhc3MAoNsidSRsbEVxdWFsAACgZyJyI2VhdGVyAACgoirlIXNzAKB3IuwkYW50RXF1YWwAoH4qaSJsZGUAAKBzImMAcgAA4DXYotwAoGsiAARBYWNmaW9zdfkE/QQFBQgFCwUTBSIFKwVSIkRjeQAqZAABY3QBBQQFZQBrAMdiXmDpIXJjJGFyAACgDCFsJWJlcnRTcGFjZQAAoAsh8AEYBQAAGwVmAACgDSHpJXpvbnRhbExpbmUAoAAlAAFjdCYFKAXyABIF8iFvayZhbQBwAEQBMQU5BW8AdwBuAEgAdQBtAPAAAAFxInVhbAAAoE8iAAdFSk9hY2RmZ21ub3N0dVMFVgVZBVwFYwVtBXAFcwV6BZAFtgXFBckFzQVjAHkAFWTsIWlnMmFjAHkAAWRjAHUAdABlADuAzQDNQAABaXlnBWwFcgBjADuAzgDOQBhkbwB0ADBhcgAAoBEhcgBhAHYAZQA7gMwAzEAAoREhYXB/BYsFAAFjZ4MFhQVyACphaSNuYXJ5SQAAoEghbABpAGUA8wD6AvQBlQUAAKUFZaAsIgABZ3KaBZ4F8iFhbACgKyLzI2VjdGlvbgCgwiJpI3NpYmxlAAABQ1SsBbEFbyJtbWEAAKBjIGkibWVzAACgYiCAAWdwdAC8Bb8FwwVvAG4ALmFmAADgNdhA3WEAmWNjAHIAAKAQIWkibGRlAChh6wHSBQAA1QVjAHkABmRsADuAzwDPQIACY2Zvc3UA4QXpBe0F8gX9BQABaXnlBegFcgBjADRhGWRyAADgNdgN3XAAZgAA4DXYQd3jAfcFAAD7BXIAAOA12KXc8iFjeQhk6yFjeQRkgANISmFjZm9zAAwGDwYSBhUGHQYhBiYGYwB5ACVkYwB5AAxk8CFwYZpjAAFleRkGHAbkIWlsNmEaZHIAAOA12A7dcABmAADgNdhC3WMAcgAA4DXYptyABUpUYWNlZmxtb3N0AD0GQAZDBl4GawZkB2gHcAd0B80H2gdjAHkACWQ7gDwAPECAAmNtbnByAEwGTwZSBlUGWwb1IXRlOWHiIWRhm2NnAACg6ifsI2FjZXRyZgCgEiFyAACgniGAAWFleQBkBmcGagbyIW9uPWHkIWlsO2EbZAABZnNvBjQHdAAABUFDREZSVFVWYXKABp4GpAbGBssG3AYDByEHwQIqBwABbnKEBowGZyVsZUJyYWNrZXQAAKDoJ/Ihb3cAoZAhQlKTBpcGYQByAACg5CHpJGdodEFycm93AKDGIWUjaWxpbmcAAKAII28A9QGqBgAAsgZiJWxlQnJhY2tldAAAoOYnbgDUAbcGAAC+BmUkZVZlY3RvcgAAoGEp5SJjdG9yQqDDIWEAcgAAoFkpbCJvb3IAAKAKI2kiZ2h0AAABQVbSBtcGciJyb3cAAKCUIeUiY3RvcgCgTikAAWVy4AbwBmUAAKGjIkFW5gbrBnIicm93AACgpCHlImN0b3IAoFopaSNhbmdsZQBCorIi+wYAAAAA/wZhAHIAAKDPKXEidWFsAACgtCJwAIABRFRWAAoHEQcYB+8kd25WZWN0b3IAoFEpZSRlVmVjdG9yAACgYCnlImN0b3JCoL8hYQByAACgWCnlImN0b3JCoLwhYQByAACgUilpAGcAaAB0AGEAcgByAG8A9wDMAnMAAANFRkdMU1Q/B0cHTgdUB1gHXwfxJXVhbEdyZWF0ZXIAoNoidSRsbEVxdWFsAACgZiJyI2VhdGVyAACgdiLlIXNzAKChKuwkYW50RXF1YWwAoH0qaSJsZGUAAKByInIAAOA12A/dZaDYIuYjdGFycm93AKDaIWkiZG90AD9hgAFucHcAege1B7kHZwAAAkxSbHKCB5QHmwerB+UhZnQAAUFSiAeNB3Iicm93AACg9SfpJGdodEFycm93AKD3J+kkZ2h0QXJyb3cAoPYn5SFmdAABYXLcAqEHaQBnAGgAdABhAHIAcgBvAPcA5wJpAGcAaAB0AGEAcgByAG8A9wDuAmYAAOA12EPdZQByAAABTFK/B8YHZSRmdEFycm93AACgmSHpJGdodEFycm93AKCYIYABY2h0ANMH1QfXB/IAWgYAoLAh8iFva0FhAKBqIgAEYWNlZmlvc3XpB+wH7gf/BwMICQgOCBEIcAAAoAUpeQAcZAABZGzyB/kHaSR1bVNwYWNlAACgXyBsI2ludHJmAACgMyFyAADgNdgQ3e4jdXNQbHVzAKATInAAZgAA4DXYRN1jAPIA/gecY4AESmFjZWZvc3R1ACEIJAgoCDUIgQiFCDsKQApHCmMAeQAKZGMidXRlAENhgAFhZXkALggxCDQI8iFvbkdh5CFpbEVhHWSAAWdzdwA7CGEIfQjhInRpdmWAAU1UVgBECEwIWQhlJWRpdW1TcGFjZQAAoAsgaABpAAABY25SCFMIawBTAHAAYQBjAOUASwhlAHIAeQBUAGgAaQDuAFQI9CFlZAABR0xnCHUIcgBlAGEAdABlAHIARwByAGUAYQB0AGUA8gDrBGUAcwBzAEwAZQBzAPMA2wdMImluZQAKYHIAAOA12BHdAAJCbnB0jAiRCJkInAhyImVhawAAoGAgwiZyZWFraW5nU3BhY2WgYGYAAKAVIUOq7CqzCMIIzQgAAOcIGwkAAAAAAAAtCQAAbwkAAIcJAACdCcAJGQoAADQKAAFvdbYIvAjuI2dydWVudACgYiJwIkNhcAAAoG0ibyh1YmxlVmVydGljYWxCYXIAAKAmIoABbHF4ANII1wjhCOUibWVudACgCSL1IWFsVKBgImkibGRlAADgQiI4A2kic3RzAACgBCJyI2VhdGVyAACjbyJFRkdMU1T1CPoIAgkJCQ0JFQlxInVhbAAAoHEidSRsbEVxdWFsAADgZyI4A3IjZWF0ZXIAAOBrIjgD5SFzcwCgeSLsJGFudEVxdWFsAOB+KjgDaSJsZGUAAKB1IvUhbXBEASAJJwnvI3duSHVtcADgTiI4A3EidWFsAADgTyI4A2UAAAFmczEJRgn0JFRyaWFuZ2xlQqLqIj0JAAAAAEIJYQByAADgzyk4A3EidWFsAACg7CJzAICibiJFR0xTVABRCVYJXAlhCWkJcSJ1YWwAAKBwInIjZWF0ZXIAAKB4IuUhc3MA4GoiOAPsJGFudEVxdWFsAOB9KjgDaSJsZGUAAKB0IuUic3RlZAABR0x1CX8J8iZlYXRlckdyZWF0ZXIA4KIqOAPlI3NzTGVzcwDgoSo4A/IjZWNlZGVzAKGAIkVTjwmVCXEidWFsAADgryo4A+wkYW50RXF1YWwAoOAiAAFlaaAJqQl2JmVyc2VFbGVtZW50AACgDCLnJWh0VHJpYW5nbGVCousitgkAAAAAuwlhAHIAAODQKTgDcSJ1YWwAAKDtIgABcXXDCeAJdSNhcmVTdQAAAWJwywnVCfMhZXRF4I8iOANxInVhbAAAoOIi5SJyc2V0ReCQIjgDcSJ1YWwAAKDjIoABYmNwAOYJ8AkNCvMhZXRF4IIi0iBxInVhbAAAoIgi4yJlZWRzgKGBIkVTVAD6CQAKBwpxInVhbAAA4LAqOAPsJGFudEVxdWFsAKDhImkibGRlAADgfyI4A+UicnNldEXggyLSIHEidWFsAACgiSJpImxkZQCAoUEiRUZUACIKJwouCnEidWFsAACgRCJ1JGxsRXF1YWwAAKBHImkibGRlAACgSSJlJXJ0aWNhbEJhcgAAoCQiYwByAADgNdip3GkAbABkAGUAO4DRANFAnWMAB0VhY2RmZ21vcHJzdHV2XgphCmgKcgp2CnoKgQqRCpYKqwqtCrsKyArNCuwhaWdSYWMAdQB0AGUAO4DTANNAAAFpeWwKcQpyAGMAO4DUANRAHmRiImxhYwBQYXIAAOA12BLdcgBhAHYAZQA7gNIA0kCAAWFlaQCHCooKjQpjAHIATGFnAGEAqWNjInJvbgCfY3AAZgAA4DXYRt3lI25DdXJseQABRFGeCqYKbyV1YmxlUXVvdGUAAKAcIHUib3RlAACgGCAAoFQqAAFjbLEKtQpyAADgNdiq3GEAcwBoADuA2ADYQGkAbAHACsUKZABlADuA1QDVQGUAcwAAoDcqbQBsADuA1gDWQGUAcgAAAUJQ0wrmCgABYXLXCtoKcgAAoD4gYQBjAAABZWvgCuIKAKDeI2UAdAAAoLQjYSVyZW50aGVzaXMAAKDcI4AEYWNmaGlsb3JzAP0KAwsFCwkLCwsMCxELIwtaC3IjdGlhbEQAAKACInkAH2RyAADgNdgT3WkApmOgY/Ujc01pbnVzsWAAAWlwFQsgC24AYwBhAHIAZQBwAGwAYQBuAOUACgVmAACgGSGAobsqZWlvACoLRQtJC+MiZWRlc4CheiJFU1QANAs5C0ALcSJ1YWwAAKCvKuwkYW50RXF1YWwAoHwiaSJsZGUAAKB+Im0AZQAAoDMgAAFkcE0LUQv1IWN0AKAPIm8jcnRpb24AYaA3ImwAAKAdIgABY2leC2ILcgAA4DXYq9yoYwACVWZvc2oLbwtzC3cLTwBUADuAIgAiQHIAAOA12BTdcABmAACgGiFjAHIAAOA12KzcAAZCRWFjZWZoaW9yc3WPC5MLlwupC7YL2AvbC90LhQyTDJoMowzhIXJyAKAQKUcAO4CuAK5AgAFjbnIAnQugC6ML9SF0ZVRhZwAAoOsncgB0oKAhbAAAoBYpgAFhZXkArwuyC7UL8iFvblhh5CFpbFZhIGR2oBwhZSJyc2UAAAFFVb8LzwsAAWxxwwvIC+UibWVudACgCyL1JGlsaWJyaXVtAKDLIXAmRXF1aWxpYnJpdW0AAKBvKXIAAKAcIW8AoWPnIWh0AARBQ0RGVFVWYewLCgwQDDIMNwxeDHwM9gIAAW5y8Av4C2clbGVCcmFja2V0AACg6SfyIW93AKGSIUJM/wsDDGEAcgAAoOUhZSRmdEFycm93AACgxCFlI2lsaW5nAACgCSNvAPUBFgwAAB4MYiVsZUJyYWNrZXQAAKDnJ24A1AEjDAAAKgxlJGVWZWN0b3IAAKBdKeUiY3RvckKgwiFhAHIAAKBVKWwib29yAACgCyMAAWVyOwxLDGUAAKGiIkFWQQxGDHIicm93AACgpiHlImN0b3IAoFspaSNhbmdsZQBCorMiVgwAAAAAWgxhAHIAAKDQKXEidWFsAACgtSJwAIABRFRWAGUMbAxzDO8kd25WZWN0b3IAoE8pZSRlVmVjdG9yAACgXCnlImN0b3JCoL4hYQByAACgVCnlImN0b3JCoMAhYQByAACgUykAAXB1iQyMDGYAAKAdIe4kZEltcGxpZXMAoHAp6SRnaHRhcnJvdwCg2yEAAWNongyhDHIAAKAbIQCgsSHsJGVEZWxheWVkAKD0KYAGSE9hY2ZoaW1vcXN0dQC/DMgMzAzQDOIM5gwKDQ0NFA0ZDU8NVA1YDQABQ2PDDMYMyCFjeSlkeQAoZEYiVGN5ACxkYyJ1dGUAWmEAorwqYWVpedgM2wzeDOEM8iFvbmBh5CFpbF5hcgBjAFxhIWRyAADgNdgW3e8hcnQAAkRMUlXvDPYM/QwEDW8kd25BcnJvdwAAoJMhZSRmdEFycm93AACgkCHpJGdodEFycm93AKCSIXAjQXJyb3cAAKCRIechbWGjY+EkbGxDaXJjbGUAoBgicABmAADgNdhK3XICHw0AAAAAIg10AACgGiLhIXJlgKGhJUlTVQAqDTINSg3uJXRlcnNlY3Rpb24AoJMidQAAAWJwNw1ADfMhZXRFoI8icSJ1YWwAAKCRIuUicnNldEWgkCJxInVhbAAAoJIibiJpb24AAKCUImMAcgAA4DXYrtxhAHIAAKDGIgACYmNtcF8Nag2ODZANc6DQImUAdABFoNAicSJ1YWwAAKCGIgABY2huDYkNZSJlZHMAgKF7IkVTVAB4DX0NhA1xInVhbAAAoLAq7CRhbnRFcXVhbACgfSJpImxkZQAAoH8iVABoAGEA9ADHCwCgESIAodEiZXOVDZ8NciJzZXQARaCDInEidWFsAACghyJlAHQAAKDRIoAFSFJTYWNmaGlvcnMAtQ27Db8NyA3ODdsN3w3+DRgOHQ4jDk8AUgBOADuA3gDeQMEhREUAoCIhAAFIY8MNxg1jAHkAC2R5ACZkAAFidcwNzQ0JYKRjgAFhZXkA1A3XDdoN8iFvbmRh5CFpbGJhImRyAADgNdgX3QABZWnjDe4N8gHoDQAA7Q3lImZvcmUAoDQiYQCYYwABY27yDfkNayNTcGFjZQAA4F8gCiDTInBhY2UAoAkg7CFkZYChPCJFRlQABw4MDhMOcSJ1YWwAAKBDInUkbGxFcXVhbAAAoEUiaSJsZGUAAKBIInAAZgAA4DXYS93pI3BsZURvdACg2yAAAWN0Jw4rDnIAAOA12K/c8iFva2Zh4QpFDlYOYA5qDgAAbg5yDgAAAAAAAAAAAAB5DnwOqA6zDgAADg8RDxYPGg8AAWNySA5ODnUAdABlADuA2gDaQHIAb6CfIeMhaXIAoEkpcgDjAVsOAABdDnkADmR2AGUAbGEAAWl5Yw5oDnIAYwA7gNsA20AjZGIibGFjAHBhcgAA4DXYGN1yAGEAdgBlADuA2QDZQOEhY3JqYQABZGl/Dp8OZQByAAABQlCFDpcOAAFhcokOiw5yAF9gYQBjAAABZWuRDpMOAKDfI2UAdAAAoLUjYSVyZW50aGVzaXMAAKDdI28AbgBQoMMi7CF1cwCgjiIAAWdwqw6uDm8AbgByYWYAAOA12EzdAARBREVUYWRwc78O0g7ZDuEOBQPqDvMOBw9yInJvdwDCoZEhyA4AAMwOYQByAACgEilvJHduQXJyb3cAAKDFIW8kd25BcnJvdwAAoJUhcSV1aWxpYnJpdW0AAKBuKWUAZQBBoKUiciJyb3cAAKClIW8AdwBuAGEAcgByAG8A9wAQA2UAcgAAAUxS+Q4AD2UkZnRBcnJvdwAAoJYh6SRnaHRBcnJvdwCglyFpAGyg0gNvAG4ApWPpIW5nbmFjAHIAAOA12LDcaSJsZGUAaGFtAGwAO4DcANxAgAREYmNkZWZvc3YALQ8xDzUPNw89D3IPdg97D4AP4SFzaACgqyJhAHIAAKDrKnkAEmThIXNobKCpIgCg5ioAAWVyQQ9DDwCgwSKAAWJ0eQBJD00Paw9hAHIAAKAWIGmgFiDjIWFsAAJCTFNUWA9cD18PZg9hAHIAAKAjIukhbmV8YGUkcGFyYXRvcgAAoFgnaSJsZGUAAKBAItQkaGluU3BhY2UAoAogcgAA4DXYGd1wAGYAAOA12E3dYwByAADgNdix3GQiYXNoAACgqiKAAmNlZm9zAI4PkQ+VD5kPng/pIXJjdGHkIWdlAKDAInIAAOA12BrdcABmAADgNdhO3WMAcgAA4DXYstwAAmZpb3OqD64Prw+0D3IAAOA12BvdnmNwAGYAAOA12E/dYwByAADgNdiz3IAEQUlVYWNmb3N1AMgPyw/OD9EP2A/gD+QP6Q/uD2MAeQAvZGMAeQAHZGMAeQAuZGMAdQB0AGUAO4DdAN1AAAFpedwP3w9yAGMAdmErZHIAAOA12BzdcABmAADgNdhQ3WMAcgAA4DXYtNxtAGwAeGEABEhhY2RlZm9z/g8BEAUQDRAQEB0QIBAkEGMAeQAWZGMidXRlAHlhAAFheQkQDBDyIW9ufWEXZG8AdAB7YfIBFRAAABwQbwBXAGkAZAB0AOgAVAhhAJZjcgAAoCghcABmAACgJCFjAHIAAOA12LXc4QtCEEkQTRAAAGcQbRByEAAAAAAAAAAAeRCKEJcQ8hD9EAAAGxEhETIROREAAD4RYwB1AHQAZQA7gOEA4UByImV2ZQADYYCiPiJFZGl1eQBWEFkQWxBgEGUQAOA+IjMDAKA/InIAYwA7gOIA4kB0AGUAO4C0ALRAMGRsAGkAZwA7gOYA5kByoGEgAOA12B7dcgBhAHYAZQA7gOAA4EAAAWVwfBCGEAABZnCAEIQQ8yF5bQCgNSHoAIMQaABhALFjAAFhcI0QWwAAAWNskRCTEHIAAWFnAACgPypkApwQAAAAALEQAKInImFkc3ajEKcQqRCuEG4AZAAAoFUqAKBcKmwib3BlAACgWCoAoFoqAKMgImVsbXJzersQvRDAEN0Q5RDtEACgpCllAACgICJzAGQAYaAhImEEzhDQENIQ1BDWENgQ2hDcEACgqCkAoKkpAKCqKQCgqykAoKwpAKCtKQCgrikAoK8pdAB2oB8iYgBkoL4iAKCdKQABcHTpEOwQaAAAoCIixWDhIXJyAKB8IwABZ3D1EPgQbwBuAAVhZgAA4DXYUt0Ao0giRWFlaW9wBxEJEQ0RDxESERQRAKBwKuMhaXIAoG8qAKBKImQAAKBLInMAJ2DyIW94ZaBIIvEADhFpAG4AZwA7gOUA5UCAAWN0eQAmESoRKxFyAADgNdi23CpgbQBwAGWgSCLxAPgBaQBsAGQAZQA7gOMA40BtAGwAO4DkAORAAAFjaUERRxFvAG4AaQBuAPQA6AFuAHQAAKARKgAITmFiY2RlZmlrbG5vcHJzdWQRaBGXEZ8RpxGrEdIR1hErEjASexKKEn0RThNbE3oTbwB0AACg7SoAAWNybBGJEWsAAAJjZXBzdBF4EX0RghHvIW5nAKBMInAjc2lsb24A9mNyImltZQAAoDUgaQBtAGWgPSJxAACgzSJ2AY0RkRFlAGUAAKC9ImUAZABnoAUjZQAAoAUjcgBrAHSgtSPiIXJrAKC2IwABb3mjEaYRbgDnAHcRMWTxIXVvAKAeIIACY21wcnQAtBG5Eb4RwRHFEeEhdXPloDUi5ABwInR5dgAAoLApcwDpAH0RbgBvAPUA6gCAAWFodwDLEcwRzhGyYwCgNiHlIWVuAKBsInIAAOA12B/dZwCAA2Nvc3R1dncA4xHyEQUSEhIhEiYSKRKAAWFpdQDpEesR7xHwAKMFcgBjAACg7yVwAACgwyKAAWRwdAD4EfwRABJvAHQAAKAAKuwhdXMAoAEqaSJtZXMAAKACKnECCxIAAAAADxLjIXVwAKAGKmEAcgAAoAUm8iNpYW5nbGUAAWR1GhIeEu8hd24AoL0lcAAAoLMlcCJsdXMAAKAEKmUA5QBCD+UAkg9hInJvdwAAoA0pgAFha28ANhJoEncSAAFjbjoSZRJrAIABbHN0AEESRxJNEm8jemVuZ2UAAKDrKXEAdQBhAHIA5QBcBPIjaWFuZ2xlgKG0JWRscgBYElwSYBLvIXduAKC+JeUhZnQAoMIlaSJnaHQAAKC4JWsAAKAjJLEBbRIAAHUSsgFxEgAAcxIAoJIlAKCRJTQAAKCTJWMAawAAoIglAAFlb38ShxJx4D0A5SD1IWl2AOBhIuUgdAAAoBAjAAJwdHd4kRKVEpsSnxJmAADgNdhT3XSgpSJvAG0AAKClIvQhaWUAoMgiAAZESFVWYmRobXB0dXayEsES0RLgEvcS+xIKExoTHxMjEygTNxMAAkxSbHK5ErsSvRK/EgCgVyUAoFQlAKBWJQCgUyUAolAlRFVkdckSyxLNEs8SAKBmJQCgaSUAoGQlAKBnJQACTFJsctgS2hLcEt4SAKBdJQCgWiUAoFwlAKBZJQCjUSVITFJobHLrEu0S7xLxEvMS9RIAoGwlAKBjJQCgYCUAoGslAKBiJQCgXyVvAHgAAKDJKQACTFJscgITBBMGEwgTAKBVJQCgUiUAoBAlAKAMJQCiACVEVWR1EhMUExYTGBMAoGUlAKBoJQCgLCUAoDQlaSJudXMAAKCfIuwhdXMAoJ4iaSJtZXMAAKCgIgACTFJsci8TMRMzEzUTAKBbJQCgWCUAoBglAKAUJQCjAiVITFJobHJCE0QTRhNIE0oTTBMAoGolAKBhJQCgXiUAoDwlAKAkJQCgHCUAAWV2UhNVE3YA5QD5AGIAYQByADuApgCmQAACY2Vpb2ITZhNqE24TcgAA4DXYt9xtAGkAAKBPIG0A5aA9IogRbAAAoVwAYmh0E3YTAKDFKfMhdWIAoMgnbAF+E4QTbABloCIgdAAAoCIgcAAAoU4iRWWJE4sTAKCuKvGgTyI8BeEMqRMAAN8TABQDFB8UAAAjFDQUAAAAAIUUAAAAAI0UAAAAANcU4xT3FPsUAACIFQAAlhWAAWNwcgCuE7ET1RP1IXRlB2GAoikiYWJjZHMAuxO/E8QTzhPSE24AZAAAoEQqciJjdXAAAKBJKgABYXXIE8sTcAAAoEsqcAAAoEcqbwB0AACgQCoA4CkiAP4AAWVv2RPcE3QAAKBBIO4ABAUAAmFlaXXlE+8T9RP4E/AB6hMAAO0TcwAAoE0qbwBuAA1hZABpAGwAO4DnAOdAcgBjAAlhcABzAHOgTCptAACgUCpvAHQAC2GAAWRtbgAIFA0UEhRpAGwAO4C4ALhAcCJ0eXYAAKCyKXQAAIGiADtlGBQZFKJAcgBkAG8A9ABiAXIAAOA12CDdgAFjZWkAKBQqFDIUeQBHZGMAawBtoBMn4SFyawCgEyfHY3IAAKPLJUVjZWZtcz8UQRRHFHcUfBSAFACgwykAocYCZWxGFEkUcQAAoFciZQBhAlAUAAAAAGAUciJyb3cAAAFsclYUWhTlIWZ0AKC6IWkiZ2h0AACguyGAAlJTYWNkAGgUaRRrFG8UcxSuYACgyCRzAHQAAKCbIukhcmMAoJoi4SFzaACgnSJuImludAAAoBAqaQBkAACg7yrjIWlyAKDCKfUhYnN1oGMmaQB0AACgYybsApMUmhS2FAAAwxRvAG4AZaA6APGgVCKrAG0CnxQAAAAAoxRhAHSgLABAYAChASJmbKcUqRTuABMNZQAAAW14rhSyFOUhbnQAoAEiZQDzANIB5wG6FAAAwBRkoEUibwB0AACgbSpuAPQAzAGAAWZyeQDIFMsUzhQA4DXYVN1vAOQA1wEAgakAO3MeAdMUcgAAoBchAAFhb9oU3hRyAHIAAKC1IXMAcwAAoBcnAAFjdeYU6hRyAADgNdi43AABYnDuFPIUZaDPKgCg0SploNAqAKDSKuQhb3QAoO8igANkZWxwcnZ3AAYVEBUbFSEVRBVlFYQV4SFycgABbHIMFQ4VAKA4KQCgNSlwAhYVAAAAABkVcgAAoN4iYwAAoN8i4SFycnCgtiEAoD0pgKIqImJjZG9zACsVMBU6FT4VQRVyImNhcAAAoEgqAAFhdTQVNxVwAACgRipwAACgSipvAHQAAKCNInIAAKBFKgDgKiIA/gACYWxydksVURVuFXMVcgByAG2gtyEAoDwpeQCAAWV2dwBYFWUVaRVxAHACXxUAAAAAYxVyAGUA4wAXFXUA4wAZFWUAZQAAoM4iZSJkZ2UAAKDPImUAbgA7gKQApEBlI2Fycm93AAABbHJ7FX8V5SFmdACgtiFpImdodAAAoLchZQDkAG0VAAFjaYsVkRVvAG4AaQBuAPQAkwFuAHQAAKAxImwiY3R5AACgLSOACUFIYWJjZGVmaGlqbG9yc3R1d3oAuBW7Fb8V1RXgFegV+RUKFhUWHxZUFlcWZRbFFtsW7xb7FgUXChdyAPIAtAJhAHIAAKBlKQACZ2xyc8YVyhXOFdAV5yFlcgCgICDlIXRoAKA4IfIA9QxoAHagECAAoKMiawHZFd4VYSJyb3cAAKAPKWEA4wBfAgABYXnkFecV8iFvbg9hNGQAoUYhYW/tFfQVAAFnciEC8RVyAACgyiF0InNlcQAAoHcqgAFnbG0A/xUCFgUWO4CwALBAdABhALRjcCJ0eXYAAKCxKQABaXIOFhIW8yFodACgfykA4DXYId1hAHIAAAFschsWHRYAoMMhAKDCIYACYWVnc3YAKBauAjYWOhY+Fm0AAKHEIm9zLhY0Fm4AZABzoMQi9SFpdACgZiZhIm1tYQDdY2kAbgAAoPIiAKH3AGlvQxZRFmQAZQAAgfcAO29KFksW90BuI3RpbWVzAACgxyJuAPgAUBZjAHkAUmRjAG8CXhYAAAAAYhZyAG4AAKAeI28AcAAAoA0jgAJscHR1dwBuFnEWdRaSFp4W7CFhciRgZgAA4DXYVd0AotkCZW1wc30WhBaJFo0WcQBkoFAibwB0AACgUSJpIm51cwAAoDgi7CF1cwCgFCLxInVhcmUAoKEiYgBsAGUAYgBhAHIAdwBlAGQAZwDlANcAbgCAAWFkaAClFqoWtBZyAHIAbwD3APUMbwB3AG4AYQByAHIAbwB3APMA8xVhI3Jwb29uAAABbHK8FsAWZQBmAPQAHBZpAGcAaAD0AB4WYgHJFs8WawBhAHIAbwD3AJILbwLUFgAAAADYFnIAbgAAoB8jbwBwAACgDCOAAWNvdADhFukW7BYAAXJ55RboFgDgNdi53FVkbAAAoPYp8iFvaxFhAAFkcvMW9xZvAHQAAKDxImkA5qC/JVsSAAFhaP8WAhdyAPIANQNhAPIA1wvhIm5nbGUAoKYpAAFjaQ4XEBd5AF9k5yJyYXJyAKD/JwAJRGFjZGVmZ2xtbm9wcXJzdHV4MRc4F0YXWxcyBF4XaRd5F40XrBe0F78X2RcVGCEYLRg1GEAYAAFEbzUXgRZvAPQA+BUAAWNzPBdCF3UAdABlADuA6QDpQPQhZXIAoG4qAAJhaW95TRdQF1YXWhfyIW9uG2FyAGOgViI7gOoA6kDsIW9uAKBVIk1kbwB0ABdhAAFEcmIXZhdvAHQAAKBSIgDgNdgi3XKhmipuF3QXYQB2AGUAO4DoAOhAZKCWKm8AdAAAoJgqgKGZKmlscwCAF4UXhxfuInRlcnMAoOcjAKATIWSglSpvAHQAAKCXKoABYXBzAJMXlheiF2MAcgATYXQAeQBzogUinxcAAAAAoRdlAHQAAKAFInAAMaADIDMBqRerFwCgBCAAoAUgAAFnc7AXsRdLYXAAAKACIAABZ3C4F7sXbwBuABlhZgAA4DXYVt2AAWFscwDFF8sXzxdyAHOg1SJsAACg4yl1AHMAAKBxKmkAAKG1A2x21RfYF28AbgC1Y/VjAAJjc3V24BfoF/0XEBgAAWlv5BdWF3IAYwAAoFYiaQLuFwAAAADwF+0ADQThIW50AAFnbPUX+Rd0AHIAAKCWKuUhc3MAoJUqgAFhZWkAAxgGGAoYbABzAD1gcwB0AACgXyJ2AESgYSJEAACgeCrwImFyc2wAoOUpAAFEYRkYHRhvAHQAAKBTInIAcgAAoHEpgAFjZGkAJxgqGO0XcgAAoC8hbwD0AIwCAAFhaDEYMhi3YzuA8ADwQAABbXI5GD0YbAA7gOsA60BvAACgrCCAAWNpcABGGEgYSxhsACFgcwD0ACwEAAFlb08YVxhjAHQAYQB0AGkAbwDuABoEbgBlAG4AdABpAGEAbADlADME4Ql1GAAAgRgAAIMYiBgAAAAAoRilGAAAqhgAALsYvhjRGAAA1xgnGWwAbABpAG4AZwBkAG8AdABzAGUA8QBlF3kARGRtImFsZQAAoEAmgAFpbHIAjRiRGJ0Y7CFpZwCgA/tpApcYAAAAAJoYZwAAoAD7aQBnAACgBPsA4DXYI93sIWlnAKAB++whaWcA4GYAagCAAWFsdACvGLIYthh0AACgbSZpAGcAAKAC+24AcwAAoLElbwBmAJJh8AHCGAAAxhhmAADgNdhX3QABYWvJGMwYbADsAGsEdqDUIgCg2SphI3J0aW50AACgDSoAAWFv2hgiGQABY3PeGB8ZsQPnGP0YBRkSGRUZAAAdGbID7xjyGPQY9xj5GAAA+xg7gL0AvUAAoFMhO4C8ALxAAKBVIQCgWSEAoFshswEBGQAAAxkAoFQhAKBWIbQCCxkOGQAAAAAQGTuAvgC+QACgVyEAoFwhNQAAoFghtgEZGQAAGxkAoFohAKBdITgAAKBeIWwAAKBEIHcAbgAAoCIjYwByAADgNdi73IAIRWFiY2RlZmdpamxub3JzdHYARhlKGVoZXhlmGWkZkhmWGZkZnRmgGa0ZxhnLGc8Z4BkjGmygZyIAoIwqgAFjbXAAUBlTGVgZ9SF0ZfVhbQBhAOSgswM6FgCghipyImV2ZQAfYQABaXliGWUZcgBjAB1hM2RvAHQAIWGAoWUibHFzAMYEcBl6GfGhZSLOBAAAdhlsAGEAbgD0AN8EgKF+KmNkbACBGYQZjBljAACgqSpvAHQAb6CAKmyggioAoIQqZeDbIgD+cwAAoJQqcgAA4DXYJN3noGsirATtIWVsAKA3IWMAeQBTZIChdyJFYWoApxmpGasZAKCSKgCgpSoAoKQqAAJFYWVztBm2Gb0ZwhkAoGkicABwoIoq8iFveACgiipxoIgq8aCIKrUZaQBtAACg5yJwAGYAAOA12FjdYQB2AOUAYwIAAWNp0xnWGXIAAKAKIW0AAKFzImVs3BneGQCgjioAoJAqAIM+ADtjZGxxco0E6xn0GfgZ/BkBGgABY2nvGfEZAKCnKnIAAKB6Km8AdAAAoNci0CFhcgCglSl1ImVzdAAAoHwqgAJhZGVscwAKGvQZFhrVBCAa8AEPGgAAFBpwAHIAbwD4AFkZcgAAoHgpcQAAAWxxxAQbGmwAZQBzAPMASRlpAO0A5AQAAWVuJxouGnIjdG5lcXEAAOBpIgD+xQAsGgAFQWFiY2Vma29zeUAaQxpmGmoabRqDGocalhrCGtMacgDyAMwCAAJpbG1yShpOGlAaVBpyAHMA8ABxD2YAvWBpAGwA9AASBQABZHJYGlsaYwB5AEpkAKGUIWN3YBpkGmkAcgAAoEgpAKCtIWEAcgAAoA8h6SFyYyVhgAFhbHIAcxp7Gn8a8iF0c3WgZSZpAHQAAKBlJuwhaXAAoCYg4yFvbgCguSJyAADgNdgl3XMAAAFld4wakRphInJvdwAAoCUpYSJyb3cAAKAmKYACYW1vcHIAnxqjGqcauhq+GnIAcgAAoP8h9CFodACgOyJrAAABbHKsGrMaZSRmdGFycm93AACgqSHpJGdodGFycm93AKCqIWYAAOA12Fnd4iFhcgCgFSCAAWNsdADIGswa0BpyAADgNdi93GEAcwDoAGka8iFvaydhAAFicNca2xr1IWxsAKBDIOghZW4AoBAg4Qr2GgAA/RoAAAgbExsaGwAAIRs7GwAAAAA+G2IbmRuVG6sbAACyG80b0htjAHUAdABlADuA7QDtQAChYyBpeQEbBhtyAGMAO4DuAO5AOGQAAWN4CxsNG3kANWRjAGwAO4ChAKFAAAFmcssCFhsA4DXYJt1yAGEAdgBlADuA7ADsQIChSCFpbm8AJxsyGzYbAAFpbisbLxtuAHQAAKAMKnQAAKAtIuYhaW4AoNwpdABhAACgKSHsIWlnM2GAAWFvcABDG1sbXhuAAWNndABJG0sbWRtyACthgAFlbHAAcQVRG1UbaQBuAOUAyAVhAHIA9AByBWgAMWFmAACgtyJlAGQAtWEAoggiY2ZvdGkbbRt1G3kb4SFyZQCgBSFpAG4AdKAeImkAZQAAoN0pZABvAPQAWxsAoisiY2VscIEbhRuPG5QbYQBsAACguiIAAWdyiRuNG2UAcgDzACMQ4wCCG2EicmhrAACgFyryIW9kAKA8KgACY2dwdJ8boRukG6gbeQBRZG8AbgAvYWYAAOA12FrdYQC5Y3UAZQBzAHQAO4C/AL9AAAFjabUbuRtyAADgNdi+3G4AAKIIIkVkc3bCG8QbyBvQAwCg+SJvAHQAAKD1Inag9CIAoPMiaaBiIOwhZGUpYesB1hsAANkbYwB5AFZkbAA7gO8A70AAA2NmbW9zdeYb7hvyG/Ub+hsFHAABaXnqG+0bcgBjADVhOWRyAADgNdgn3eEhdGg3YnAAZgAA4DXYW93jAf8bAAADHHIAAOA12L/c8iFjeVhk6yFjeVRkAARhY2ZnaGpvcxUcGhwiHCYcKhwtHDAcNRzwIXBhdqC6A/BjAAFleR4cIRzkIWlsN2E6ZHIAAOA12CjdciJlZW4AOGFjAHkARWRjAHkAXGRwAGYAAOA12FzdYwByAADgNdjA3IALQUJFSGFiY2RlZmdoamxtbm9wcnN0dXYAXhxtHHEcdRx5HN8cBx0dHTwd3B3tHfEdAR4EHh0eLB5FHrwewx7hHgkfPR9LH4ABYXJ0AGQcZxxpHHIA8gBvB/IAxQLhIWlsAKAbKeEhcnIAoA4pZ6BmIgCgiyphAHIAAKBiKWMJjRwAAJAcAACVHAAAAAAAAAAAAACZHJwcAACmHKgcrRwAANIc9SF0ZTph7SJwdHl2AKC0KXIAYQDuAFoG4iFkYbtjZwAAoegnZGyhHKMcAKCRKeUAiwYAoIUqdQBvADuAqwCrQHIAgKOQIWJmaGxwc3QAuhy/HMIcxBzHHMoczhxmoOQhcwAAoB8pcwAAoB0p6wCyGnAAAKCrIWwAAKA5KWkAbQAAoHMpbAAAoKIhAKGrKmFl1hzaHGkAbAAAoBkpc6CtKgDgrSoA/oABYWJyAOUc6RztHHIAcgAAoAwpcgBrAACgcicAAWFr8Rz4HGMAAAFla/Yc9xx7YFtgAAFlc/wc/hwAoIspbAAAAWR1Ax0FHQCgjykAoI0pAAJhZXV5Dh0RHRodHB3yIW9uPmEAAWRpFR0YHWkAbAA8YewAowbiAPccO2QAAmNxcnMkHScdLB05HWEAAKA2KXUAbwDyoBwgqhEAAWR1MB00HeghYXIAoGcpcyJoYXIAAKBLKWgAAKCyIQCiZCJmZ3FzRB1FB5Qdnh10AIACYWhscnQATh1WHWUdbB2NHXIicm93AHSgkCFhAOkAzxxhI3Jwb29uAAABZHVeHWId7yF3bgCgvSFwAACgvCHlJGZ0YXJyb3dzAKDHIWkiZ2h0AIABYWhzAHUdex2DHXIicm93APOglCGdBmEAcgBwAG8AbwBuAPMAzgtxAHUAaQBnAGEAcgByAG8A9wBlGugkcmVldGltZXMAoMsi8aFkIk0HAACaHWwAYQBuAPQAXgcAon0qY2Rnc6YdqR2xHbcdYwAAoKgqbwB0AG+gfypyoIEqAKCDKmXg2iIA/nMAAKCTKoACYWRlZ3MAwB3GHcod1h3ZHXAAcAByAG8A+ACmHG8AdAAAoNYicQAAAWdxzx3SHXQA8gBGB2cAdADyAHQcdADyAFMHaQDtAGMHgAFpbHIA4h3mHeod8yFodACgfClvAG8A8gDKBgDgNdgp3UWgdiIAoJEqYQH1Hf4dcgAAAWR1YB35HWygvCEAoGopbABrAACghCVjAHkAWWQAomoiYWNodAweDx4VHhkecgDyAGsdbwByAG4AZQDyAGAW4SFyZACgaylyAGkAAKD6JQABaW8hHiQe5CFvdEBh9SFzdGGgsCPjIWhlAKCwIwACRWFlczMeNR48HkEeAKBoInAAcKCJKvIhb3gAoIkqcaCHKvGghyo0HmkAbQAAoOYiAARhYm5vcHR3elIeXB5fHoUelh6mHqsetB4AAW5yVh5ZHmcAAKDsJ3IAAKD9IXIA6wCwBmcAgAFsbXIAZh52Hnse5SFmdAABYXKIB2weaQBnAGgAdABhAHIAcgBvAPcAkwfhInBzdG8AoPwnaQBnAGgAdABhAHIAcgBvAPcAmgdwI2Fycm93AAABbHKNHpEeZQBmAPQAxhxpImdodAAAoKwhgAFhZmwAnB6fHqIecgAAoIUpAOA12F3ddQBzAACgLSppIm1lcwAAoDQqYQGvHrMecwB0AACgFyLhAIoOZaHKJbkeRhLuIWdlAKDKJWEAcgBsoCgAdAAAoJMpgAJhY2htdADMHs8e1R7bHt0ecgDyAJ0GbwByAG4AZQDyANYWYQByAGSgyyEAoG0pAKAOIHIAaQAAoL8iAANhY2hpcXTrHu8e1QfzHv0eBh/xIXVvAKA5IHIAAOA12MHcbQDloXIi+h4AAPweAKCNKgCgjyoAAWJ19xwBH28AcqAYIACgGiDyIW9rQmEAhDwAO2NkaGlscXJCBhcfxh0gHyQfKB8sHzEfAAFjaRsfHR8AoKYqcgAAoHkqcgBlAOUAkx3tIWVzAKDJIuEhcnIAoHYpdSJlc3QAAKB7KgABUGk1HzkfYQByAACglillocMlAgdfEnIAAAFkdUIfRx9zImhhcgAAoEop6CFhcgCgZikAAWVuTx9WH3IjdG5lcXEAAOBoIgD+xQBUHwAHRGFjZGVmaGlsbm9wc3VuH3Ifoh+rH68ftx+7H74f5h/uH/MfBwj/HwsgxCFvdACgOiIAAmNscHJ5H30fiR+eH3IAO4CvAK9AAAFldIEfgx8AoEImZaAgJ3MAZQAAoCAnc6CmIXQAbwCAoaYhZGx1AJQfmB+cH28AdwDuAHkDZQBmAPQA6gbwAOkO6yFlcgCgriUAAW95ph+qH+0hbWEAoCkqPGThIXNoAKAUIOElc3VyZWRhbmdsZQCgISJyAADgNdgq3W8AAKAnIYABY2RuAMQfyR/bH3IAbwA7gLUAtUBhoiMi0B8AANMf1x9zAPQAKxFpAHIAAKDwKm8AdAA7gLcAt0B1AHMA4qESIh4TAADjH3WgOCIAoCoqYwHqH+0fcAAAoNsq8gB+GnAAbAB1APMACAgAAWRw9x/7H+UhbHMAoKciZgAA4DXYXt0AAWN0AyAHIHIAAOA12MLc8CFvcwCgPiJsobwDECAVIPQiaW1hcACguCJhAPAAEyAADEdMUlZhYmNkZWZnaGlqbG1vcHJzdHV2dzwgRyBmIG0geSCqILgg2iDeIBEhFSEyIUMhTSFQIZwhnyHSIQAiIyKLIrEivyIUIwABZ3RAIEMgAODZIjgD9uBrItIgBwmAAWVsdABNIF8gYiBmAHQAAAFhclMgWCByInJvdwAAoM0h6SRnaHRhcnJvdwCgziEA4NgiOAP24Goi0iBfCekkZ2h0YXJyb3cAoM8hAAFEZHEgdSDhIXNoAKCvIuEhc2gAoK4igAJiY25wdACCIIYgiSCNIKIgbABhAACgByL1IXRlRGFnAADgICLSIACiSSJFaW9wlSCYIJwgniAA4HAqOANkAADgSyI4A3MASWFyAG8A+AAyCnUAcgBhoG4mbADzoG4mmwjzAa8gAACzIHAAO4CgAKBAbQBwAOXgTiI4AyoJgAJhZW91eQDBIMogzSDWINkg8AHGIAAAyCAAoEMqbwBuAEhh5CFpbEZhbgBnAGSgRyJvAHQAAOBtKjgDcAAAoEIqPWThIXNoAKATIACjYCJBYWRxc3jpIO0g+SD+IAIhDCFyAHIAAKDXIXIAAAFocvIg9SBrAACgJClvoJch9wAGD28AdAAA4FAiOAN1AGkA9gC7CAABZWkGIQohYQByAACgKCntAN8I6SFzdPOgBCLlCHIAAOA12CvdAAJFZXN0/wgcISshLiHxoXEiIiEAABMJ8aFxIgAJAAAnIWwAYQBuAPQAEwlpAO0AGQlyoG8iAKBvIoABQWFwADghOyE/IXIA8gBeIHIAcgAAoK4hYQByAACg8ipzogsiSiEAAAAAxwtkoPwiAKD6ImMAeQBaZIADQUVhZGVzdABcIV8hYiFmIWkhkyGWIXIA8gBXIADgZiI4A3IAcgAAoJohcgAAoCUggKFwImZxcwBwIYQhjiF0AAABYXJ1IXohcgByAG8A9wBlIWkAZwBoAHQAYQByAHIAbwD3AD4h8aFwImAhAACKIWwAYQBuAPQAZwlz4H0qOAMAoG4iaQDtAG0JcqBuImkA5aDqIkUJaQDkADoKAAFwdKMhpyFmAADgNdhf3YCBrAA7aW4AriGvIcchrEBuAIChCSJFZHYAtyG6Ib8hAOD5IjgDbwB0AADg9SI4A+EB1gjEIcYhAKD3IgCg9iJpAHagDCLhAagJzyHRIQCg/iIAoP0igAFhb3IA2CHsIfEhcgCAoSYiYXN0AOAh5SHpIWwAbABlAOwAywhsAADg/SrlIADgAiI4A2wiaW50AACgFCrjoYAi9yEAAPohdQDlAJsJY+CvKjgDZaCAIvEAkwkAAkFhaXQHIgoiFyIeInIA8gBsIHIAcgAAoZshY3cRIhQiAOAzKTgDAOCdITgDZyRodGFycm93AACgmyFyAGkA5aDrIr4JgANjaGltcHF1AC8iPCJHIpwhTSJQIloigKGBImNlcgA2Iv0JOSJ1AOUABgoA4DXYw9zvIXJ0bQKdIQAAAABEImEAcgDhAOEhbQBloEEi8aBEIiYKYQDyAMsIcwB1AAABYnBWIlgi5QDUCeUA3wmAAWJjcABgInMieCKAoYQiRWVzAGci7glqIgDgxSo4A2UAdABl4IIi0iBxAPGgiCJoImMAZaCBIvEA/gmAoYUiRWVzAH8iFgqCIgDgxio4A2UAdABl4IMi0iBxAPGgiSKAIgACZ2lscpIilCKaIpwi7AAMCWwAZABlADuA8QDxQOcAWwlpI2FuZ2xlAAABbHKkIqoi5SFmdGWg6iLxAEUJaSJnaHQAZaDrIvEAvgltoL0DAKEjAGVzuCK8InIAbwAAoBYhcAAAoAcggARESGFkZ2lscnMAziLSItYi2iLeIugi7SICIw8j4SFzaACgrSLhIXJyAKAEKXAAAOBNItIg4SFzaACgrCIAAWV04iLlIgDgZSLSIADgPgDSIG4iZmluAACg3imAAUFldADzIvci+iJyAHIAAKACKQDgZCLSIHLgPADSIGkAZQAA4LQi0iAAAUF0BiMKI3IAcgAAoAMp8iFpZQDgtSLSIGkAbQAA4Dwi0iCAAUFhbgAaIx4jKiNyAHIAAKDWIXIAAAFociMjJiNrAACgIylvoJYh9wD/DuUhYXIAoCcpUxJqFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVCMAAF4jaSN/I4IjjSOeI8AUAAAAAKYjwCMAANoj3yMAAO8jHiQvJD8kRCQAAWNzVyNsFHUAdABlADuA8wDzQAABaXlhI2cjcgBjoJoiO4D0APRAPmSAAmFiaW9zAHEjdCN3I3EBeiNzAOgAdhTsIWFjUWF2AACgOCrvIWxkAKC8KewhaWdTYQABY3KFI4kjaQByAACgvykA4DXYLN1vA5QjAAAAAJYjAACcI24A22JhAHYAZQA7gPIA8kAAoMEpAAFibaEjjAphAHIAAKC1KQACYWNpdKwjryO6I70jcgDyAFkUAAFpcrMjtiNyAACgvinvIXNzAKC7KW4A5QDZCgCgwCmAAWFlaQDFI8gjyyNjAHIATWFnAGEAyWOAAWNkbgDRI9Qj1iPyIW9uv2MAoLYpdQDzAHgBcABmAADgNdhg3YABYWVsAOQj5yPrI3IAAKC3KXIAcAAAoLkpdQDzAHwBAKMoImFkaW9zdvkj/CMPJBMkFiQbJHIA8gBeFIChXSplZm0AAyQJJAwkcgBvoDQhZgAAoDQhO4CqAKpAO4C6ALpA5yFvZgCgtiJyAACgVipsIm9wZQAAoFcqAKBbKoABY2xvACMkJSQrJPIACCRhAHMAaAA7gPgA+EBsAACgmCJpAGwBMyQ4JGQAZQA7gPUA9UBlAHMAYaCXInMAAKA2Km0AbAA7gPYA9kDiIWFyAKA9I+EKXiQAAHokAAB8JJQkAACYJKkkAAAAALUkEQsAAPAkAAAAAAQleiUAAIMlcgCAoSUiYXN0AGUkbyQBCwCBtgA7bGokayS2QGwAZQDsABgDaQJ1JAAAAAB4JG0AAKDzKgCg/Sp5AD9kcgCAAmNpbXB0AIUkiCSLJJkSjyRuAHQAJWBvAGQALmBpAGwAAKAwIOUhbmsAoDEgcgAA4DXYLd2AAWltbwCdJKAkpCR2oMYD1WNtAGEA9AD+B24AZQAAoA4m9KHAA64kAAC0JGMjaGZvcmsAAKDUItZjAAFhdbgkxCRuAAABY2u9JMIkawBooA8hAKAOIfYAaRpzAACkKwBhYmNkZW1zdNMkIRPXJNsk4STjJOck6yTjIWlyAKAjKmkAcgAAoCIqAAFvdYsW3yQAoCUqAKByKm4AO4CxALFAaQBtAACgJip3AG8AAKAnKoABaXB1APUk+iT+JO4idGludACgFSpmAADgNdhh3W4AZAA7gKMAo0CApHoiRWFjZWlub3N1ABMlFSUYJRslTCVRJVklSSV1JQCgsypwAACgtyp1AOUAPwtjoK8qgKJ6ImFjZW5zACclLSU0JTYlSSVwAHAAcgBvAPgAFyV1AHIAbAB5AGUA8QA/C/EAOAuAAWFlcwA8JUElRSXwInByb3gAoLkqcQBxAACgtSppAG0AAKDoImkA7QBEC20AZQDzoDIgIguAAUVhcwBDJVclRSXwAEAlgAFkZnAATwtfJXElgAFhbHMAZSVpJW0l7CFhcgCgLiPpIW5lAKASI/UhcmYAoBMjdKAdIu8AWQvyIWVsAKCwIgABY2l9JYElcgAA4DXYxdzIY24iY3NwAACgCCAAA2Zpb3BzdZElKxuVJZolnyWkJXIAAOA12C7dcABmAADgNdhi3XIiaW1lAACgVyBjAHIAAOA12MbcgAFhZW8AqiW6JcAldAAAAWVpryW2JXIAbgBpAG8AbgDzABkFbgB0AACgFipzAHQAZaA/APEACRj0AG0LgApBQkhhYmNkZWZoaWxtbm9wcnN0dXgA4yXyJfYl+iVpJpAmpia9JtUm5ib4JlonaCdxJ3UnnietJ7EnyCfiJ+cngAFhcnQA6SXsJe4lcgDyAJkM8gD6AuEhaWwAoBwpYQByAPIA3BVhAHIAAKBkKYADY2RlbnFydAAGJhAmEyYYJiYmKyZaJgABZXUKJg0mAOA9IjEDdABlAFVhaQDjACAN7SJwdHl2AKCzKWcAgKHpJ2RlbAAgJiImJCYAoJIpAKClKeUA9wt1AG8AO4C7ALtAcgAApZIhYWJjZmhscHN0dz0mQCZFJkcmSiZMJk4mUSZVJlgmcAAAoHUpZqDlIXMAAKAgKQCgMylzAACgHinrALka8ACVHmwAAKBFKWkAbQAAoHQpbAAAoKMhAKCdIQABYWleJmImaQBsAACgGilvAG6gNiJhAGwA8wB2C4ABYWJyAG8mciZ2JnIA8gAvEnIAawAAoHMnAAFha3omgSZjAAABZWt/JoAmfWBdYAABZXOFJocmAKCMKWwAAAFkdYwmjiYAoI4pAKCQKQACYWV1eZcmmiajJqUm8iFvbllhAAFkaZ4moSZpAGwAV2HsAA8M4gCAJkBkAAJjbHFzrSawJrUmuiZhAACgNylkImhhcgAAoGkpdQBvAPKgHSCjAWgAAKCzIYABYWNnAMMm0iaUC2wAgKEcIWlwcwDLJs4migxuAOUAoAxhAHIA9ADaC3QAAKCtJYABaWxyANsm3ybjJvMhaHQAoH0pbwBvAPIANgwA4DXYL90AAWFv6ib1JnIAAAFkde8m8SYAoMEhbKDAIQCgbCl2oMED8WOAAWducwD+Jk4nUCdoAHQAAANhaGxyc3QKJxInISc1Jz0nRydyInJvdwB0oJIhYQDpAFYmYSNycG9vbgAAAWR1GiceJ28AdwDuAPAmcAAAoMAh5SFmdAABYWgnJy0ncgByAG8AdwDzAAkMYQByAHAAbwBvAG4A8wATBGklZ2h0YXJyb3dzAACgySFxAHUAaQBnAGEAcgByAG8A9wBZJugkcmVldGltZXMAoMwiZwDaYmkAbgBnAGQAbwB0AHMAZQDxABwYgAFhaG0AYCdjJ2YncgDyAAkMYQDyABMEAKAPIG8idXN0AGGgsSPjIWhlAKCxI+0haWQAoO4qAAJhYnB0fCeGJ4knmScAAW5ygCeDJ2cAAKDtJ3IAAKD+IXIA6wAcDIABYWZsAI8nkieVJ3IAAKCGKQDgNdhj3XUAcwAAoC4qaSJtZXMAAKA1KgABYXCiJ6gncgBnoCkAdAAAoJQp7yJsaW50AKASKmEAcgDyADwnAAJhY2hxuCe8J6EMwCfxIXVvAKA6IHIAAOA12MfcAAFidYAmxCdvAPKgGSCoAYABaGlyAM4n0ifWJ3IAZQDlAE0n7SFlcwCgyiJpAIChuSVlZmwAXAxjEt4n9CFyaQCgzinsInVoYXIAoGgpAKAeIWENBSgJKA0oSyhVKIYoAACLKLAoAAAAAOMo5ygAABApJCkxKW0pcSmHKaYpAACYKgAAAACxKmMidXRlAFthcQB1AO8ABR+ApHsiRWFjZWlucHN5ABwoHignKCooLygyKEEoRihJKACgtCrwASMoAAAlKACguCpvAG4AYWF1AOUAgw1koLAqaQBsAF9hcgBjAF1hgAFFYXMAOCg6KD0oAKC2KnAAAKC6KmkAbQAAoOki7yJsaW50AKATKmkA7QCIDUFkbwB0AGKixSKRFgAAAABTKACgZiqAA0FhY21zdHgAYChkKG8ocyh1KHkogihyAHIAAKDYIXIAAAFocmkoayjrAJAab6CYIfcAzAd0ADuApwCnQGkAO2D3IWFyAKApKW0AAAFpbn4ozQBuAHUA8wDOAHQAAKA2J3IA7+A12DDdIxkAAmFjb3mRKJUonSisKHIAcAAAoG8mAAFoeZkonChjAHkASWRIZHIAdABtAqUoAAAAAKgoaQDkAFsPYQByAGEA7ABsJDuArQCtQAABZ22zKLsobQBhAAChwwNmdroouijCY4CjPCJkZWdsbnByAMgozCjPKNMo1yjaKN4obwB0AACgairxoEMiCw5FoJ4qAKCgKkWgnSoAoJ8qZQAAoEYi7CF1cwCgJCrhIXJyAKByKWEAcgDyAPwMAAJhZWl07Sj8KAEpCCkAAWxz8Sj4KGwAcwBlAHQAbQDpAH8oaABwAACgMyrwImFyc2wAoOQpAAFkbFoPBSllAACgIyNloKoqc6CsKgDgrCoA/oABZmxwABUpGCkfKfQhY3lMZGKgLwBhoMQpcgAAoD8jZgAA4DXYZN1hAAABZHIoKRcDZQBzAHWgYCZpAHQAAKBgJoABY3N1ADYpRilhKQABYXU6KUApcABzoJMiAOCTIgD+cABzoJQiAOCUIgD+dQAAAWJwSylWKQChjyJlcz4NUCllAHQAZaCPIvEAPw0AoZAiZXNIDVspZQB0AGWgkCLxAEkNAKGhJWFmZilbBHIAZQFrKVwEAKChJWEAcgDyAAMNAAJjZW10dyl7KX8pgilyAADgNdjI3HQAbQDuAM4AaQDsAAYpYQByAOYAVw0AAWFyiimOKXIA5qAGJhESAAFhbpIpoylpImdodAAAAWVwmSmgKXAAcwBpAGwAbwDuANkXaADpAKAkcwCvYIACYmNtbnAArin8KY4NJSooKgCkgiJFZGVtbnByc7wpvinCKcgpzCnUKdgp3CkAoMUqbwB0AACgvSpkoIYibwB0AACgwyr1IWx0AKDBKgABRWXQKdIpAKDLKgCgiiLsIXVzAKC/KuEhcnIAoHkpgAFlaXUA4inxKfQpdAAAoYIiZW7oKewpcQDxoIYivSllAHEA8aCKItEpbQAAoMcqAAFicPgp+ikAoNUqAKDTKmMAgKJ7ImFjZW5zAAcqDSoUKhYqRihwAHAAcgBvAPgAIyh1AHIAbAB5AGUA8QCDDfEAfA2AAWFlcwAcKiIqPShwAHAAcgBvAPgAPChxAPEAOShnAACgaiYApoMiMTIzRWRlaGxtbnBzPCo/KkIqRSpHKlIqWCpjKmcqaypzKncqO4C5ALlAO4CyALJAO4CzALNAAKDGKgABb3NLKk4qdAAAoL4qdQBiAACg2CpkoIcibwB0AACgxCpzAAABb3VdKmAqbAAAoMknYgAAoNcq4SFycgCgeyn1IWx0AKDCKgABRWVvKnEqAKDMKgCgiyLsIXVzAKDAKoABZWl1AH0qjCqPKnQAAKGDImVugyqHKnEA8aCHIkYqZQBxAPGgiyJwKm0AAKDIKgABYnCTKpUqAKDUKgCg1iqAAUFhbgCdKqEqrCpyAHIAAKDZIXIAAAFocqYqqCrrAJUab6CZIfcAxQf3IWFyAKAqKWwAaQBnADuA3wDfQOELzyrZKtwq6SrsKvEqAAD1KjQrAAAAAAAAAAAAAEwrbCsAAHErvSsAAAAAAADRK3IC1CoAAAAA2CrnIWV0AKAWI8RjcgDrAOUKgAFhZXkA4SrkKucq8iFvbmVh5CFpbGNhQmRvAPQAIg5sInJlYwAAoBUjcgAA4DXYMd0AAmVpa2/7KhIrKCsuK/IBACsAAAkrZQAAATRm6g0EK28AcgDlAOsNYQBzorgDECsAAAAAEit5AG0A0WMAAWNuFislK2sAAAFhcxsrIStwAHAAcgBvAPgAFw5pAG0AAKA8InMA8AD9DQABYXMsKyEr8AAXDnIAbgA7gP4A/kDsATgrOyswG2QA5QBnAmUAcwCAgdcAO2JkAEMrRCtJK9dAYaCgInIAAKAxKgCgMCqAAWVwcwBRK1MraSvhAAkh4qKkIlsrXysAAAAAYytvAHQAAKA2I2kAcgAAoPEqb+A12GXdcgBrAACg2irhAHgociJpbWUAAKA0IIABYWlwAHYreSu3K2QA5QC+DYADYWRlbXBzdACFK6MrmiunK6wrsCuzK24iZ2xlAACitSVkbHFykCuUK5ornCvvIXduAKC/JeUhZnRloMMl8QACBwCgXCJpImdodABloLkl8QBdDG8AdAAAoOwlaSJudXMAAKA6KuwhdXMAoDkqYgAAoM0p6SFtZQCgOyrlInppdW0AoOIjgAFjaHQAwivKK80rAAFyecYrySsA4DXYydxGZGMAeQBbZPIhb2tnYQABaW/UK9creAD0ANERaCJlYWQAAAFsct4r5ytlAGYAdABhAHIAcgBvAPcAXQbpJGdodGFycm93AKCgIQAJQUhhYmNkZmdobG1vcHJzdHV3CiwNLBEsHSwnLDEsQCxLLFIsYix6LIQsjyzLLOgs7Sz/LAotcgDyAAkDYQByAACgYykAAWNyFSwbLHUAdABlADuA+gD6QPIACQ1yAOMBIywAACUseQBeZHYAZQBtYQABaXkrLDAscgBjADuA+wD7QENkgAFhYmgANyw6LD0scgDyANEO7CFhY3FhYQDyAOAOAAFpckQsSCzzIWh0AKB+KQDgNdgy3XIAYQB2AGUAO4D5APlAYQFWLF8scgAAAWxyWixcLACgvyEAoL4hbABrAACggCUAAWN0Zix2LG8CbCwAAAAAcyxyAG4AZaAcI3IAAKAcI28AcAAAoA8jcgBpAACg+CUAAWFsfiyBLGMAcgBrYTuAqACoQAABZ3CILIssbwBuAHNhZgAA4DXYZt0AA2FkaGxzdZksniynLLgsuyzFLHIAcgBvAPcACQ1vAHcAbgBhAHIAcgBvAPcA2A5hI3Jwb29uAAABbHKvLLMsZQBmAPQAWyxpAGcAaAD0AF0sdQDzAKYOaQAAocUDaGzBLMIs0mNvAG4AxWPwI2Fycm93cwCgyCGAAWNpdADRLOEs5CxvAtcsAAAAAN4scgBuAGWgHSNyAACgHSNvAHAAAKAOI24AZwBvYXIAaQAAoPklYwByAADgNdjK3IABZGlyAPMs9yz6LG8AdAAAoPAi7CFkZWlhaQBmoLUlAKC0JQABYW0DLQYtcgDyAMosbAA7gPwA/EDhIm5nbGUAoKcpgAdBQkRhY2RlZmxub3Byc3oAJy0qLTAtNC2bLZ0toS2/LcMtxy3TLdgt3C3gLfwtcgDyABADYQByAHag6CoAoOkqYQBzAOgA/gIAAW5yOC08LechcnQAoJwpgANla25wcnN0AJkpSC1NLVQtXi1iLYItYQBwAHAA4QAaHG8AdABoAGkAbgDnAKEXgAFoaXIAoSmzJFotbwBwAPQAdCVooJUh7wD4JgABaXVmLWotZwBtAOEAuygAAWJwbi14LXMjZXRuZXEAceCKIgD+AODLKgD+cyNldG5lcQBx4IsiAP4A4MwqAP4AAWhyhi2KLWUAdADhABIraSNhbmdsZQAAAWxyki2WLeUhZnQAoLIiaSJnaHQAAKCzInkAMmThIXNoAKCiIoABZWxyAKcttC24LWKiKCKuLQAAAACyLWEAcgAAoLsicQAAoFoi7CFpcACg7iIAAWJ0vC1eD2EA8gBfD3IAAOA12DPddAByAOkAlS1zAHUAAAFicM0t0C0A4IIi0iAA4IMi0iBwAGYAAOA12GfdcgBvAPAAWQt0AHIA6QCaLQABY3XkLegtcgAA4DXYy9wAAWJw7C30LW4AAAFFZXUt8S0A4IoiAP5uAAABRWV/LfktAOCLIgD+6SJnemFnAKCaKYADY2Vmb3BycwANLhAuJS4pLiMuLi40LukhcmN1YQABZGkULiEuAAFiZxguHC5hAHIAAKBfKmUAcaAnIgCgWSLlIXJwAKAYIXIAAOA12DTdcABmAADgNdho3WWgQCJhAHQA6ABqD2MAcgAA4DXYzNzjCuQRUC4AAFQuAABYLmIuAAAAAGMubS5wLnQuAAAAAIguki4AAJouJxIqEnQAcgDpAB0ScgAA4DXYNd0AAUFhWy5eLnIA8gDnAnIA8gCTB75jAAFBYWYuaS5yAPIA4AJyAPIAjAdhAPAAeh5pAHMAAKD7IoABZHB0APgReS6DLgABZmx9LoAuAOA12GnddQDzAP8RaQBtAOUABBIAAUFhiy6OLnIA8gDuAnIA8gCaBwABY3GVLgoScgAA4DXYzdwAAXB0nS6hLmwAdQDzACUScgDpACASAARhY2VmaW9zdbEuvC7ELsguzC7PLtQu2S5jAAABdXm2LrsudABlADuA/QD9QE9kAAFpecAuwy5yAGMAd2FLZG4AO4ClAKVAcgAA4DXYNt1jAHkAV2RwAGYAAOA12GrdYwByAADgNdjO3AABY23dLt8ueQBOZGwAO4D/AP9AAAVhY2RlZmhpb3N38y73Lv8uAi8MLxAvEy8YLx0vIi9jInV0ZQB6YQABYXn7Lv4u8iFvbn5hN2RvAHQAfGEAAWV0Bi8KL3QAcgDmAB8QYQC2Y3IAAOA12DfdYwB5ADZk5yJyYXJyAKDdIXAAZgAA4DXYa91jAHIAAOA12M/cAAFqbiYvKC8AoA0gagAAoAwg"),
    ui = ur("AAJhZ2xxBwARABMAFQBtAg0AAAAAAA8AcAAmYG8AcwAnYHQAPmB0ADxg9SFvdCJg"),
    ae;
  (function (e) {
    e[e.VALUE_LENGTH = 49152] = "VALUE_LENGTH", e[e.FLAG13 = 8192] = "FLAG13", e[e.BRANCH_LENGTH = 8064] = "BRANCH_LENGTH", e[e.JUMP_TABLE = 127] = "JUMP_TABLE";
  })(ae || (ae = {}));
  var U;
  (function (e) {
    e[e.NUM = 35] = "NUM", e[e.SEMI = 59] = "SEMI", e[e.EQUALS = 61] = "EQUALS", e[e.ZERO = 48] = "ZERO", e[e.NINE = 57] = "NINE", e[e.LOWER_A = 97] = "LOWER_A", e[e.LOWER_F = 102] = "LOWER_F", e[e.LOWER_X = 120] = "LOWER_X", e[e.LOWER_Z = 122] = "LOWER_Z", e[e.UPPER_A = 65] = "UPPER_A", e[e.UPPER_F = 70] = "UPPER_F", e[e.UPPER_Z = 90] = "UPPER_Z";
  })(U || (U = {}));
  var ar = 32;
  function Mn(e) {
    return e >= U.ZERO && e <= U.NINE;
  }
  function ai(e) {
    return e >= U.UPPER_A && e <= U.UPPER_F || e >= U.LOWER_A && e <= U.LOWER_F;
  }
  function ii(e) {
    return e >= U.UPPER_A && e <= U.UPPER_Z || e >= U.LOWER_A && e <= U.LOWER_Z || Mn(e);
  }
  function si(e) {
    return e === U.EQUALS || ii(e);
  }
  var Z;
  (function (e) {
    e[e.EntityStart = 0] = "EntityStart", e[e.NumericStart = 1] = "NumericStart", e[e.NumericDecimal = 2] = "NumericDecimal", e[e.NumericHex = 3] = "NumericHex", e[e.NamedEntity = 4] = "NamedEntity";
  })(Z || (Z = {}));
  var me;
  (function (e) {
    e[e.Legacy = 0] = "Legacy", e[e.Strict = 1] = "Strict", e[e.Attribute = 2] = "Attribute";
  })(me || (me = {}));
  var oi = class {
    constructor(e, t, n) {
      this.decodeTree = e, this.emitCodePoint = t, this.errors = n, this.state = Z.EntityStart, this.consumed = 1, this.result = 0, this.treeIndex = 0, this.excess = 1, this.decodeMode = me.Strict, this.runConsumed = 0;
    }
    startEntity(e) {
      this.decodeMode = e, this.state = Z.EntityStart, this.result = 0, this.treeIndex = 0, this.excess = 1, this.consumed = 1, this.runConsumed = 0;
    }
    write(e, t) {
      switch (this.state) {
        case Z.EntityStart:
          return e.charCodeAt(t) === U.NUM ? (this.state = Z.NumericStart, this.consumed += 1, this.stateNumericStart(e, t + 1)) : (this.state = Z.NamedEntity, this.stateNamedEntity(e, t));
        case Z.NumericStart:
          return this.stateNumericStart(e, t);
        case Z.NumericDecimal:
          return this.stateNumericDecimal(e, t);
        case Z.NumericHex:
          return this.stateNumericHex(e, t);
        case Z.NamedEntity:
          return this.stateNamedEntity(e, t);
      }
    }
    stateNumericStart(e, t) {
      return t >= e.length ? -1 : (e.charCodeAt(t) | ar) === U.LOWER_X ? (this.state = Z.NumericHex, this.consumed += 1, this.stateNumericHex(e, t + 1)) : (this.state = Z.NumericDecimal, this.stateNumericDecimal(e, t));
    }
    stateNumericHex(e, t) {
      for (; t < e.length;) {
        let n = e.charCodeAt(t);
        if (Mn(n) || ai(n)) {
          let r = n <= U.NINE ? n - U.ZERO : (n | ar) - U.LOWER_A + 10;
          this.result = this.result * 16 + r, this.consumed++, t++;
        } else return this.emitNumericEntity(n, 3);
      }
      return -1;
    }
    stateNumericDecimal(e, t) {
      for (; t < e.length;) {
        let n = e.charCodeAt(t);
        if (Mn(n)) this.result = this.result * 10 + (n - U.ZERO), this.consumed++, t++;else return this.emitNumericEntity(n, 2);
      }
      return -1;
    }
    emitNumericEntity(e, t) {
      var n;
      if (this.consumed <= t) return (n = this.errors) === null || n === void 0 || n.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
      if (e === U.SEMI) this.consumed += 1;else if (this.decodeMode === me.Strict) return 0;
      return this.emitCodePoint(ni(this.result), this.consumed), this.errors && (e !== U.SEMI && this.errors.missingSemicolonAfterCharacterReference(), this.errors.validateNumericCharacterReference(this.result)), this.consumed;
    }
    stateNamedEntity(e, t) {
      let n = this.decodeTree,
        r = n[this.treeIndex],
        a = (r & ae.VALUE_LENGTH) >> 14;
      for (; t < e.length;) {
        if (a === 0 && (r & ae.FLAG13) !== 0) {
          let s = (r & ae.BRANCH_LENGTH) >> 7;
          if (this.runConsumed === 0) {
            let l = r & ae.JUMP_TABLE;
            if (e.charCodeAt(t) !== l) return this.result === 0 ? 0 : this.emitNotTerminatedNamedEntity();
            t++, this.excess++, this.runConsumed++;
          }
          for (; this.runConsumed < s;) {
            if (t >= e.length) return -1;
            let l = this.runConsumed - 1,
              c = n[this.treeIndex + 1 + (l >> 1)],
              E = l % 2 === 0 ? c & 255 : c >> 8 & 255;
            if (e.charCodeAt(t) !== E) return this.runConsumed = 0, this.result === 0 ? 0 : this.emitNotTerminatedNamedEntity();
            t++, this.excess++, this.runConsumed++;
          }
          this.runConsumed = 0, this.treeIndex += 1 + (s >> 1), r = n[this.treeIndex], a = (r & ae.VALUE_LENGTH) >> 14;
        }
        if (t >= e.length) break;
        let i = e.charCodeAt(t);
        if (i === U.SEMI && a !== 0 && (r & ae.FLAG13) !== 0) return this.emitNamedEntityData(this.treeIndex, a, this.consumed + this.excess);
        if (this.treeIndex = Ai(n, r, this.treeIndex + Math.max(1, a), i), this.treeIndex < 0) return this.result === 0 || this.decodeMode === me.Attribute && (a === 0 || si(i)) ? 0 : this.emitNotTerminatedNamedEntity();
        if (r = n[this.treeIndex], a = (r & ae.VALUE_LENGTH) >> 14, a !== 0) {
          if (i === U.SEMI) return this.emitNamedEntityData(this.treeIndex, a, this.consumed + this.excess);
          this.decodeMode !== me.Strict && (r & ae.FLAG13) === 0 && (this.result = this.treeIndex, this.consumed += this.excess, this.excess = 0);
        }
        t++, this.excess++;
      }
      return -1;
    }
    emitNotTerminatedNamedEntity() {
      var e;
      let t = this.result,
        n = this.decodeTree,
        r = (n[t] & ae.VALUE_LENGTH) >> 14;
      return this.emitNamedEntityData(t, r, this.consumed), (e = this.errors) === null || e === void 0 || e.missingSemicolonAfterCharacterReference(), this.consumed;
    }
    emitNamedEntityData(e, t, n) {
      let r = this.decodeTree;
      return this.emitCodePoint(t === 1 ? r[e] & ~(ae.VALUE_LENGTH | ae.FLAG13) : r[e + 1], n), t === 3 && this.emitCodePoint(r[e + 2], n), n;
    }
    end() {
      var e;
      switch (this.state) {
        case Z.NamedEntity:
          return this.result !== 0 && (this.decodeMode !== me.Attribute || this.result === this.treeIndex) ? this.emitNotTerminatedNamedEntity() : 0;
        case Z.NumericDecimal:
          return this.emitNumericEntity(0, 2);
        case Z.NumericHex:
          return this.emitNumericEntity(0, 3);
        case Z.NumericStart:
          return (e = this.errors) === null || e === void 0 || e.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
        case Z.EntityStart:
          return 0;
      }
    }
  };
  function Ai(e, t, n, r) {
    let a = (t & ae.BRANCH_LENGTH) >> 7,
      i = t & ae.JUMP_TABLE;
    if (a === 0) return i !== 0 && r === i ? n : -1;
    if (i) {
      let E = r - i;
      return E < 0 || E >= a ? -1 : e[n + E] - 1;
    }
    let s = a + 1 >> 1,
      l = 0,
      c = a - 1;
    for (; l <= c;) {
      let E = l + c >>> 1,
        f = E >> 1,
        m = e[n + f] >> (E & 1) * 8 & 255;
      if (m < r) l = E + 1;else if (m > r) c = E - 1;else return e[n + s + E];
    }
    return -1;
  }
  var F;
  (function (e) {
    e[e.Tab = 9] = "Tab", e[e.NewLine = 10] = "NewLine", e[e.FormFeed = 12] = "FormFeed", e[e.CarriageReturn = 13] = "CarriageReturn", e[e.Space = 32] = "Space", e[e.ExclamationMark = 33] = "ExclamationMark", e[e.Number = 35] = "Number", e[e.Amp = 38] = "Amp", e[e.SingleQuote = 39] = "SingleQuote", e[e.DoubleQuote = 34] = "DoubleQuote", e[e.Dash = 45] = "Dash", e[e.Slash = 47] = "Slash", e[e.Zero = 48] = "Zero", e[e.Nine = 57] = "Nine", e[e.Semi = 59] = "Semi", e[e.Lt = 60] = "Lt", e[e.Eq = 61] = "Eq", e[e.Gt = 62] = "Gt", e[e.Questionmark = 63] = "Questionmark", e[e.UpperA = 65] = "UpperA", e[e.LowerA = 97] = "LowerA", e[e.UpperF = 70] = "UpperF", e[e.LowerF = 102] = "LowerF", e[e.UpperZ = 90] = "UpperZ", e[e.LowerZ = 122] = "LowerZ", e[e.LowerX = 120] = "LowerX", e[e.OpeningSquareBracket = 91] = "OpeningSquareBracket";
  })(F || (F = {}));
  var x;
  (function (e) {
    e[e.Text = 1] = "Text", e[e.BeforeTagName = 2] = "BeforeTagName", e[e.InTagName = 3] = "InTagName", e[e.InSelfClosingTag = 4] = "InSelfClosingTag", e[e.BeforeClosingTagName = 5] = "BeforeClosingTagName", e[e.InClosingTagName = 6] = "InClosingTagName", e[e.AfterClosingTagName = 7] = "AfterClosingTagName", e[e.BeforeAttributeName = 8] = "BeforeAttributeName", e[e.InAttributeName = 9] = "InAttributeName", e[e.AfterAttributeName = 10] = "AfterAttributeName", e[e.BeforeAttributeValue = 11] = "BeforeAttributeValue", e[e.InAttributeValueDq = 12] = "InAttributeValueDq", e[e.InAttributeValueSq = 13] = "InAttributeValueSq", e[e.InAttributeValueNq = 14] = "InAttributeValueNq", e[e.BeforeDeclaration = 15] = "BeforeDeclaration", e[e.InDeclaration = 16] = "InDeclaration", e[e.InProcessingInstruction = 17] = "InProcessingInstruction", e[e.BeforeComment = 18] = "BeforeComment", e[e.CDATASequence = 19] = "CDATASequence", e[e.InSpecialComment = 20] = "InSpecialComment", e[e.InCommentLike = 21] = "InCommentLike", e[e.BeforeSpecialS = 22] = "BeforeSpecialS", e[e.BeforeSpecialT = 23] = "BeforeSpecialT", e[e.SpecialStartSequence = 24] = "SpecialStartSequence", e[e.InSpecialTag = 25] = "InSpecialTag", e[e.InEntity = 26] = "InEntity";
  })(x || (x = {}));
  function Se(e) {
    return e === F.Space || e === F.NewLine || e === F.Tab || e === F.FormFeed || e === F.CarriageReturn;
  }
  function qt(e) {
    return e === F.Slash || e === F.Gt || Se(e);
  }
  function ci(e) {
    return e >= F.LowerA && e <= F.LowerZ || e >= F.UpperA && e <= F.UpperZ;
  }
  var De;
  (function (e) {
    e[e.NoValue = 0] = "NoValue", e[e.Unquoted = 1] = "Unquoted", e[e.Single = 2] = "Single", e[e.Double = 3] = "Double";
  })(De || (De = {}));
  var X = {
      Cdata: new Uint8Array([67, 68, 65, 84, 65, 91]),
      CdataEnd: new Uint8Array([93, 93, 62]),
      CommentEnd: new Uint8Array([45, 45, 62]),
      ScriptEnd: new Uint8Array([60, 47, 115, 99, 114, 105, 112, 116]),
      StyleEnd: new Uint8Array([60, 47, 115, 116, 121, 108, 101]),
      TitleEnd: new Uint8Array([60, 47, 116, 105, 116, 108, 101]),
      TextareaEnd: new Uint8Array([60, 47, 116, 101, 120, 116, 97, 114, 101, 97]),
      XmpEnd: new Uint8Array([60, 47, 120, 109, 112])
    },
    li = class {
      constructor({
        xmlMode: e = !1,
        decodeEntities: t = !0
      }, n) {
        this.cbs = n, this.state = x.Text, this.buffer = "", this.sectionStart = 0, this.index = 0, this.entityStart = 0, this.baseState = x.Text, this.isSpecial = !1, this.running = !0, this.offset = 0, this.currentSequence = void 0, this.sequenceIndex = 0, this.xmlMode = e, this.decodeEntities = t, this.entityDecoder = new oi(e ? ui : ri, (r, a) => this.emitCodePoint(r, a));
      }
      reset() {
        this.state = x.Text, this.buffer = "", this.sectionStart = 0, this.index = 0, this.baseState = x.Text, this.currentSequence = void 0, this.running = !0, this.offset = 0;
      }
      write(e) {
        this.offset += this.buffer.length, this.buffer = e, this.parse();
      }
      end() {
        this.running && this.finish();
      }
      pause() {
        this.running = !1;
      }
      resume() {
        this.running = !0, this.index < this.buffer.length + this.offset && this.parse();
      }
      stateText(e) {
        e === F.Lt || !this.decodeEntities && this.fastForwardTo(F.Lt) ? (this.index > this.sectionStart && this.cbs.ontext(this.sectionStart, this.index), this.state = x.BeforeTagName, this.sectionStart = this.index) : this.decodeEntities && e === F.Amp && this.startEntity();
      }
      stateSpecialStartSequence(e) {
        let t = this.sequenceIndex === this.currentSequence.length;
        if (!(t ? qt(e) : (e | 32) === this.currentSequence[this.sequenceIndex])) this.isSpecial = !1;else if (!t) {
          this.sequenceIndex++;
          return;
        }
        this.sequenceIndex = 0, this.state = x.InTagName, this.stateInTagName(e);
      }
      stateInSpecialTag(e) {
        if (this.sequenceIndex === this.currentSequence.length) {
          if (e === F.Gt || Se(e)) {
            let t = this.index - this.currentSequence.length;
            if (this.sectionStart < t) {
              let n = this.index;
              this.index = t, this.cbs.ontext(this.sectionStart, t), this.index = n;
            }
            this.isSpecial = !1, this.sectionStart = t + 2, this.stateInClosingTagName(e);
            return;
          }
          this.sequenceIndex = 0;
        }
        (e | 32) === this.currentSequence[this.sequenceIndex] ? this.sequenceIndex += 1 : this.sequenceIndex === 0 ? this.currentSequence === X.TitleEnd ? this.decodeEntities && e === F.Amp && this.startEntity() : this.fastForwardTo(F.Lt) && (this.sequenceIndex = 1) : this.sequenceIndex = +(e === F.Lt);
      }
      stateCDATASequence(e) {
        e === X.Cdata[this.sequenceIndex] ? ++this.sequenceIndex === X.Cdata.length && (this.state = x.InCommentLike, this.currentSequence = X.CdataEnd, this.sequenceIndex = 0, this.sectionStart = this.index + 1) : (this.sequenceIndex = 0, this.state = x.InDeclaration, this.stateInDeclaration(e));
      }
      fastForwardTo(e) {
        for (; ++this.index < this.buffer.length + this.offset;) if (this.buffer.charCodeAt(this.index - this.offset) === e) return !0;
        return this.index = this.buffer.length + this.offset - 1, !1;
      }
      stateInCommentLike(e) {
        e === this.currentSequence[this.sequenceIndex] ? ++this.sequenceIndex === this.currentSequence.length && (this.currentSequence === X.CdataEnd ? this.cbs.oncdata(this.sectionStart, this.index, 2) : this.cbs.oncomment(this.sectionStart, this.index, 2), this.sequenceIndex = 0, this.sectionStart = this.index + 1, this.state = x.Text) : this.sequenceIndex === 0 ? this.fastForwardTo(this.currentSequence[0]) && (this.sequenceIndex = 1) : e !== this.currentSequence[this.sequenceIndex - 1] && (this.sequenceIndex = 0);
      }
      isTagStartChar(e) {
        return this.xmlMode ? !qt(e) : ci(e);
      }
      startSpecial(e, t) {
        this.isSpecial = !0, this.currentSequence = e, this.sequenceIndex = t, this.state = x.SpecialStartSequence;
      }
      stateBeforeTagName(e) {
        if (e === F.ExclamationMark) this.state = x.BeforeDeclaration, this.sectionStart = this.index + 1;else if (e === F.Questionmark) this.state = x.InProcessingInstruction, this.sectionStart = this.index + 1;else if (this.isTagStartChar(e)) {
          let t = e | 32;
          this.sectionStart = this.index, this.xmlMode ? this.state = x.InTagName : t === X.ScriptEnd[2] ? this.state = x.BeforeSpecialS : t === X.TitleEnd[2] || t === X.XmpEnd[2] ? this.state = x.BeforeSpecialT : this.state = x.InTagName;
        } else e === F.Slash ? this.state = x.BeforeClosingTagName : (this.state = x.Text, this.stateText(e));
      }
      stateInTagName(e) {
        qt(e) && (this.cbs.onopentagname(this.sectionStart, this.index), this.sectionStart = -1, this.state = x.BeforeAttributeName, this.stateBeforeAttributeName(e));
      }
      stateBeforeClosingTagName(e) {
        Se(e) || (e === F.Gt ? this.state = x.Text : (this.state = this.isTagStartChar(e) ? x.InClosingTagName : x.InSpecialComment, this.sectionStart = this.index));
      }
      stateInClosingTagName(e) {
        (e === F.Gt || Se(e)) && (this.cbs.onclosetag(this.sectionStart, this.index), this.sectionStart = -1, this.state = x.AfterClosingTagName, this.stateAfterClosingTagName(e));
      }
      stateAfterClosingTagName(e) {
        (e === F.Gt || this.fastForwardTo(F.Gt)) && (this.state = x.Text, this.sectionStart = this.index + 1);
      }
      stateBeforeAttributeName(e) {
        e === F.Gt ? (this.cbs.onopentagend(this.index), this.isSpecial ? (this.state = x.InSpecialTag, this.sequenceIndex = 0) : this.state = x.Text, this.sectionStart = this.index + 1) : e === F.Slash ? this.state = x.InSelfClosingTag : Se(e) || (this.state = x.InAttributeName, this.sectionStart = this.index);
      }
      stateInSelfClosingTag(e) {
        e === F.Gt ? (this.cbs.onselfclosingtag(this.index), this.state = x.Text, this.sectionStart = this.index + 1, this.isSpecial = !1) : Se(e) || (this.state = x.BeforeAttributeName, this.stateBeforeAttributeName(e));
      }
      stateInAttributeName(e) {
        (e === F.Eq || qt(e)) && (this.cbs.onattribname(this.sectionStart, this.index), this.sectionStart = this.index, this.state = x.AfterAttributeName, this.stateAfterAttributeName(e));
      }
      stateAfterAttributeName(e) {
        e === F.Eq ? this.state = x.BeforeAttributeValue : e === F.Slash || e === F.Gt ? (this.cbs.onattribend(De.NoValue, this.sectionStart), this.sectionStart = -1, this.state = x.BeforeAttributeName, this.stateBeforeAttributeName(e)) : Se(e) || (this.cbs.onattribend(De.NoValue, this.sectionStart), this.state = x.InAttributeName, this.sectionStart = this.index);
      }
      stateBeforeAttributeValue(e) {
        e === F.DoubleQuote ? (this.state = x.InAttributeValueDq, this.sectionStart = this.index + 1) : e === F.SingleQuote ? (this.state = x.InAttributeValueSq, this.sectionStart = this.index + 1) : Se(e) || (this.sectionStart = this.index, this.state = x.InAttributeValueNq, this.stateInAttributeValueNoQuotes(e));
      }
      handleInAttributeValue(e, t) {
        e === t || !this.decodeEntities && this.fastForwardTo(t) ? (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = -1, this.cbs.onattribend(t === F.DoubleQuote ? De.Double : De.Single, this.index + 1), this.state = x.BeforeAttributeName) : this.decodeEntities && e === F.Amp && this.startEntity();
      }
      stateInAttributeValueDoubleQuotes(e) {
        this.handleInAttributeValue(e, F.DoubleQuote);
      }
      stateInAttributeValueSingleQuotes(e) {
        this.handleInAttributeValue(e, F.SingleQuote);
      }
      stateInAttributeValueNoQuotes(e) {
        Se(e) || e === F.Gt ? (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = -1, this.cbs.onattribend(De.Unquoted, this.index), this.state = x.BeforeAttributeName, this.stateBeforeAttributeName(e)) : this.decodeEntities && e === F.Amp && this.startEntity();
      }
      stateBeforeDeclaration(e) {
        e === F.OpeningSquareBracket ? (this.state = x.CDATASequence, this.sequenceIndex = 0) : this.state = e === F.Dash ? x.BeforeComment : x.InDeclaration;
      }
      stateInDeclaration(e) {
        (e === F.Gt || this.fastForwardTo(F.Gt)) && (this.cbs.ondeclaration(this.sectionStart, this.index), this.state = x.Text, this.sectionStart = this.index + 1);
      }
      stateInProcessingInstruction(e) {
        (e === F.Gt || this.fastForwardTo(F.Gt)) && (this.cbs.onprocessinginstruction(this.sectionStart, this.index), this.state = x.Text, this.sectionStart = this.index + 1);
      }
      stateBeforeComment(e) {
        e === F.Dash ? (this.state = x.InCommentLike, this.currentSequence = X.CommentEnd, this.sequenceIndex = 2, this.sectionStart = this.index + 1) : this.state = x.InDeclaration;
      }
      stateInSpecialComment(e) {
        (e === F.Gt || this.fastForwardTo(F.Gt)) && (this.cbs.oncomment(this.sectionStart, this.index, 0), this.state = x.Text, this.sectionStart = this.index + 1);
      }
      stateBeforeSpecialS(e) {
        let t = e | 32;
        t === X.ScriptEnd[3] ? this.startSpecial(X.ScriptEnd, 4) : t === X.StyleEnd[3] ? this.startSpecial(X.StyleEnd, 4) : (this.state = x.InTagName, this.stateInTagName(e));
      }
      stateBeforeSpecialT(e) {
        switch (e | 32) {
          case X.TitleEnd[3]:
            {
              this.startSpecial(X.TitleEnd, 4);
              break;
            }
          case X.TextareaEnd[3]:
            {
              this.startSpecial(X.TextareaEnd, 4);
              break;
            }
          case X.XmpEnd[3]:
            {
              this.startSpecial(X.XmpEnd, 4);
              break;
            }
          default:
            this.state = x.InTagName, this.stateInTagName(e);
        }
      }
      startEntity() {
        this.baseState = this.state, this.state = x.InEntity, this.entityStart = this.index, this.entityDecoder.startEntity(this.xmlMode ? me.Strict : this.baseState === x.Text || this.baseState === x.InSpecialTag ? me.Legacy : me.Attribute);
      }
      stateInEntity() {
        let e = this.index - this.offset,
          t = this.entityDecoder.write(this.buffer, e);
        if (t >= 0) this.state = this.baseState, t === 0 && (this.index -= 1);else {
          if (e < this.buffer.length && this.buffer.charCodeAt(e) === F.Amp) {
            this.state = this.baseState, this.index -= 1;
            return;
          }
          this.index = this.offset + this.buffer.length - 1;
        }
      }
      cleanup() {
        this.running && this.sectionStart !== this.index && (this.state === x.Text || this.state === x.InSpecialTag && this.sequenceIndex === 0 ? (this.cbs.ontext(this.sectionStart, this.index), this.sectionStart = this.index) : (this.state === x.InAttributeValueDq || this.state === x.InAttributeValueSq || this.state === x.InAttributeValueNq) && (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = this.index));
      }
      shouldContinue() {
        return this.index < this.buffer.length + this.offset && this.running;
      }
      parse() {
        for (; this.shouldContinue();) {
          let e = this.buffer.charCodeAt(this.index - this.offset);
          switch (this.state) {
            case x.Text:
              {
                this.stateText(e);
                break;
              }
            case x.SpecialStartSequence:
              {
                this.stateSpecialStartSequence(e);
                break;
              }
            case x.InSpecialTag:
              {
                this.stateInSpecialTag(e);
                break;
              }
            case x.CDATASequence:
              {
                this.stateCDATASequence(e);
                break;
              }
            case x.InAttributeValueDq:
              {
                this.stateInAttributeValueDoubleQuotes(e);
                break;
              }
            case x.InAttributeName:
              {
                this.stateInAttributeName(e);
                break;
              }
            case x.InCommentLike:
              {
                this.stateInCommentLike(e);
                break;
              }
            case x.InSpecialComment:
              {
                this.stateInSpecialComment(e);
                break;
              }
            case x.BeforeAttributeName:
              {
                this.stateBeforeAttributeName(e);
                break;
              }
            case x.InTagName:
              {
                this.stateInTagName(e);
                break;
              }
            case x.InClosingTagName:
              {
                this.stateInClosingTagName(e);
                break;
              }
            case x.BeforeTagName:
              {
                this.stateBeforeTagName(e);
                break;
              }
            case x.AfterAttributeName:
              {
                this.stateAfterAttributeName(e);
                break;
              }
            case x.InAttributeValueSq:
              {
                this.stateInAttributeValueSingleQuotes(e);
                break;
              }
            case x.BeforeAttributeValue:
              {
                this.stateBeforeAttributeValue(e);
                break;
              }
            case x.BeforeClosingTagName:
              {
                this.stateBeforeClosingTagName(e);
                break;
              }
            case x.AfterClosingTagName:
              {
                this.stateAfterClosingTagName(e);
                break;
              }
            case x.BeforeSpecialS:
              {
                this.stateBeforeSpecialS(e);
                break;
              }
            case x.BeforeSpecialT:
              {
                this.stateBeforeSpecialT(e);
                break;
              }
            case x.InAttributeValueNq:
              {
                this.stateInAttributeValueNoQuotes(e);
                break;
              }
            case x.InSelfClosingTag:
              {
                this.stateInSelfClosingTag(e);
                break;
              }
            case x.InDeclaration:
              {
                this.stateInDeclaration(e);
                break;
              }
            case x.BeforeDeclaration:
              {
                this.stateBeforeDeclaration(e);
                break;
              }
            case x.BeforeComment:
              {
                this.stateBeforeComment(e);
                break;
              }
            case x.InProcessingInstruction:
              {
                this.stateInProcessingInstruction(e);
                break;
              }
            case x.InEntity:
              {
                this.stateInEntity();
                break;
              }
          }
          this.index++;
        }
        this.cleanup();
      }
      finish() {
        this.state === x.InEntity && (this.entityDecoder.end(), this.state = this.baseState), this.handleTrailingData(), this.cbs.onend();
      }
      handleTrailingData() {
        let e = this.buffer.length + this.offset;
        this.sectionStart >= e || (this.state === x.InCommentLike ? this.currentSequence === X.CdataEnd ? this.cbs.oncdata(this.sectionStart, e, 0) : this.cbs.oncomment(this.sectionStart, e, 0) : this.state === x.InTagName || this.state === x.BeforeAttributeName || this.state === x.BeforeAttributeValue || this.state === x.AfterAttributeName || this.state === x.InAttributeName || this.state === x.InAttributeValueSq || this.state === x.InAttributeValueDq || this.state === x.InAttributeValueNq || this.state === x.InClosingTagName || this.cbs.ontext(this.sectionStart, e));
      }
      emitCodePoint(e, t) {
        this.baseState !== x.Text && this.baseState !== x.InSpecialTag ? (this.sectionStart < this.entityStart && this.cbs.onattribdata(this.sectionStart, this.entityStart), this.sectionStart = this.entityStart + t, this.index = this.sectionStart - 1, this.cbs.onattribentity(e)) : (this.sectionStart < this.entityStart && this.cbs.ontext(this.sectionStart, this.entityStart), this.sectionStart = this.entityStart + t, this.index = this.sectionStart - 1, this.cbs.ontextentity(e, this.sectionStart));
      }
    },
    st = new Set(["input", "option", "optgroup", "select", "button", "datalist", "textarea"]),
    R = new Set(["p"]),
    ir = new Set(["thead", "tbody"]),
    sr = new Set(["dd", "dt"]),
    or = new Set(["rt", "rp"]),
    hi = new Map([["tr", new Set(["tr", "th", "td"])], ["th", new Set(["th"])], ["td", new Set(["thead", "th", "td"])], ["body", new Set(["head", "link", "script"])], ["li", new Set(["li"])], ["p", R], ["h1", R], ["h2", R], ["h3", R], ["h4", R], ["h5", R], ["h6", R], ["select", st], ["input", st], ["output", st], ["button", st], ["datalist", st], ["textarea", st], ["option", new Set(["option"])], ["optgroup", new Set(["optgroup", "option"])], ["dd", sr], ["dt", sr], ["address", R], ["article", R], ["aside", R], ["blockquote", R], ["details", R], ["div", R], ["dl", R], ["fieldset", R], ["figcaption", R], ["figure", R], ["footer", R], ["form", R], ["header", R], ["hr", R], ["main", R], ["nav", R], ["ol", R], ["pre", R], ["section", R], ["table", R], ["ul", R], ["rt", or], ["rp", or], ["tbody", ir], ["tfoot", ir]]),
    di = new Set(["area", "base", "basefont", "br", "col", "command", "embed", "frame", "hr", "img", "input", "isindex", "keygen", "link", "meta", "param", "source", "track", "wbr"]),
    Ar = new Set(["math", "svg"]),
    cr = new Set(["mi", "mo", "mn", "ms", "mtext", "annotation-xml", "foreignobject", "desc", "title"]),
    Ei = /\s|\//,
    Ci = class {
      constructor(e, t = {}) {
        var n, r, a, i, s, l;
        this.options = t, this.startIndex = 0, this.endIndex = 0, this.openTagStart = 0, this.tagname = "", this.attribname = "", this.attribvalue = "", this.attribs = null, this.stack = [], this.buffers = [], this.bufferOffset = 0, this.writeIndex = 0, this.ended = !1, this.cbs = e != null ? e : {}, this.htmlMode = !this.options.xmlMode, this.lowerCaseTagNames = (n = t.lowerCaseTags) !== null && n !== void 0 ? n : this.htmlMode, this.lowerCaseAttributeNames = (r = t.lowerCaseAttributeNames) !== null && r !== void 0 ? r : this.htmlMode, this.recognizeSelfClosing = (a = t.recognizeSelfClosing) !== null && a !== void 0 ? a : !this.htmlMode, this.tokenizer = new ((i = t.Tokenizer) !== null && i !== void 0 ? i : li)(this.options, this), this.foreignContext = [!this.htmlMode], (l = (s = this.cbs).onparserinit) === null || l === void 0 || l.call(s, this);
      }
      ontext(e, t) {
        var n, r;
        let a = this.getSlice(e, t);
        this.endIndex = t - 1, (r = (n = this.cbs).ontext) === null || r === void 0 || r.call(n, a), this.startIndex = t;
      }
      ontextentity(e, t) {
        var n, r;
        this.endIndex = t - 1, (r = (n = this.cbs).ontext) === null || r === void 0 || r.call(n, rr(e)), this.startIndex = t;
      }
      isVoidElement(e) {
        return this.htmlMode && di.has(e);
      }
      onopentagname(e, t) {
        this.endIndex = t;
        let n = this.getSlice(e, t);
        this.lowerCaseTagNames && (n = n.toLowerCase()), this.emitOpenTag(n);
      }
      emitOpenTag(e) {
        var t, n, r, a;
        this.openTagStart = this.startIndex, this.tagname = e;
        let i = this.htmlMode && hi.get(e);
        if (i) for (; this.stack.length > 0 && i.has(this.stack[0]);) {
          let s = this.stack.shift();
          (n = (t = this.cbs).onclosetag) === null || n === void 0 || n.call(t, s, !0);
        }
        this.isVoidElement(e) || (this.stack.unshift(e), this.htmlMode && (Ar.has(e) ? this.foreignContext.unshift(!0) : cr.has(e) && this.foreignContext.unshift(!1))), (a = (r = this.cbs).onopentagname) === null || a === void 0 || a.call(r, e), this.cbs.onopentag && (this.attribs = {});
      }
      endOpenTag(e) {
        var t, n;
        this.startIndex = this.openTagStart, this.attribs && ((n = (t = this.cbs).onopentag) === null || n === void 0 || n.call(t, this.tagname, this.attribs, e), this.attribs = null), this.cbs.onclosetag && this.isVoidElement(this.tagname) && this.cbs.onclosetag(this.tagname, !0), this.tagname = "";
      }
      onopentagend(e) {
        this.endIndex = e, this.endOpenTag(!1), this.startIndex = e + 1;
      }
      onclosetag(e, t) {
        var n, r, a, i, s, l, c, E;
        this.endIndex = t;
        let f = this.getSlice(e, t);
        if (this.lowerCaseTagNames && (f = f.toLowerCase()), this.htmlMode && (Ar.has(f) || cr.has(f)) && this.foreignContext.shift(), this.isVoidElement(f)) this.htmlMode && f === "br" && ((i = (a = this.cbs).onopentagname) === null || i === void 0 || i.call(a, "br"), (l = (s = this.cbs).onopentag) === null || l === void 0 || l.call(s, "br", {}, !0), (E = (c = this.cbs).onclosetag) === null || E === void 0 || E.call(c, "br", !1));else {
          let m = this.stack.indexOf(f);
          if (m !== -1) for (let D = 0; D <= m; D++) {
            let T = this.stack.shift();
            (r = (n = this.cbs).onclosetag) === null || r === void 0 || r.call(n, T, D !== m);
          } else this.htmlMode && f === "p" && (this.emitOpenTag("p"), this.closeCurrentTag(!0));
        }
        this.startIndex = t + 1;
      }
      onselfclosingtag(e) {
        this.endIndex = e, this.recognizeSelfClosing || this.foreignContext[0] ? (this.closeCurrentTag(!1), this.startIndex = e + 1) : this.onopentagend(e);
      }
      closeCurrentTag(e) {
        var t, n;
        let r = this.tagname;
        this.endOpenTag(e), this.stack[0] === r && ((n = (t = this.cbs).onclosetag) === null || n === void 0 || n.call(t, r, !e), this.stack.shift());
      }
      onattribname(e, t) {
        this.startIndex = e;
        let n = this.getSlice(e, t);
        this.attribname = this.lowerCaseAttributeNames ? n.toLowerCase() : n;
      }
      onattribdata(e, t) {
        this.attribvalue += this.getSlice(e, t);
      }
      onattribentity(e) {
        this.attribvalue += rr(e);
      }
      onattribend(e, t) {
        var n, r;
        this.endIndex = t, (r = (n = this.cbs).onattribute) === null || r === void 0 || r.call(n, this.attribname, this.attribvalue, e === De.Double ? '"' : e === De.Single ? "'" : e === De.NoValue ? void 0 : null), this.attribs && !Object.prototype.hasOwnProperty.call(this.attribs, this.attribname) && (this.attribs[this.attribname] = this.attribvalue), this.attribvalue = "";
      }
      getInstructionName(e) {
        let t = e.search(Ei),
          n = t < 0 ? e : e.substr(0, t);
        return this.lowerCaseTagNames && (n = n.toLowerCase()), n;
      }
      ondeclaration(e, t) {
        this.endIndex = t;
        let n = this.getSlice(e, t);
        if (this.cbs.onprocessinginstruction) {
          let r = this.getInstructionName(n);
          this.cbs.onprocessinginstruction("!".concat(r), "!".concat(n));
        }
        this.startIndex = t + 1;
      }
      onprocessinginstruction(e, t) {
        this.endIndex = t;
        let n = this.getSlice(e, t);
        if (this.cbs.onprocessinginstruction) {
          let r = this.getInstructionName(n);
          this.cbs.onprocessinginstruction("?".concat(r), "?".concat(n));
        }
        this.startIndex = t + 1;
      }
      oncomment(e, t, n) {
        var r, a, i, s;
        this.endIndex = t, (a = (r = this.cbs).oncomment) === null || a === void 0 || a.call(r, this.getSlice(e, t - n)), (s = (i = this.cbs).oncommentend) === null || s === void 0 || s.call(i), this.startIndex = t + 1;
      }
      oncdata(e, t, n) {
        var r, a, i, s, l, c, E, f, m, D;
        this.endIndex = t;
        let T = this.getSlice(e, t - n);
        !this.htmlMode || this.options.recognizeCDATA ? ((a = (r = this.cbs).oncdatastart) === null || a === void 0 || a.call(r), (s = (i = this.cbs).ontext) === null || s === void 0 || s.call(i, T), (c = (l = this.cbs).oncdataend) === null || c === void 0 || c.call(l)) : ((f = (E = this.cbs).oncomment) === null || f === void 0 || f.call(E, "[CDATA[".concat(T, "]]")), (D = (m = this.cbs).oncommentend) === null || D === void 0 || D.call(m)), this.startIndex = t + 1;
      }
      onend() {
        var e, t;
        if (this.cbs.onclosetag) {
          this.endIndex = this.startIndex;
          for (let n = 0; n < this.stack.length; n++) this.cbs.onclosetag(this.stack[n], !0);
        }
        (t = (e = this.cbs).onend) === null || t === void 0 || t.call(e);
      }
      reset() {
        var e, t, n, r;
        (t = (e = this.cbs).onreset) === null || t === void 0 || t.call(e), this.tokenizer.reset(), this.tagname = "", this.attribname = "", this.attribs = null, this.stack.length = 0, this.startIndex = 0, this.endIndex = 0, (r = (n = this.cbs).onparserinit) === null || r === void 0 || r.call(n, this), this.buffers.length = 0, this.foreignContext.length = 0, this.foreignContext.unshift(!this.htmlMode), this.bufferOffset = 0, this.writeIndex = 0, this.ended = !1;
      }
      parseComplete(e) {
        this.reset(), this.end(e);
      }
      getSlice(e, t) {
        for (; e - this.bufferOffset >= this.buffers[0].length;) this.shiftBuffer();
        let n = this.buffers[0].slice(e - this.bufferOffset, t - this.bufferOffset);
        for (; t - this.bufferOffset > this.buffers[0].length;) this.shiftBuffer(), n += this.buffers[0].slice(0, t - this.bufferOffset);
        return n;
      }
      shiftBuffer() {
        this.bufferOffset += this.buffers[0].length, this.writeIndex--, this.buffers.shift();
      }
      write(e) {
        var t, n;
        if (this.ended) {
          (n = (t = this.cbs).onerror) === null || n === void 0 || n.call(t, new Error(".write() after done!"));
          return;
        }
        this.buffers.push(e), this.tokenizer.running && (this.tokenizer.write(e), this.writeIndex++);
      }
      end(e) {
        var t, n;
        if (this.ended) {
          (n = (t = this.cbs).onerror) === null || n === void 0 || n.call(t, new Error(".end() after done!"));
          return;
        }
        e && this.write(e), this.ended = !0, this.tokenizer.end();
      }
      pause() {
        this.tokenizer.pause();
      }
      resume() {
        for (this.tokenizer.resume(); this.tokenizer.running && this.writeIndex < this.buffers.length;) this.tokenizer.write(this.buffers[this.writeIndex++]);
        this.ended && this.tokenizer.end();
      }
      parseChunk(e) {
        this.write(e);
      }
      done(e) {
        this.end(e);
      }
    };
  function pi(e, t) {
    let n = new qu(void 0, t);
    return new Ci(n, t).end(e), n.root;
  }
  var Hn,
    ft = (Hn = Object.hasOwn) !== null && Hn !== void 0 ? Hn : (e, t) => Object.prototype.hasOwnProperty.call(e, t),
    xt = /\s+/,
    Pn = "data-",
    Un = /^(?:autofocus|autoplay|async|checked|controls|defer|disabled|hidden|loop|multiple|open|readonly|required|scoped|selected)$/i,
    fi = /^{[^]*}$|^\[[^]*]$/;
  function Vt(e, t, n) {
    var r;
    if (!(!e || !y(e))) {
      if ((r = e.attribs) !== null && r !== void 0 || (e.attribs = {}), !t) return e.attribs;
      if (ft(e.attribs, t)) return !n && Un.test(t) ? t : e.attribs[t];
      if (e.name === "option" && t === "value") return pt(e.children);
      if (e.name === "input" && (e.attribs.type === "radio" || e.attribs.type === "checkbox") && t === "value") return "on";
    }
  }
  function ot(e, t, n) {
    n === null ? Er(e, t) : e.attribs[t] = "".concat(n);
  }
  function xi(e, t) {
    if (typeof e == "object" || t !== void 0) {
      if (typeof t == "function") {
        if (typeof e != "string") throw new Error("Bad combination of arguments.");
        return P(this, (n, r) => {
          y(n) && ot(n, e, t.call(n, r, n.attribs[e]));
        });
      }
      return P(this, n => {
        if (y(n)) if (typeof e == "object") for (let r of Object.keys(e)) {
          let a = e[r];
          ot(n, r, a);
        } else ot(n, e, t);
      });
    }
    return arguments.length > 1 ? this : Vt(this[0], e, this.options.xmlMode);
  }
  function lr(e, t, n) {
    return t in e ? e[t] : !n && Un.test(t) ? Vt(e, t, !1) !== void 0 : Vt(e, t, n);
  }
  function Wn(e, t, n, r) {
    t in e ? e[t] = n : ot(e, t, !r && Un.test(t) ? n ? "" : null : "".concat(n));
  }
  function Bi(e, t) {
    var n;
    if (typeof e == "string" && t === void 0) {
      let r = this[0];
      if (!r) return;
      switch (e) {
        case "style":
          {
            let a = this.css(),
              i = Object.keys(a);
            for (let s = 0; s < i.length; s++) a[s] = i[s];
            return a.length = i.length, a;
          }
        case "tagName":
        case "nodeName":
          return y(r) ? r.name.toUpperCase() : void 0;
        case "href":
        case "src":
          {
            if (!y(r)) return;
            let a = (n = r.attribs) === null || n === void 0 ? void 0 : n[e];
            return typeof URL < "u" && (e === "href" && (r.tagName === "a" || r.tagName === "link") || e === "src" && (r.tagName === "img" || r.tagName === "iframe" || r.tagName === "audio" || r.tagName === "video" || r.tagName === "source")) && a !== void 0 && this.options.baseURI ? new URL(a, this.options.baseURI).href : a;
          }
        case "innerText":
          return Qt(r);
        case "textContent":
          return ut(r);
        case "outerHTML":
          return r.type === qe.Root ? this.html() : this.clone().wrap("<container />").parent().html();
        case "innerHTML":
          return this.html();
        default:
          return y(r) ? lr(r, e, this.options.xmlMode) : void 0;
      }
    }
    if (typeof e == "object" || t !== void 0) {
      if (typeof t == "function") {
        if (typeof e == "object") throw new TypeError("Bad combination of arguments.");
        return P(this, (r, a) => {
          y(r) && Wn(r, e, t.call(r, a, lr(r, e, this.options.xmlMode)), this.options.xmlMode);
        });
      }
      return P(this, r => {
        if (y(r)) if (typeof e == "object") for (let a of Object.keys(e)) {
          let i = e[a];
          Wn(r, a, i, this.options.xmlMode);
        } else Wn(r, e, t, this.options.xmlMode);
      });
    }
  }
  function hr(e, t, n) {
    var r;
    (r = e.data) !== null && r !== void 0 || (e.data = {}), typeof t == "object" ? Object.assign(e.data, t) : typeof t == "string" && n !== void 0 && (e.data[t] = n);
  }
  function mi(e) {
    for (let t of Object.keys(e.attribs)) {
      if (!t.startsWith(Pn)) continue;
      let n = $a(t.slice(Pn.length));
      ft(e.data, n) || (e.data[n] = dr(e.attribs[t]));
    }
    return e.data;
  }
  function Di(e, t) {
    let n = Pn + ei(t),
      r = e.data;
    if (ft(r, t)) return r[t];
    if (ft(e.attribs, n)) return r[t] = dr(e.attribs[n]);
  }
  function dr(e) {
    if (e === "null") return null;
    if (e === "true") return !0;
    if (e === "false") return !1;
    let t = Number(e);
    if (e === String(t)) return t;
    if (fi.test(e)) try {
      return JSON.parse(e);
    } catch (n) {}
    return e;
  }
  function gi(e, t) {
    var n;
    let r = this[0];
    if (!r || !y(r)) return;
    let a = r;
    return (n = a.data) !== null && n !== void 0 || (a.data = {}), e == null ? mi(a) : typeof e == "object" || t !== void 0 ? (P(this, i => {
      y(i) && (typeof e == "object" ? hr(i, e) : hr(i, e, t));
    }), this) : Di(a, e);
  }
  function Ti(e) {
    let t = arguments.length === 0,
      n = this[0];
    if (!n || !y(n)) return t ? void 0 : this;
    switch (n.name) {
      case "textarea":
        return this.text(e);
      case "select":
        {
          let r = this.find("option:selected");
          if (!t) {
            if (this.attr("multiple") == null && typeof e == "object") return this;
            this.find("option").removeAttr("selected");
            let a = typeof e == "object" ? e : [e];
            for (let i of a) this.find('option[value="'.concat(i, '"]')).attr("selected", "");
            return this;
          }
          return this.attr("multiple") ? r.toArray().map(a => pt(a.children)) : r.attr("value");
        }
      case "button":
      case "input":
      case "option":
        return t ? this.attr("value") : this.attr("value", e);
    }
  }
  function Er(e, t) {
    !e.attribs || !ft(e.attribs, t) || delete e.attribs[t];
  }
  function jt(e) {
    return e ? e.trim().split(xt) : [];
  }
  function Fi(e) {
    let t = jt(e);
    for (let n of t) P(this, r => {
      y(r) && Er(r, n);
    });
    return this;
  }
  function _i(e) {
    return this.toArray().some(t => {
      let n = y(t) && t.attribs.class,
        r = -1;
      if (n && e.length > 0) for (; (r = n.indexOf(e, r + 1)) > -1;) {
        let a = r + e.length;
        if ((r === 0 || xt.test(n[r - 1])) && (a === n.length || xt.test(n[a]))) return !0;
      }
      return !1;
    });
  }
  function Cr(e) {
    if (typeof e == "function") return P(this, (r, a) => {
      if (y(r)) {
        let i = r.attribs.class || "";
        Cr.call([r], e.call(r, a, i));
      }
    });
    if (!e || typeof e != "string") return this;
    let t = e.split(xt),
      n = this.length;
    for (let r = 0; r < n; r++) {
      let a = this[r];
      if (!y(a)) continue;
      let i = Vt(a, "class", !1);
      if (i) {
        let s = " ".concat(i, " ");
        for (let l of t) {
          let c = "".concat(l, " ");
          s.includes(" ".concat(c)) || (s += c);
        }
        ot(a, "class", s.trim());
      } else ot(a, "class", t.join(" ").trim());
    }
    return this;
  }
  function pr(e) {
    if (typeof e == "function") return P(this, (a, i) => {
      y(a) && pr.call([a], e.call(a, i, a.attribs.class || ""));
    });
    let t = jt(e),
      n = t.length,
      r = arguments.length === 0;
    return P(this, a => {
      if (y(a)) if (r) a.attribs.class = "";else {
        let i = jt(a.attribs.class),
          s = !1;
        for (let l = 0; l < n; l++) {
          let c = i.indexOf(t[l]);
          c !== -1 && (i.splice(c, 1), s = !0, l--);
        }
        s && (a.attribs.class = i.join(" "));
      }
    });
  }
  function fr(e, t) {
    if (typeof e == "function") return P(this, (s, l) => {
      y(s) && fr.call([s], e.call(s, l, s.attribs.class || "", t), t);
    });
    if (!e || typeof e != "string") return this;
    let n = e.split(xt),
      r = n.length,
      a = typeof t == "boolean" ? t ? 1 : -1 : 0,
      i = this.length;
    for (let s = 0; s < i; s++) {
      let l = this[s];
      if (!y(l)) continue;
      let c = jt(l.attribs.class);
      for (let E = 0; E < r; E++) {
        let f = c.indexOf(n[E]);
        a >= 0 && f === -1 ? c.push(n[E]) : a <= 0 && f !== -1 && c.splice(f, 1);
      }
      l.attribs.class = c.join(" ");
    }
    return this;
  }
  var xr = {};
  le(xr, {
    _findBySelector: () => ss,
    add: () => Os,
    addBack: () => Ls,
    children: () => Bs,
    closest: () => ls,
    contents: () => ms,
    each: () => Ds,
    end: () => Rs,
    eq: () => Is,
    filter: () => Ts,
    filterArray: () => o0,
    find: () => is,
    first: () => bs,
    get: () => vs,
    has: () => ys,
    index: () => ks,
    is: () => Fs,
    last: () => Ss,
    map: () => gs,
    next: () => hs,
    nextAll: () => ds,
    nextUntil: () => Es,
    not: () => _s,
    parent: () => os,
    parents: () => As,
    parentsUntil: () => cs,
    prev: () => Cs,
    prevAll: () => ps,
    prevUntil: () => fs,
    siblings: () => xs,
    slice: () => ws,
    toArray: () => Ns
  });
  var _;
  (function (e) {
    e.Attribute = "attribute", e.Pseudo = "pseudo", e.PseudoElement = "pseudo-element", e.Tag = "tag", e.Universal = "universal", e.Adjacent = "adjacent", e.Child = "child", e.Descendant = "descendant", e.Parent = "parent", e.Sibling = "sibling", e.ColumnCombinator = "column-combinator";
  })(_ || (_ = {}));
  var J;
  (function (e) {
    e.Any = "any", e.Element = "element", e.End = "end", e.Equals = "equals", e.Exists = "exists", e.Hyphen = "hyphen", e.Not = "not", e.Start = "start";
  })(J || (J = {}));
  var Br = /^[^\\#]?(?:\\(?:[\da-f]{1,6}\s?|.)|[\w\-\u00b0-\uFFFF])+/,
    yi = /\\([\da-f]{1,6}\s?|(\s)|.)/gi,
    bi = new Map([[126, J.Element], [94, J.Start], [36, J.End], [42, J.Any], [33, J.Not], [124, J.Hyphen]]),
    Si = new Set(["has", "not", "matches", "is", "where", "host", "host-context"]);
  function Bt(e) {
    switch (e.type) {
      case _.Adjacent:
      case _.Child:
      case _.Descendant:
      case _.Parent:
      case _.Sibling:
      case _.ColumnCombinator:
        return !0;
      default:
        return !1;
    }
  }
  var Ii = new Set(["contains", "icontains"]);
  function vi(e, t, n) {
    let r = parseInt(t, 16) - 65536;
    return r !== r || n ? t : r < 0 ? String.fromCharCode(r + 65536) : String.fromCharCode(r >> 10 | 55296, r & 1023 | 56320);
  }
  function mt(e) {
    return e.replace(yi, vi);
  }
  function Gn(e) {
    return e === 39 || e === 34;
  }
  function mr(e) {
    return e === 32 || e === 9 || e === 10 || e === 12 || e === 13;
  }
  function Dt(e) {
    let t = [],
      n = Dr(t, "".concat(e), 0);
    if (n < e.length) throw new Error("Unmatched selector: ".concat(e.slice(n)));
    return t;
  }
  function Dr(e, t, n) {
    let r = [];
    function a(D) {
      let T = t.slice(n + D).match(Br);
      if (!T) throw new Error("Expected name, found ".concat(t.slice(n)));
      let b = _slicedToArray(T, 1),
        Y = b[0];
      return n += D + Y.length, mt(Y);
    }
    function i(D) {
      for (n += D; n < t.length && mr(t.charCodeAt(n));) n++;
    }
    function s() {
      n += 1;
      let D = n,
        T = 1;
      for (; T > 0 && n < t.length; n++) t.charCodeAt(n) === 40 && !l(n) ? T++ : t.charCodeAt(n) === 41 && !l(n) && T--;
      if (T) throw new Error("Parenthesis not matched");
      return mt(t.slice(D, n - 1));
    }
    function l(D) {
      let T = 0;
      for (; t.charCodeAt(--D) === 92;) T++;
      return (T & 1) === 1;
    }
    function c() {
      if (r.length > 0 && Bt(r[r.length - 1])) throw new Error("Did not expect successive traversals.");
    }
    function E(D) {
      if (r.length > 0 && r[r.length - 1].type === _.Descendant) {
        r[r.length - 1].type = D;
        return;
      }
      c(), r.push({
        type: D
      });
    }
    function f(D, T) {
      r.push({
        type: _.Attribute,
        name: D,
        action: T,
        value: a(1),
        namespace: null,
        ignoreCase: "quirks"
      });
    }
    function m() {
      if (r.length && r[r.length - 1].type === _.Descendant && r.pop(), r.length === 0) throw new Error("Empty sub-selector");
      e.push(r);
    }
    if (i(0), t.length === n) return n;
    e: for (; n < t.length;) {
      let D = t.charCodeAt(n);
      switch (D) {
        case 32:
        case 9:
        case 10:
        case 12:
        case 13:
          {
            (r.length === 0 || r[0].type !== _.Descendant) && (c(), r.push({
              type: _.Descendant
            })), i(1);
            break;
          }
        case 62:
          {
            E(_.Child), i(1);
            break;
          }
        case 60:
          {
            E(_.Parent), i(1);
            break;
          }
        case 126:
          {
            E(_.Sibling), i(1);
            break;
          }
        case 43:
          {
            E(_.Adjacent), i(1);
            break;
          }
        case 46:
          {
            f("class", J.Element);
            break;
          }
        case 35:
          {
            f("id", J.Equals);
            break;
          }
        case 91:
          {
            i(1);
            let T,
              b = null;
            t.charCodeAt(n) === 124 ? T = a(1) : t.startsWith("*|", n) ? (b = "*", T = a(2)) : (T = a(0), t.charCodeAt(n) === 124 && t.charCodeAt(n + 1) !== 61 && (b = T, T = a(1))), i(0);
            let Y = J.Exists,
              re = bi.get(t.charCodeAt(n));
            if (re) {
              if (Y = re, t.charCodeAt(n + 1) !== 61) throw new Error("Expected `=`");
              i(2);
            } else t.charCodeAt(n) === 61 && (Y = J.Equals, i(1));
            let k = "",
              g = null;
            if (Y !== "exists") {
              if (Gn(t.charCodeAt(n))) {
                let ee = t.charCodeAt(n),
                  q = n + 1;
                for (; q < t.length && (t.charCodeAt(q) !== ee || l(q));) q += 1;
                if (t.charCodeAt(q) !== ee) throw new Error("Attribute value didn't end");
                k = mt(t.slice(n + 1, q)), n = q + 1;
              } else {
                let ee = n;
                for (; n < t.length && (!mr(t.charCodeAt(n)) && t.charCodeAt(n) !== 93 || l(n));) n += 1;
                k = mt(t.slice(ee, n));
              }
              i(0);
              let te = t.charCodeAt(n) | 32;
              te === 115 ? (g = !1, i(1)) : te === 105 && (g = !0, i(1));
            }
            if (t.charCodeAt(n) !== 93) throw new Error("Attribute selector didn't terminate");
            n += 1;
            let I = {
              type: _.Attribute,
              name: T,
              action: Y,
              value: k,
              namespace: b,
              ignoreCase: g
            };
            r.push(I);
            break;
          }
        case 58:
          {
            if (t.charCodeAt(n + 1) === 58) {
              r.push({
                type: _.PseudoElement,
                name: a(2).toLowerCase(),
                data: t.charCodeAt(n) === 40 ? s() : null
              });
              continue;
            }
            let T = a(1).toLowerCase(),
              b = null;
            if (t.charCodeAt(n) === 40) if (Si.has(T)) {
              if (Gn(t.charCodeAt(n + 1))) throw new Error("Pseudo-selector ".concat(T, " cannot be quoted"));
              if (b = [], n = Dr(b, t, n + 1), t.charCodeAt(n) !== 41) throw new Error("Missing closing parenthesis in :".concat(T, " (").concat(t, ")"));
              n += 1;
            } else {
              if (b = s(), Ii.has(T)) {
                let Y = b.charCodeAt(0);
                Y === b.charCodeAt(b.length - 1) && Gn(Y) && (b = b.slice(1, -1));
              }
              b = mt(b);
            }
            r.push({
              type: _.Pseudo,
              name: T,
              data: b
            });
            break;
          }
        case 44:
          {
            m(), r = [], i(1);
            break;
          }
        default:
          {
            if (t.startsWith("/*", n)) {
              let Y = t.indexOf("*/", n + 2);
              if (Y < 0) throw new Error("Comment was not terminated");
              n = Y + 2, r.length === 0 && i(0);
              break;
            }
            let T = null,
              b;
            if (D === 42) n += 1, b = "*";else if (D === 124) {
              if (b = "", t.charCodeAt(n + 1) === 124) {
                E(_.ColumnCombinator), i(2);
                break;
              }
            } else if (Br.test(t.slice(n))) b = a(0);else break e;
            t.charCodeAt(n) === 124 && t.charCodeAt(n + 1) !== 124 && (T = b, t.charCodeAt(n + 1) === 42 ? (b = "*", n += 2) : b = a(1)), r.push(b === "*" ? {
              type: _.Universal,
              namespace: T
            } : {
              type: _.Tag,
              name: b,
              namespace: T
            });
          }
      }
    }
    return m(), n;
  }
  var gr = Xe(Je(), 1),
    He = Xe(Je(), 1),
    Tr = new Map([[_.Universal, 50], [_.Tag, 30], [_.Attribute, 1], [_.Pseudo, 0]]);
  function Kn(e) {
    return !Tr.has(e.type);
  }
  var Ni = new Map([[J.Exists, 10], [J.Equals, 8], [J.Not, 7], [J.Start, 6], [J.End, 6], [J.Any, 5]]);
  function ki(e) {
    let t = e.map(Fr);
    for (let n = 1; n < e.length; n++) {
      let r = t[n];
      if (!(r < 0)) for (let a = n - 1; a >= 0 && r < t[a]; a--) {
        let i = e[a + 1];
        e[a + 1] = e[a], e[a] = i, t[a + 1] = t[a], t[a] = r;
      }
    }
  }
  function Fr(e) {
    var t, n;
    let r = (t = Tr.get(e.type)) !== null && t !== void 0 ? t : -1;
    return e.type === _.Attribute ? (r = (n = Ni.get(e.action)) !== null && n !== void 0 ? n : 4, e.action === J.Equals && e.name === "id" && (r = 9), e.ignoreCase && (r >>= 1)) : e.type === _.Pseudo && (e.data ? e.name === "has" || e.name === "contains" ? r = 0 : Array.isArray(e.data) ? (r = Math.min(...e.data.map(a => Math.min(...a.map(Fr)))), r < 0 && (r = 0)) : r = 2 : r = 3), r;
  }
  var Zt = Xe(Je(), 1),
    wi = /[-[\]{}()*+?.,\\^$|#\s]/g;
  function _r(e) {
    return e.replace(wi, "\\$&");
  }
  var Ri = new Set(["accept", "accept-charset", "align", "alink", "axis", "bgcolor", "charset", "checked", "clear", "codetype", "color", "compact", "declare", "defer", "dir", "direction", "disabled", "enctype", "face", "frame", "hreflang", "http-equiv", "lang", "language", "link", "media", "method", "multiple", "nohref", "noresize", "noshade", "nowrap", "readonly", "rel", "rev", "rules", "scope", "scrolling", "selected", "shape", "target", "text", "type", "valign", "valuetype", "vlink"]);
  function $e(e, t) {
    return typeof e.ignoreCase == "boolean" ? e.ignoreCase : e.ignoreCase === "quirks" ? !!t.quirksMode : !t.xmlMode && Ri.has(e.name);
  }
  var Oi = {
      equals(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value;
        return $e(t, n) ? (i = i.toLowerCase(), s => {
          let l = r.getAttributeValue(s, a);
          return l != null && l.length === i.length && l.toLowerCase() === i && e(s);
        }) : s => r.getAttributeValue(s, a) === i && e(s);
      },
      hyphen(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value,
          s = i.length;
        return $e(t, n) ? (i = i.toLowerCase(), function (l) {
          let c = r.getAttributeValue(l, a);
          return c != null && (c.length === s || c.charAt(s) === "-") && c.substr(0, s).toLowerCase() === i && e(l);
        }) : function (l) {
          let c = r.getAttributeValue(l, a);
          return c != null && (c.length === s || c.charAt(s) === "-") && c.substr(0, s) === i && e(l);
        };
      },
      element(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value;
        if (/\s/.test(i)) return Zt.default.falseFunc;
        let s = new RegExp("(?:^|\\s)".concat(_r(i), "(?:$|\\s)"), $e(t, n) ? "i" : "");
        return function (l) {
          let c = r.getAttributeValue(l, a);
          return c != null && c.length >= i.length && s.test(c) && e(l);
        };
      },
      exists(e, {
        name: t
      }, {
        adapter: n
      }) {
        return r => n.hasAttrib(r, t) && e(r);
      },
      start(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value,
          s = i.length;
        return s === 0 ? Zt.default.falseFunc : $e(t, n) ? (i = i.toLowerCase(), l => {
          let c = r.getAttributeValue(l, a);
          return c != null && c.length >= s && c.substr(0, s).toLowerCase() === i && e(l);
        }) : l => {
          var c;
          return !!(!((c = r.getAttributeValue(l, a)) === null || c === void 0) && c.startsWith(i)) && e(l);
        };
      },
      end(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value,
          s = -i.length;
        return s === 0 ? Zt.default.falseFunc : $e(t, n) ? (i = i.toLowerCase(), l => {
          var c;
          return ((c = r.getAttributeValue(l, a)) === null || c === void 0 ? void 0 : c.substr(s).toLowerCase()) === i && e(l);
        }) : l => {
          var c;
          return !!(!((c = r.getAttributeValue(l, a)) === null || c === void 0) && c.endsWith(i)) && e(l);
        };
      },
      any(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value;
        if (i === "") return Zt.default.falseFunc;
        if ($e(t, n)) {
          let s = new RegExp(_r(i), "i");
          return function (l) {
            let c = r.getAttributeValue(l, a);
            return c != null && c.length >= i.length && s.test(c) && e(l);
          };
        }
        return s => {
          var l;
          return !!(!((l = r.getAttributeValue(s, a)) === null || l === void 0) && l.includes(i)) && e(s);
        };
      },
      not(e, t, n) {
        let r = n.adapter,
          a = t.name,
          i = t.value;
        return i === "" ? s => !!r.getAttributeValue(s, a) && e(s) : $e(t, n) ? (i = i.toLowerCase(), s => {
          let l = r.getAttributeValue(s, a);
          return (l == null || l.length !== i.length || l.toLowerCase() !== i) && e(s);
        }) : s => r.getAttributeValue(s, a) !== i && e(s);
      }
    },
    Li = new Set([9, 10, 12, 13, 32]),
    yr = 48,
    Mi = 57;
  function Hi(e) {
    if (e = e.trim().toLowerCase(), e === "even") return [2, 0];
    if (e === "odd") return [2, 1];
    let t = 0,
      n = 0,
      r = i(),
      a = s();
    if (t < e.length && e.charAt(t) === "n" && (t++, n = r * (a != null ? a : 1), l(), t < e.length ? (r = i(), l(), a = s()) : r = a = 0), a === null || t < e.length) throw new Error("n-th rule couldn't be parsed ('".concat(e, "')"));
    return [n, r * a];
    function i() {
      return e.charAt(t) === "-" ? (t++, -1) : (e.charAt(t) === "+" && t++, 1);
    }
    function s() {
      let c = t,
        E = 0;
      for (; t < e.length && e.charCodeAt(t) >= yr && e.charCodeAt(t) <= Mi;) E = E * 10 + (e.charCodeAt(t) - yr), t++;
      return t === c ? null : E;
    }
    function l() {
      for (; t < e.length && Li.has(e.charCodeAt(t));) t++;
    }
  }
  var br = Xe(Je(), 1);
  function Pi(e) {
    let t = e[0],
      n = e[1] - 1;
    if (n < 0 && t <= 0) return br.default.falseFunc;
    if (t === -1) return i => i <= n;
    if (t === 0) return i => i === n;
    if (t === 1) return n < 0 ? br.default.trueFunc : i => i >= n;
    let r = Math.abs(t),
      a = (n % r + r) % r;
    return t > 1 ? i => i >= n && i % r === a : i => i <= n && i % r === a;
  }
  function zt(e) {
    return Pi(Hi(e));
  }
  var he = Xe(Je(), 1);
  function $t(e, t) {
    return n => {
      let r = t.getParent(n);
      return r != null && t.isTag(r) && e(n);
    };
  }
  var Qn = {
    contains(e, t, {
      adapter: n
    }) {
      return function (r) {
        return e(r) && n.getText(r).includes(t);
      };
    },
    icontains(e, t, {
      adapter: n
    }) {
      let r = t.toLowerCase();
      return function (a) {
        return e(a) && n.getText(a).toLowerCase().includes(r);
      };
    },
    "nth-child"(e, t, {
      adapter: n,
      equals: r
    }) {
      let a = zt(t);
      return a === he.default.falseFunc ? he.default.falseFunc : a === he.default.trueFunc ? $t(e, n) : function (i) {
        let s = n.getSiblings(i),
          l = 0;
        for (let c = 0; c < s.length && !r(i, s[c]); c++) n.isTag(s[c]) && l++;
        return a(l) && e(i);
      };
    },
    "nth-last-child"(e, t, {
      adapter: n,
      equals: r
    }) {
      let a = zt(t);
      return a === he.default.falseFunc ? he.default.falseFunc : a === he.default.trueFunc ? $t(e, n) : function (i) {
        let s = n.getSiblings(i),
          l = 0;
        for (let c = s.length - 1; c >= 0 && !r(i, s[c]); c--) n.isTag(s[c]) && l++;
        return a(l) && e(i);
      };
    },
    "nth-of-type"(e, t, {
      adapter: n,
      equals: r
    }) {
      let a = zt(t);
      return a === he.default.falseFunc ? he.default.falseFunc : a === he.default.trueFunc ? $t(e, n) : function (i) {
        let s = n.getSiblings(i),
          l = 0;
        for (let c = 0; c < s.length; c++) {
          let E = s[c];
          if (r(i, E)) break;
          n.isTag(E) && n.getName(E) === n.getName(i) && l++;
        }
        return a(l) && e(i);
      };
    },
    "nth-last-of-type"(e, t, {
      adapter: n,
      equals: r
    }) {
      let a = zt(t);
      return a === he.default.falseFunc ? he.default.falseFunc : a === he.default.trueFunc ? $t(e, n) : function (i) {
        let s = n.getSiblings(i),
          l = 0;
        for (let c = s.length - 1; c >= 0; c--) {
          let E = s[c];
          if (r(i, E)) break;
          n.isTag(E) && n.getName(E) === n.getName(i) && l++;
        }
        return a(l) && e(i);
      };
    },
    root(e, t, {
      adapter: n
    }) {
      return r => {
        let a = n.getParent(r);
        return (a == null || !n.isTag(a)) && e(r);
      };
    },
    scope(e, t, n, r) {
      let a = n.equals;
      return !r || r.length === 0 ? Qn.root(e, t, n) : r.length === 1 ? i => a(r[0], i) && e(i) : i => r.includes(i) && e(i);
    },
    hover: Yn("isHovered"),
    visited: Yn("isVisited"),
    active: Yn("isActive")
  };
  function Yn(e) {
    return function (t, n, {
      adapter: r
    }) {
      let a = r[e];
      return typeof a != "function" ? he.default.falseFunc : function (i) {
        return a(i) && t(i);
      };
    };
  }
  var Sr = {
    empty(e, {
      adapter: t
    }) {
      return !t.getChildren(e).some(n => t.isTag(n) || t.getText(n) !== "");
    },
    "first-child"(e, {
      adapter: t,
      equals: n
    }) {
      if (t.prevElementSibling) return t.prevElementSibling(e) == null;
      let r = t.getSiblings(e).find(a => t.isTag(a));
      return r != null && n(e, r);
    },
    "last-child"(e, {
      adapter: t,
      equals: n
    }) {
      let r = t.getSiblings(e);
      for (let a = r.length - 1; a >= 0; a--) {
        if (n(e, r[a])) return !0;
        if (t.isTag(r[a])) break;
      }
      return !1;
    },
    "first-of-type"(e, {
      adapter: t,
      equals: n
    }) {
      let r = t.getSiblings(e),
        a = t.getName(e);
      for (let i = 0; i < r.length; i++) {
        let s = r[i];
        if (n(e, s)) return !0;
        if (t.isTag(s) && t.getName(s) === a) break;
      }
      return !1;
    },
    "last-of-type"(e, {
      adapter: t,
      equals: n
    }) {
      let r = t.getSiblings(e),
        a = t.getName(e);
      for (let i = r.length - 1; i >= 0; i--) {
        let s = r[i];
        if (n(e, s)) return !0;
        if (t.isTag(s) && t.getName(s) === a) break;
      }
      return !1;
    },
    "only-of-type"(e, {
      adapter: t,
      equals: n
    }) {
      let r = t.getName(e);
      return t.getSiblings(e).every(a => n(e, a) || !t.isTag(a) || t.getName(a) !== r);
    },
    "only-child"(e, {
      adapter: t,
      equals: n
    }) {
      return t.getSiblings(e).every(r => n(e, r) || !t.isTag(r));
    }
  };
  function Ir(e, t, n, r) {
    if (n === null) {
      if (e.length > r) throw new Error("Pseudo-class :".concat(t, " requires an argument"));
    } else if (e.length === r) throw new Error("Pseudo-class :".concat(t, " doesn't have any arguments"));
  }
  var Ui = {
      "any-link": ":is(a, area, link)[href]",
      link: ":any-link:not(:visited)",
      disabled: ":is(\n        :is(button, input, select, textarea, optgroup, option)[disabled],\n        optgroup[disabled] > option,\n        fieldset[disabled]:not(fieldset[disabled] legend:first-of-type *)\n    )",
      enabled: ":not(:disabled)",
      checked: ":is(:is(input[type=radio], input[type=checkbox])[checked], option:selected)",
      required: ":is(input, select, textarea)[required]",
      optional: ":is(input, select, textarea):not([required])",
      selected: "option:is([selected], select:not([multiple]):not(:has(> option[selected])) > :first-of-type)",
      checkbox: "[type=checkbox]",
      file: "[type=file]",
      password: "[type=password]",
      radio: "[type=radio]",
      reset: "[type=reset]",
      image: "[type=image]",
      submit: "[type=submit]",
      parent: ":not(:empty)",
      header: ":is(h1, h2, h3, h4, h5, h6)",
      button: ":is(button, input[type=button])",
      input: ":is(input, textarea, select, button)",
      text: "input:is(:not([type!='']), [type=text])"
    },
    fe = Xe(Je(), 1),
    vr = {};
  function Nr(e, t) {
    return e === fe.default.falseFunc ? fe.default.falseFunc : n => t.isTag(n) && e(n);
  }
  function kr(e, t) {
    let n = t.getSiblings(e);
    if (n.length <= 1) return [];
    let r = n.indexOf(e);
    return r < 0 || r === n.length - 1 ? [] : n.slice(r + 1).filter(t.isTag);
  }
  function Xn(e) {
    return {
      xmlMode: !!e.xmlMode,
      lowerCaseAttributeNames: !!e.lowerCaseAttributeNames,
      lowerCaseTags: !!e.lowerCaseTags,
      quirksMode: !!e.quirksMode,
      cacheResults: !!e.cacheResults,
      pseudos: e.pseudos,
      adapter: e.adapter,
      equals: e.equals
    };
  }
  var Jn = (e, t, n, r, a) => {
      let i = a(t, Xn(n), r);
      return i === fe.default.trueFunc ? e : i === fe.default.falseFunc ? fe.default.falseFunc : s => i(s) && e(s);
    },
    qn = {
      is: Jn,
      matches: Jn,
      where: Jn,
      not(e, t, n, r, a) {
        let i = a(t, Xn(n), r);
        return i === fe.default.falseFunc ? e : i === fe.default.trueFunc ? fe.default.falseFunc : s => !i(s) && e(s);
      },
      has(e, t, n, r, a) {
        let i = n.adapter,
          s = Xn(n);
        s.relativeSelector = !0;
        let l = t.some(f => f.some(Kn)) ? [vr] : void 0,
          c = a(t, s, l);
        if (c === fe.default.falseFunc) return fe.default.falseFunc;
        let E = Nr(c, i);
        if (l && c !== fe.default.trueFunc) {
          let f = c.shouldTestNextSiblings,
            m = f === void 0 ? !1 : f;
          return D => {
            if (!e(D)) return !1;
            l[0] = D;
            let T = i.getChildren(D),
              b = m ? [...T, ...kr(D, i)] : T;
            return i.existsOne(E, b);
          };
        }
        return f => e(f) && i.existsOne(E, i.getChildren(f));
      }
    };
  function Wi(e, t, n, r, a) {
    var i;
    let s = t.name,
      l = t.data;
    if (Array.isArray(l)) {
      if (!(s in qn)) throw new Error("Unknown pseudo-class :".concat(s, "(").concat(l, ")"));
      return qn[s](e, l, n, r, a);
    }
    let c = (i = n.pseudos) === null || i === void 0 ? void 0 : i[s],
      E = typeof c == "string" ? c : Ui[s];
    if (typeof E == "string") {
      if (l != null) throw new Error("Pseudo ".concat(s, " doesn't have any arguments"));
      let f = Dt(E);
      return qn.is(e, f, n, r, a);
    }
    if (typeof c == "function") return Ir(c, s, l, 1), f => c(f, l) && e(f);
    if (s in Qn) return Qn[s](e, l, n, r);
    if (s in Sr) {
      let f = Sr[s];
      return Ir(f, s, l, 2), m => f(m, n, l) && e(m);
    }
    throw new Error("Unknown pseudo-class :".concat(s));
  }
  function Vn(e, t) {
    let n = t.getParent(e);
    return n && t.isTag(n) ? n : null;
  }
  function Gi(e, t, n, r, a) {
    let i = n.adapter,
      s = n.equals;
    switch (t.type) {
      case _.PseudoElement:
        throw new Error("Pseudo-elements are not supported by css-select");
      case _.ColumnCombinator:
        throw new Error("Column combinators are not yet supported by css-select");
      case _.Attribute:
        {
          if (t.namespace != null) throw new Error("Namespaced attributes are not yet supported by css-select");
          return (!n.xmlMode || n.lowerCaseAttributeNames) && (t.name = t.name.toLowerCase()), Oi[t.action](e, t, n);
        }
      case _.Pseudo:
        return Wi(e, t, n, r, a);
      case _.Tag:
        {
          if (t.namespace != null) throw new Error("Namespaced tag names are not yet supported by css-select");
          let l = t.name;
          return (!n.xmlMode || n.lowerCaseTags) && (l = l.toLowerCase()), function (c) {
            return i.getName(c) === l && e(c);
          };
        }
      case _.Descendant:
        {
          if (n.cacheResults === !1 || typeof WeakSet > "u") return function (c) {
            let E = c;
            for (; E = Vn(E, i);) if (e(E)) return !0;
            return !1;
          };
          let l = new WeakSet();
          return function (c) {
            let E = c;
            for (; E = Vn(E, i);) if (!l.has(E)) {
              if (i.isTag(E) && e(E)) return !0;
              l.add(E);
            }
            return !1;
          };
        }
      case "_flexibleDescendant":
        return function (l) {
          let c = l;
          do if (e(c)) return !0; while (c = Vn(c, i));
          return !1;
        };
      case _.Parent:
        return function (l) {
          return i.getChildren(l).some(c => i.isTag(c) && e(c));
        };
      case _.Child:
        return function (l) {
          let c = i.getParent(l);
          return c != null && i.isTag(c) && e(c);
        };
      case _.Sibling:
        return function (l) {
          let c = i.getSiblings(l);
          for (let E = 0; E < c.length; E++) {
            let f = c[E];
            if (s(l, f)) break;
            if (i.isTag(f) && e(f)) return !0;
          }
          return !1;
        };
      case _.Adjacent:
        return i.prevElementSibling ? function (l) {
          let c = i.prevElementSibling(l);
          return c != null && e(c);
        } : function (l) {
          let c = i.getSiblings(l),
            E;
          for (let f = 0; f < c.length; f++) {
            let m = c[f];
            if (s(l, m)) break;
            i.isTag(m) && (E = m);
          }
          return !!E && e(E);
        };
      case _.Universal:
        {
          if (t.namespace != null && t.namespace !== "*") throw new Error("Namespaced universal selectors are not yet supported by css-select");
          return e;
        }
    }
  }
  function Ki(e, t, n) {
    let r = jn(e, t, n);
    return Nr(r, t.adapter);
  }
  function jn(e, t, n) {
    let r = typeof e == "string" ? Dt(e) : e;
    return Zn(r, t, n);
  }
  function wr(e) {
    return e.type === _.Pseudo && (e.name === "scope" || Array.isArray(e.data) && e.data.some(t => t.some(wr)));
  }
  var Qi = {
      type: _.Descendant
    },
    Yi = {
      type: "_flexibleDescendant"
    },
    Xi = {
      type: _.Pseudo,
      name: "scope",
      data: null
    };
  function Ji(e, {
    adapter: t
  }, n) {
    let r = !!(n != null && n.every(a => {
      let i = t.isTag(a) && t.getParent(a);
      return a === vr || i && t.isTag(i);
    }));
    for (let a of e) {
      if (!(a.length > 0 && Kn(a[0]) && a[0].type !== _.Descendant)) if (r && !a.some(wr)) a.unshift(Qi);else continue;
      a.unshift(Xi);
    }
  }
  function Zn(e, t, n) {
    var r;
    e.forEach(ki), n = (r = t.context) !== null && r !== void 0 ? r : n;
    let a = Array.isArray(n),
      i = n && (Array.isArray(n) ? n : [n]);
    if (t.relativeSelector !== !1) Ji(e, t, i);else if (e.some(c => c.length > 0 && Kn(c[0]))) throw new Error("Relative selectors are not allowed when the `relativeSelector` option is disabled");
    let s = !1,
      l = e.map(c => {
        if (c.length >= 2) {
          let E = _slicedToArray(c, 2),
            f = E[0],
            m = E[1];
          f.type !== _.Pseudo || f.name !== "scope" || (a && m.type === _.Descendant ? c[1] = Yi : (m.type === _.Adjacent || m.type === _.Sibling) && (s = !0));
        }
        return qi(c, t, i);
      }).reduce(Vi, He.default.falseFunc);
    return l.shouldTestNextSiblings = s, l;
  }
  function qi(e, t, n) {
    var r;
    return e.reduce((a, i) => a === He.default.falseFunc ? He.default.falseFunc : Gi(a, i, t, n, Zn), (r = t.rootFunc) !== null && r !== void 0 ? r : He.default.trueFunc);
  }
  function Vi(e, t) {
    return t === He.default.falseFunc || e === He.default.trueFunc ? e : e === He.default.falseFunc || t === He.default.trueFunc ? t : function (n) {
      return e(n) || t(n);
    };
  }
  var Rr = (e, t) => e === t,
    ji = {
      adapter: ht,
      equals: Rr
    };
  function Or(e) {
    var t, n, r, a;
    let i = e != null ? e : ji;
    return (t = i.adapter) !== null && t !== void 0 || (i.adapter = ht), (n = i.equals) !== null && n !== void 0 || (i.equals = (a = (r = i.adapter) === null || r === void 0 ? void 0 : r.equals) !== null && a !== void 0 ? a : Rr), i;
  }
  function zn(e) {
    return function (t, n, r) {
      let a = Or(n);
      return e(t, a, r);
    };
  }
  var C1 = zn(Ki),
    p1 = zn(jn),
    $n = zn(Zn);
  function Lr(e) {
    return function (t, n, r) {
      let a = Or(r);
      typeof t != "function" && (t = jn(t, a, n));
      let i = e0(n, a.adapter, t.shouldTestNextSiblings);
      return e(t, i, a);
    };
  }
  function e0(e, t, n = !1) {
    return n && (e = Zi(e, t)), Array.isArray(e) ? t.removeSubsets(e) : t.getChildren(e);
  }
  function Zi(e, t) {
    let n = Array.isArray(e) ? e.slice(0) : [e],
      r = n.length;
    for (let a = 0; a < r; a++) {
      let i = kr(n[a], t);
      n.push(...i);
    }
    return n;
  }
  var f1 = Lr((e, t, n) => e === gr.default.falseFunc || !t || t.length === 0 ? [] : n.adapter.findAll(e, t)),
    x1 = Lr((e, t, n) => e === gr.default.falseFunc || !t || t.length === 0 ? null : n.adapter.findOne(e, t)),
    t0 = Xe(Je(), 1),
    zi = new Set(["first", "last", "eq", "gt", "nth", "lt", "even", "odd"]);
  function en(e) {
    return e.type !== "pseudo" ? !1 : zi.has(e.name) ? !0 : e.name === "not" && Array.isArray(e.data) ? e.data.some(t => t.some(en)) : !1;
  }
  function $i(e, t, n) {
    let r = t != null ? parseInt(t, 10) : NaN;
    switch (e) {
      case "first":
        return 1;
      case "nth":
      case "eq":
        return isFinite(r) ? r >= 0 ? r + 1 : 1 / 0 : 0;
      case "lt":
        return isFinite(r) ? r >= 0 ? Math.min(r, n) : 1 / 0 : 0;
      case "gt":
        return isFinite(r) ? 1 / 0 : 0;
      case "odd":
        return 2 * n;
      case "even":
        return 2 * n - 1;
      case "last":
      case "not":
        return 1 / 0;
    }
  }
  function es(e) {
    for (; e.parent;) e = e.parent;
    return e;
  }
  function n0(e) {
    let t = [],
      n = [];
    for (let r of e) r.some(en) ? t.push(r) : n.push(r);
    return [n, t];
  }
  var ts = {
      type: _.Universal,
      namespace: null
    },
    ns = {
      type: _.Pseudo,
      name: "scope",
      data: null
    };
  function Mr(e, t, n = {}) {
    return Hr([e], t, n);
  }
  function Hr(e, t, n = {}) {
    if (typeof t == "function") return e.some(t);
    let r = n0(Dt(t)),
      a = _slicedToArray(r, 2),
      i = a[0],
      s = a[1];
    return i.length > 0 && e.some($n(i, n)) || s.some(l => Wr(l, e, n).length > 0);
  }
  function rs(e, t, n, r) {
    let a = typeof n == "string" ? parseInt(n, 10) : NaN;
    switch (e) {
      case "first":
      case "lt":
        return t;
      case "last":
        return t.length > 0 ? [t[t.length - 1]] : t;
      case "nth":
      case "eq":
        return isFinite(a) && Math.abs(a) < t.length ? [a < 0 ? t[t.length + a] : t[a]] : [];
      case "gt":
        return isFinite(a) ? t.slice(a + 1) : [];
      case "even":
        return t.filter((i, s) => s % 2 === 0);
      case "odd":
        return t.filter((i, s) => s % 2 === 1);
      case "not":
        {
          let i = new Set(Ur(n, t, r));
          return t.filter(s => !i.has(s));
        }
    }
  }
  function Pr(e, t, n = {}) {
    return Ur(Dt(e), t, n);
  }
  function Ur(e, t, n) {
    if (t.length === 0) return [];
    let r = n0(e),
      a = _slicedToArray(r, 2),
      i = a[0],
      s = a[1],
      l;
    if (i.length) {
      let c = u0(t, i, n);
      if (s.length === 0) return c;
      c.length && (l = new Set(c));
    }
    for (let c = 0; c < s.length && (l == null ? void 0 : l.size) !== t.length; c++) {
      let E = s[c];
      if ((l ? t.filter(m => y(m) && !l.has(m)) : t).length === 0) break;
      let f = Wr(E, t, n);
      if (f.length) if (l) f.forEach(m => l.add(m));else {
        if (c === s.length - 1) return f;
        l = new Set(f);
      }
    }
    return typeof l < "u" ? l.size === t.length ? t : t.filter(c => l.has(c)) : [];
  }
  function Wr(e, t, n) {
    var r;
    if (e.some(Bt)) {
      let a = (r = n.root) !== null && r !== void 0 ? r : es(t[0]),
        i = Ye(L({}, n), {
          context: t,
          relativeSelector: !1
        });
      return e.push(ns), tn(a, e, i, !0, t.length);
    }
    return tn(t, e, n, !1, t.length);
  }
  function us(e, t, n = {}, r = 1 / 0) {
    if (typeof e == "function") return Gr(t, e);
    let a = n0(Dt(e)),
      i = _slicedToArray(a, 2),
      s = i[0],
      l = i[1],
      c = l.map(E => tn(t, E, n, !0, r));
    return s.length && c.push(r0(t, s, n, r)), c.length === 0 ? [] : c.length === 1 ? c[0] : it(c.reduce((E, f) => [...E, ...f]));
  }
  function tn(e, t, n, r, a) {
    let i = t.findIndex(en),
      s = t.slice(0, i),
      l = t[i],
      c = t.length - 1 === i ? a : 1 / 0,
      E = $i(l.name, l.data, c);
    if (E === 0) return [];
    let f = (s.length === 0 && !Array.isArray(e) ? Yt(e).filter(y) : s.length === 0 ? (Array.isArray(e) ? e : [e]).filter(y) : r || s.some(Bt) ? r0(e, [s], n, E) : u0(e, [s], n)).slice(0, E),
      m = rs(l.name, f, l.data, n);
    if (m.length === 0 || t.length === i + 1) return m;
    let D = t.slice(i + 1),
      T = D.some(Bt);
    if (T) {
      if (Bt(D[0])) {
        let b = D[0].type;
        (b === _.Sibling || b === _.Adjacent) && (m = e0(m, ht, !0)), D.unshift(ts);
      }
      n = Ye(L({}, n), {
        relativeSelector: !1,
        rootFunc: b => m.includes(b)
      });
    } else n.rootFunc && n.rootFunc !== t0.trueFunc && (n = Ye(L({}, n), {
      rootFunc: t0.trueFunc
    }));
    return D.some(en) ? tn(m, D, n, !1, a) : T ? r0(m, [D], n, a) : u0(m, [D], n);
  }
  function r0(e, t, n, r) {
    let a = $n(t, n, e);
    return Gr(e, a, r);
  }
  function Gr(e, t, n = 1 / 0) {
    let r = e0(e, ht, t.shouldTestNextSiblings);
    return vn(a => y(a) && t(a), r, !0, n);
  }
  function u0(e, t, n) {
    let r = (Array.isArray(e) ? e : [e]).filter(y);
    if (r.length === 0) return r;
    let a = $n(t, n);
    return a === t0.trueFunc ? r : r.filter(a);
  }
  var as = /^\s*(?:[+~]|:scope\b)/;
  function is(e) {
    if (!e) return this._make([]);
    if (typeof e != "string") {
      let t = be(e) ? e.toArray() : [e],
        n = this.toArray();
      return this._make(t.filter(r => n.some(a => Rn(a, r))));
    }
    return this._findBySelector(e, Number.POSITIVE_INFINITY);
  }
  function ss(e, t) {
    var n;
    let r = this.toArray(),
      a = as.test(e) ? r : this.children().toArray(),
      i = {
        context: r,
        root: (n = this._root) === null || n === void 0 ? void 0 : n[0],
        xmlMode: this.options.xmlMode,
        lowerCaseTags: this.options.lowerCaseTags,
        lowerCaseAttributeNames: this.options.lowerCaseAttributeNames,
        pseudos: this.options.pseudos,
        quirksMode: this.options.quirksMode
      };
    return this._make(us(e, a, i, t));
  }
  function a0(e) {
    return function (t, ...n) {
      return function (r) {
        var a;
        let i = e(t, this);
        return r && (i = o0(i, r, this.options.xmlMode, (a = this._root) === null || a === void 0 ? void 0 : a[0])), this._make(this.length > 1 && i.length > 1 ? n.reduce((s, l) => l(s), i) : i);
      };
    };
  }
  var gt = a0((e, t) => {
      let n = [];
      for (let r = 0; r < t.length; r++) {
        let a = e(t[r]);
        a.length > 0 && (n = n.concat(a));
      }
      return n;
    }),
    i0 = a0((e, t) => {
      let n = [];
      for (let r = 0; r < t.length; r++) {
        let a = e(t[r]);
        a !== null && n.push(a);
      }
      return n;
    });
  function s0(e, ...t) {
    let n = null,
      r = a0((a, i) => {
        let s = [];
        return P(i, l => {
          for (let c; (c = a(l)) && !(n != null && n(c, s.length)); l = c) s.push(c);
        }), s;
      })(e, ...t);
    return function (a, i) {
      n = typeof a == "string" ? l => Mr(l, a, this.options) : a ? Tt(a) : null;
      let s = r.call(this, i);
      return n = null, s;
    };
  }
  function At(e) {
    return e.length > 1 ? Array.from(new Set(e)) : e;
  }
  var os = i0(({
      parent: e
    }) => e && !Oe(e) ? e : null, At),
    As = gt(e => {
      let t = [];
      for (; e.parent && !Oe(e.parent);) t.push(e.parent), e = e.parent;
      return t;
    }, it, e => e.reverse()),
    cs = s0(({
      parent: e
    }) => e && !Oe(e) ? e : null, it, e => e.reverse());
  function ls(e) {
    var t;
    let n = [];
    if (!e) return this._make(n);
    let r = {
        xmlMode: this.options.xmlMode,
        root: (t = this._root) === null || t === void 0 ? void 0 : t[0]
      },
      a = typeof e == "string" ? i => Mr(i, e, r) : Tt(e);
    return P(this, i => {
      for (i && !Oe(i) && !y(i) && (i = i.parent); i && y(i);) {
        if (a(i, 0)) {
          n.includes(i) || n.push(i);
          break;
        }
        i = i.parent;
      }
    }), this._make(n);
  }
  var hs = i0(e => Sn(e)),
    ds = gt(e => {
      let t = [];
      for (; e.next;) e = e.next, y(e) && t.push(e);
      return t;
    }, At),
    Es = s0(e => Sn(e), At),
    Cs = i0(e => In(e)),
    ps = gt(e => {
      let t = [];
      for (; e.prev;) e = e.prev, y(e) && t.push(e);
      return t;
    }, At),
    fs = s0(e => In(e), At),
    xs = gt(e => q0(e).filter(t => y(t) && t !== e), it),
    Bs = gt(e => Yt(e).filter(y), At);
  function ms() {
    let e = this.toArray().reduce((t, n) => Q(n) ? t.concat(n.children) : t, []);
    return this._make(e);
  }
  function Ds(e) {
    let t = 0,
      n = this.length;
    for (; t < n && e.call(this[t], t, this[t]) !== !1;) ++t;
    return this;
  }
  function gs(e) {
    let t = [];
    for (let n = 0; n < this.length; n++) {
      let r = this[n],
        a = e.call(r, n, r);
      a != null && (t = t.concat(a));
    }
    return this._make(t);
  }
  function Tt(e) {
    return typeof e == "function" ? (t, n) => e.call(t, n, t) : be(e) ? t => Array.prototype.includes.call(e, t) : function (t) {
      return e === t;
    };
  }
  function Ts(e) {
    var t;
    return this._make(o0(this.toArray(), e, this.options.xmlMode, (t = this._root) === null || t === void 0 ? void 0 : t[0]));
  }
  function o0(e, t, n, r) {
    return typeof t == "string" ? Pr(t, e, {
      xmlMode: n,
      root: r
    }) : e.filter(Tt(t));
  }
  function Fs(e) {
    let t = this.toArray();
    return typeof e == "string" ? Hr(t.filter(y), e, this.options) : e ? t.some(Tt(e)) : !1;
  }
  function _s(e) {
    let t = this.toArray();
    if (typeof e == "string") {
      let n = new Set(Pr(e, t, this.options));
      t = t.filter(r => !n.has(r));
    } else {
      let n = Tt(e);
      t = t.filter((r, a) => !n(r, a));
    }
    return this._make(t);
  }
  function ys(e) {
    return this.filter(typeof e == "string" ? ":has(".concat(e, ")") : (t, n) => this._make(n).find(e).length > 0);
  }
  function bs() {
    return this.length > 1 ? this._make(this[0]) : this;
  }
  function Ss() {
    return this.length > 0 ? this._make(this[this.length - 1]) : this;
  }
  function Is(e) {
    var t;
    return e = +e, e === 0 && this.length <= 1 ? this : (e < 0 && (e = this.length + e), this._make((t = this[e]) !== null && t !== void 0 ? t : []));
  }
  function vs(e) {
    return e == null ? this.toArray() : this[e < 0 ? this.length + e : e];
  }
  function Ns() {
    return Array.prototype.slice.call(this);
  }
  function ks(e) {
    let t, n;
    return e == null ? (t = this.parent().children(), n = this[0]) : typeof e == "string" ? (t = this._make(e), n = this[0]) : (t = this, n = be(e) ? e[0] : e), Array.prototype.indexOf.call(t, n);
  }
  function ws(e, t) {
    return this._make(Array.prototype.slice.call(this, e, t));
  }
  function Rs() {
    var e;
    return (e = this.prevObject) !== null && e !== void 0 ? e : this._make([]);
  }
  function Os(e, t) {
    let n = this._make(e, t),
      r = it([...this.get(), ...n.get()]);
    return this._make(r);
  }
  function Ls(e) {
    return this.prevObject ? this.add(e ? this.prevObject.filter(e) : this.prevObject) : this;
  }
  var Kr = {};
  le(Kr, {
    _makeDomArray: () => Hs,
    after: () => Js,
    append: () => Ws,
    appendTo: () => Ps,
    before: () => Vs,
    clone: () => ro,
    empty: () => $s,
    html: () => eo,
    insertAfter: () => qs,
    insertBefore: () => js,
    prepend: () => Gs,
    prependTo: () => Us,
    remove: () => Zs,
    replaceWith: () => zs,
    text: () => no,
    toString: () => to,
    unwrap: () => Ys,
    wrap: () => Ks,
    wrapAll: () => Xs,
    wrapInner: () => Qs
  });
  function Ms(e) {
    return function (t, n, r, a) {
      if (typeof Buffer < "u" && Buffer.isBuffer(t) && (t = t.toString()), typeof t == "string") return e(t, n, r, a);
      let i = t;
      if (!Array.isArray(i) && Oe(i)) return i;
      let s = new Ve([]);
      return et(i, s), s;
    };
  }
  function et(e, t) {
    let n = Array.isArray(e) ? e : [e];
    t ? t.children = n : t = null;
    for (let r = 0; r < n.length; r++) {
      let a = n[r];
      a.parent && a.parent.children !== n && Ze(a), t ? (a.prev = n[r - 1] || null, a.next = n[r + 1] || null) : a.prev = a.next = null, a.parent = t;
    }
    return t;
  }
  function Hs(e, t) {
    if (e == null) return [];
    if (typeof e == "string") return this._parse(e, this.options, !1, null).children.slice(0);
    if ("length" in e) {
      if (e.length === 1) return this._makeDomArray(e[0], t);
      let n = [];
      for (let r = 0; r < e.length; r++) {
        let a = e[r];
        if (typeof a == "object") {
          if (a == null) continue;
          if (!("length" in a)) {
            n.push(t ? Et(a, !0) : a);
            continue;
          }
        }
        n.push(...this._makeDomArray(a, t));
      }
      return n;
    }
    return [t ? Et(e, !0) : e];
  }
  function Qr(e) {
    return function (...t) {
      let n = this.length - 1;
      return P(this, (r, a) => {
        if (!Q(r)) return;
        let i = typeof t[0] == "function" ? t[0].call(r, a, this._render(r.children)) : t,
          s = this._makeDomArray(i, a < n);
        e(s, r.children, r);
      });
    };
  }
  function Pe(e, t, n, r, a) {
    var i, s;
    let l = [t, n, ...r],
      c = t === 0 ? null : e[t - 1],
      E = t + n >= e.length ? null : e[t + n];
    for (let f = 0; f < r.length; ++f) {
      let m = r[f],
        D = m.parent;
      if (D) {
        let T = D.children.indexOf(m);
        T !== -1 && (D.children.splice(T, 1), a === D && t > T && l[0]--);
      }
      m.parent = a, m.prev && (m.prev.next = (i = m.next) !== null && i !== void 0 ? i : null), m.next && (m.next.prev = (s = m.prev) !== null && s !== void 0 ? s : null), m.prev = f === 0 ? c : r[f - 1], m.next = f === r.length - 1 ? E : r[f + 1];
    }
    return c && (c.next = r[0]), E && (E.prev = r[r.length - 1]), e.splice(...l);
  }
  function Ps(e) {
    return (be(e) ? e : this._make(e)).append(this), this;
  }
  function Us(e) {
    return (be(e) ? e : this._make(e)).prepend(this), this;
  }
  var Ws = Qr((e, t, n) => {
      Pe(t, t.length, 0, e, n);
    }),
    Gs = Qr((e, t, n) => {
      Pe(t, 0, 0, e, n);
    });
  function Yr(e) {
    return function (t) {
      let n = this.length - 1,
        r = this.parents().last();
      for (let a = 0; a < this.length; a++) {
        let i = this[a],
          s = typeof t == "function" ? t.call(i, a, i) : typeof t == "string" && !On(t) ? r.find(t).clone() : t,
          l = this._makeDomArray(s, a < n),
          c = _slicedToArray(l, 1),
          E = c[0];
        if (!E || !Q(E)) continue;
        let f = E,
          m = 0;
        for (; m < f.children.length;) {
          let D = f.children[m];
          y(D) ? (f = D, m = 0) : m++;
        }
        e(i, f, [E]);
      }
      return this;
    };
  }
  var Ks = Yr((e, t, n) => {
      let r = e.parent;
      if (!r) return;
      let a = r.children,
        i = a.indexOf(e);
      et([e], t), Pe(a, i, 0, n, r);
    }),
    Qs = Yr((e, t, n) => {
      Q(e) && (et(e.children, t), et(n, e));
    });
  function Ys(e) {
    return this.parent(e).not("body").each((t, n) => {
      this._make(n).replaceWith(n.children);
    }), this;
  }
  function Xs(e) {
    let t = this[0];
    if (t) {
      let n = this._make(typeof e == "function" ? e.call(t, 0, t) : e).insertBefore(t),
        r;
      for (let i = 0; i < n.length; i++) n[i].type === qe.Tag && (r = n[i]);
      let a = 0;
      for (; r && a < r.children.length;) {
        let i = r.children[a];
        i.type === qe.Tag ? (r = i, a = 0) : a++;
      }
      r && this._make(r).append(this);
    }
    return this;
  }
  function Js(...e) {
    let t = this.length - 1;
    return P(this, (n, r) => {
      if (!Q(n) || !n.parent) return;
      let a = n.parent.children,
        i = a.indexOf(n);
      if (i === -1) return;
      let s = typeof e[0] == "function" ? e[0].call(n, r, this._render(n.children)) : e,
        l = this._makeDomArray(s, r < t);
      Pe(a, i + 1, 0, l, n.parent);
    });
  }
  function qs(e) {
    typeof e == "string" && (e = this._make(e)), this.remove();
    let t = [];
    for (let n of this._makeDomArray(e)) {
      let r = this.clone().toArray(),
        a = n.parent;
      if (!a) continue;
      let i = a.children,
        s = i.indexOf(n);
      s !== -1 && (Pe(i, s + 1, 0, r, a), t.push(...r));
    }
    return this._make(t);
  }
  function Vs(...e) {
    let t = this.length - 1;
    return P(this, (n, r) => {
      if (!Q(n) || !n.parent) return;
      let a = n.parent.children,
        i = a.indexOf(n);
      if (i === -1) return;
      let s = typeof e[0] == "function" ? e[0].call(n, r, this._render(n.children)) : e,
        l = this._makeDomArray(s, r < t);
      Pe(a, i, 0, l, n.parent);
    });
  }
  function js(e) {
    let t = this._make(e);
    this.remove();
    let n = [];
    return P(t, r => {
      let a = this.clone().toArray(),
        i = r.parent;
      if (!i) return;
      let s = i.children,
        l = s.indexOf(r);
      l !== -1 && (Pe(s, l, 0, a, i), n.push(...a));
    }), this._make(n);
  }
  function Zs(e) {
    let t = e ? this.filter(e) : this;
    return P(t, n => {
      Ze(n), n.prev = n.next = n.parent = null;
    }), this;
  }
  function zs(e) {
    return P(this, (t, n) => {
      let r = t.parent;
      if (!r) return;
      let a = r.children,
        i = typeof e == "function" ? e.call(t, n, t) : e,
        s = this._makeDomArray(i);
      et(s, null);
      let l = a.indexOf(t);
      Pe(a, l, 1, s, r), s.includes(t) || (t.parent = t.prev = t.next = null);
    });
  }
  function $s() {
    return P(this, e => {
      if (Q(e)) {
        for (let t of e.children) t.next = t.prev = t.parent = null;
        e.children.length = 0;
      }
    });
  }
  function eo(e) {
    if (e === void 0) {
      let t = this[0];
      return !t || !Q(t) ? null : this._render(t.children);
    }
    return P(this, t => {
      if (!Q(t)) return;
      for (let r of t.children) r.next = r.prev = r.parent = null;
      let n = be(e) ? e.toArray() : this._parse("".concat(e), this.options, !1, t).children;
      et(n, t);
    });
  }
  function to() {
    return this._render(this);
  }
  function no(e) {
    return e === void 0 ? pt(this) : typeof e == "function" ? P(this, (t, n) => this._make(t).text(e.call(t, n, pt([t])))) : P(this, t => {
      if (!Q(t)) return;
      for (let r of t.children) r.next = r.prev = r.parent = null;
      let n = new dt("".concat(e));
      et(n, t);
    });
  }
  function ro() {
    let e = Array.prototype.map.call(this.get(), n => Et(n, !0)),
      t = new Ve(e);
    for (let n of e) n.parent = t;
    return this._make(e);
  }
  var Xr = {};
  le(Xr, {
    css: () => uo
  });
  function uo(e, t) {
    if (e != null && t != null || typeof e == "object" && !Array.isArray(e)) return P(this, (n, r) => {
      y(n) && Jr(n, e, t, r);
    });
    if (this.length !== 0) return qr(this[0], e);
  }
  function Jr(e, t, n, r) {
    if (typeof t == "string") {
      let a = qr(e),
        i = typeof n == "function" ? n.call(e, r, a[t]) : n;
      i === "" ? delete a[t] : i != null && (a[t] = i), e.attribs.style = ao(a);
    } else if (typeof t == "object") {
      let a = Object.keys(t);
      for (let i = 0; i < a.length; i++) {
        let s = a[i];
        Jr(e, s, t[s], i);
      }
    }
  }
  function qr(e, t) {
    if (!e || !y(e)) return;
    let n = io(e.attribs.style);
    if (typeof t == "string") return n[t];
    if (Array.isArray(t)) {
      let r = {};
      for (let a of t) n[a] != null && (r[a] = n[a]);
      return r;
    }
    return n;
  }
  function ao(e) {
    return Object.keys(e).reduce((t, n) => "".concat(t).concat(t ? " " : "").concat(n, ": ").concat(e[n], ";"), "");
  }
  function io(e) {
    if (e = (e || "").trim(), !e) return {};
    let t = {},
      n;
    for (let r of e.split(";")) {
      let a = r.indexOf(":");
      if (a < 1 || a === r.length - 1) {
        let i = r.trimEnd();
        i.length > 0 && n !== void 0 && (t[n] += ";".concat(i));
      } else n = r.slice(0, a).trim(), t[n] = r.slice(a + 1).trim();
    }
    return t;
  }
  var Vr = {};
  le(Vr, {
    serialize: () => oo,
    serializeArray: () => Ao
  });
  var jr = "input,select,textarea,keygen",
    so = /%20/g,
    Zr = /\r?\n/g;
  function oo() {
    return this.serializeArray().map(e => "".concat(encodeURIComponent(e.name), "=").concat(encodeURIComponent(e.value))).join("&").replace(so, "+");
  }
  function Ao() {
    return this.map((e, t) => {
      let n = this._make(t);
      return y(t) && t.name === "form" ? n.find(jr).toArray() : n.filter(jr).toArray();
    }).filter('[name!=""]:enabled:not(:submit, :button, :image, :reset, :file):matches([checked], :not(:checkbox, :radio))').map((e, t) => {
      var n;
      let r = this._make(t),
        a = r.attr("name"),
        i = (n = r.val()) !== null && n !== void 0 ? n : "";
      return Array.isArray(i) ? i.map(s => ({
        name: a,
        value: s.replace(Zr, "\r\n")
      })) : {
        name: a,
        value: i.replace(Zr, "\r\n")
      };
    }).toArray();
  }
  var zr = {};
  le(zr, {
    extract: () => lo
  });
  function co(e) {
    var t;
    return typeof e == "string" ? {
      selector: e,
      value: "textContent"
    } : {
      selector: e.selector,
      value: (t = e.value) !== null && t !== void 0 ? t : "textContent"
    };
  }
  function lo(e) {
    let t = {};
    for (let n in e) {
      let r = e[n],
        a = Array.isArray(r),
        i = co(a ? r[0] : r),
        s = i.selector,
        l = i.value,
        c = typeof l == "function" ? l : typeof l == "string" ? E => this._make(E).prop(l) : E => this._make(E).extract(l);
      if (a) t[n] = this._findBySelector(s, Number.POSITIVE_INFINITY).map((E, f) => c(f, n, t)).get();else {
        let E = this._findBySelector(s, 1);
        t[n] = E.length > 0 ? c(E[0], n, t) : void 0;
      }
    }
    return t;
  }
  var Ft = class {
    constructor(e, t, n) {
      if (this.length = 0, this.options = n, this._root = t, e) {
        for (let r = 0; r < e.length; r++) this[r] = e[r];
        this.length = e.length;
      }
    }
  };
  Ft.prototype.cheerio = "[cheerio object]", Ft.prototype.splice = Array.prototype.splice, Ft.prototype[Symbol.iterator] = Array.prototype[Symbol.iterator], Object.assign(Ft.prototype, nr, xr, Kr, Xr, Vr, zr);
  function ho(e, t) {
    return function n(r, a, i = !0) {
      if (r == null) throw new Error("cheerio.load() expects a string");
      let s = wn(a),
        l = e(r, s, i, null);
      class c extends Ft {
        _make(m, D) {
          let T = E(m, D);
          return T.prevObject = this, T;
        }
        _parse(m, D, T, b) {
          return e(m, D, T, b);
        }
        _render(m) {
          return t(m, this.options);
        }
      }
      function E(f, m, D = l, T) {
        if (f && be(f)) return f;
        let b = wn(T, s),
          Y = typeof D == "string" ? [e(D, b, !1, null)] : "length" in D ? D : [D],
          re = be(Y) ? Y : new c(Y, null, b);
        if (re._root = re, !f) return new c(void 0, re, b);
        let k = typeof f == "string" && On(f) ? e(f, b, !1, null).children : Eo(f) ? [f] : Array.isArray(f) ? f : void 0,
          g = new c(k, re, b);
        if (k) return g;
        if (typeof f != "string") throw new TypeError("Unexpected type of selector");
        let I = f,
          te = m ? typeof m == "string" ? On(m) ? new c([e(m, b, !1, null)], re, b) : (I = "".concat(m, " ").concat(I), re) : be(m) ? m : new c(Array.isArray(m) ? m : [m], re, b) : re;
        return te ? te.find(I) : g;
      }
      return Object.assign(E, F0, {
        load: n,
        _root: l,
        _options: s,
        fn: c.prototype,
        prototype: c.prototype
      }), E;
    };
  }
  function Eo(e) {
    return !!e.name || e.type === qe.Root || e.type === qe.Text || e.type === qe.Comment;
  }
  var Co = new Set([65534, 65535, 131070, 131071, 196606, 196607, 262142, 262143, 327678, 327679, 393214, 393215, 458750, 458751, 524286, 524287, 589822, 589823, 655358, 655359, 720894, 720895, 786430, 786431, 851966, 851967, 917502, 917503, 983038, 983039, 1048574, 1048575, 1114110, 1114111]),
    H = "\uFFFD",
    o;
  (function (e) {
    e[e.EOF = -1] = "EOF", e[e.NULL = 0] = "NULL", e[e.TABULATION = 9] = "TABULATION", e[e.CARRIAGE_RETURN = 13] = "CARRIAGE_RETURN", e[e.LINE_FEED = 10] = "LINE_FEED", e[e.FORM_FEED = 12] = "FORM_FEED", e[e.SPACE = 32] = "SPACE", e[e.EXCLAMATION_MARK = 33] = "EXCLAMATION_MARK", e[e.QUOTATION_MARK = 34] = "QUOTATION_MARK", e[e.AMPERSAND = 38] = "AMPERSAND", e[e.APOSTROPHE = 39] = "APOSTROPHE", e[e.HYPHEN_MINUS = 45] = "HYPHEN_MINUS", e[e.SOLIDUS = 47] = "SOLIDUS", e[e.DIGIT_0 = 48] = "DIGIT_0", e[e.DIGIT_9 = 57] = "DIGIT_9", e[e.SEMICOLON = 59] = "SEMICOLON", e[e.LESS_THAN_SIGN = 60] = "LESS_THAN_SIGN", e[e.EQUALS_SIGN = 61] = "EQUALS_SIGN", e[e.GREATER_THAN_SIGN = 62] = "GREATER_THAN_SIGN", e[e.QUESTION_MARK = 63] = "QUESTION_MARK", e[e.LATIN_CAPITAL_A = 65] = "LATIN_CAPITAL_A", e[e.LATIN_CAPITAL_Z = 90] = "LATIN_CAPITAL_Z", e[e.RIGHT_SQUARE_BRACKET = 93] = "RIGHT_SQUARE_BRACKET", e[e.GRAVE_ACCENT = 96] = "GRAVE_ACCENT", e[e.LATIN_SMALL_A = 97] = "LATIN_SMALL_A", e[e.LATIN_SMALL_Z = 122] = "LATIN_SMALL_Z";
  })(o || (o = {}));
  var oe = {
    DASH_DASH: "--",
    CDATA_START: "[CDATA[",
    DOCTYPE: "doctype",
    SCRIPT: "script",
    PUBLIC: "public",
    SYSTEM: "system"
  };
  function $r(e) {
    return e >= 55296 && e <= 57343;
  }
  function po(e) {
    return e >= 56320 && e <= 57343;
  }
  function fo(e, t) {
    return (e - 55296) * 1024 + 9216 + t;
  }
  function eu(e) {
    return e !== 32 && e !== 10 && e !== 13 && e !== 9 && e !== 12 && e >= 1 && e <= 31 || e >= 127 && e <= 159;
  }
  function tu(e) {
    return e >= 64976 && e <= 65007 || Co.has(e);
  }
  var C;
  (function (e) {
    e.controlCharacterInInputStream = "control-character-in-input-stream", e.noncharacterInInputStream = "noncharacter-in-input-stream", e.surrogateInInputStream = "surrogate-in-input-stream", e.nonVoidHtmlElementStartTagWithTrailingSolidus = "non-void-html-element-start-tag-with-trailing-solidus", e.endTagWithAttributes = "end-tag-with-attributes", e.endTagWithTrailingSolidus = "end-tag-with-trailing-solidus", e.unexpectedSolidusInTag = "unexpected-solidus-in-tag", e.unexpectedNullCharacter = "unexpected-null-character", e.unexpectedQuestionMarkInsteadOfTagName = "unexpected-question-mark-instead-of-tag-name", e.invalidFirstCharacterOfTagName = "invalid-first-character-of-tag-name", e.unexpectedEqualsSignBeforeAttributeName = "unexpected-equals-sign-before-attribute-name", e.missingEndTagName = "missing-end-tag-name", e.unexpectedCharacterInAttributeName = "unexpected-character-in-attribute-name", e.unknownNamedCharacterReference = "unknown-named-character-reference", e.missingSemicolonAfterCharacterReference = "missing-semicolon-after-character-reference", e.unexpectedCharacterAfterDoctypeSystemIdentifier = "unexpected-character-after-doctype-system-identifier", e.unexpectedCharacterInUnquotedAttributeValue = "unexpected-character-in-unquoted-attribute-value", e.eofBeforeTagName = "eof-before-tag-name", e.eofInTag = "eof-in-tag", e.missingAttributeValue = "missing-attribute-value", e.missingWhitespaceBetweenAttributes = "missing-whitespace-between-attributes", e.missingWhitespaceAfterDoctypePublicKeyword = "missing-whitespace-after-doctype-public-keyword", e.missingWhitespaceBetweenDoctypePublicAndSystemIdentifiers = "missing-whitespace-between-doctype-public-and-system-identifiers", e.missingWhitespaceAfterDoctypeSystemKeyword = "missing-whitespace-after-doctype-system-keyword", e.missingQuoteBeforeDoctypePublicIdentifier = "missing-quote-before-doctype-public-identifier", e.missingQuoteBeforeDoctypeSystemIdentifier = "missing-quote-before-doctype-system-identifier", e.missingDoctypePublicIdentifier = "missing-doctype-public-identifier", e.missingDoctypeSystemIdentifier = "missing-doctype-system-identifier", e.abruptDoctypePublicIdentifier = "abrupt-doctype-public-identifier", e.abruptDoctypeSystemIdentifier = "abrupt-doctype-system-identifier", e.cdataInHtmlContent = "cdata-in-html-content", e.incorrectlyOpenedComment = "incorrectly-opened-comment", e.eofInScriptHtmlCommentLikeText = "eof-in-script-html-comment-like-text", e.eofInDoctype = "eof-in-doctype", e.nestedComment = "nested-comment", e.abruptClosingOfEmptyComment = "abrupt-closing-of-empty-comment", e.eofInComment = "eof-in-comment", e.incorrectlyClosedComment = "incorrectly-closed-comment", e.eofInCdata = "eof-in-cdata", e.absenceOfDigitsInNumericCharacterReference = "absence-of-digits-in-numeric-character-reference", e.nullCharacterReference = "null-character-reference", e.surrogateCharacterReference = "surrogate-character-reference", e.characterReferenceOutsideUnicodeRange = "character-reference-outside-unicode-range", e.controlCharacterReference = "control-character-reference", e.noncharacterCharacterReference = "noncharacter-character-reference", e.missingWhitespaceBeforeDoctypeName = "missing-whitespace-before-doctype-name", e.missingDoctypeName = "missing-doctype-name", e.invalidCharacterSequenceAfterDoctypeName = "invalid-character-sequence-after-doctype-name", e.duplicateAttribute = "duplicate-attribute", e.nonConformingDoctype = "non-conforming-doctype", e.missingDoctype = "missing-doctype", e.misplacedDoctype = "misplaced-doctype", e.endTagWithoutMatchingOpenElement = "end-tag-without-matching-open-element", e.closingOfElementWithOpenChildElements = "closing-of-element-with-open-child-elements", e.disallowedContentInNoscriptInHead = "disallowed-content-in-noscript-in-head", e.openElementsLeftAfterEof = "open-elements-left-after-eof", e.abandonedHeadElementChild = "abandoned-head-element-child", e.misplacedStartTagForHeadElement = "misplaced-start-tag-for-head-element", e.nestedNoscriptInHead = "nested-noscript-in-head", e.eofInElementThatCanContainOnlyText = "eof-in-element-that-can-contain-only-text";
  })(C || (C = {}));
  var xo = 65536,
    Bo = class {
      constructor(e) {
        this.handler = e, this.html = "", this.pos = -1, this.lastGapPos = -2, this.gapStack = [], this.skipNextNewLine = !1, this.lastChunkWritten = !1, this.endOfChunkHit = !1, this.bufferWaterline = xo, this.isEol = !1, this.lineStartPos = 0, this.droppedBufferSize = 0, this.line = 1, this.lastErrOffset = -1;
      }
      get col() {
        return this.pos - this.lineStartPos + +(this.lastGapPos !== this.pos);
      }
      get offset() {
        return this.droppedBufferSize + this.pos;
      }
      getError(e, t) {
        let n = this.line,
          r = this.col,
          a = this.offset,
          i = r + t,
          s = a + t;
        return {
          code: e,
          startLine: n,
          endLine: n,
          startCol: i,
          endCol: i,
          startOffset: s,
          endOffset: s
        };
      }
      _err(e) {
        this.handler.onParseError && this.lastErrOffset !== this.offset && (this.lastErrOffset = this.offset, this.handler.onParseError(this.getError(e, 0)));
      }
      _addGap() {
        this.gapStack.push(this.lastGapPos), this.lastGapPos = this.pos;
      }
      _processSurrogate(e) {
        if (this.pos !== this.html.length - 1) {
          let t = this.html.charCodeAt(this.pos + 1);
          if (po(t)) return this.pos++, this._addGap(), fo(e, t);
        } else if (!this.lastChunkWritten) return this.endOfChunkHit = !0, o.EOF;
        return this._err(C.surrogateInInputStream), e;
      }
      willDropParsedChunk() {
        return this.pos > this.bufferWaterline;
      }
      dropParsedChunk() {
        this.willDropParsedChunk() && (this.html = this.html.substring(this.pos), this.lineStartPos -= this.pos, this.droppedBufferSize += this.pos, this.pos = 0, this.lastGapPos = -2, this.gapStack.length = 0);
      }
      write(e, t) {
        this.html.length > 0 ? this.html += e : this.html = e, this.endOfChunkHit = !1, this.lastChunkWritten = t;
      }
      insertHtmlAtCurrentPos(e) {
        this.html = this.html.substring(0, this.pos + 1) + e + this.html.substring(this.pos + 1), this.endOfChunkHit = !1;
      }
      startsWith(e, t) {
        if (this.pos + e.length > this.html.length) return this.endOfChunkHit = !this.lastChunkWritten, !1;
        if (t) return this.html.startsWith(e, this.pos);
        for (let n = 0; n < e.length; n++) if ((this.html.charCodeAt(this.pos + n) | 32) !== e.charCodeAt(n)) return !1;
        return !0;
      }
      peek(e) {
        let t = this.pos + e;
        if (t >= this.html.length) return this.endOfChunkHit = !this.lastChunkWritten, o.EOF;
        let n = this.html.charCodeAt(t);
        return n === o.CARRIAGE_RETURN ? o.LINE_FEED : n;
      }
      advance() {
        if (this.pos++, this.isEol && (this.isEol = !1, this.line++, this.lineStartPos = this.pos), this.pos >= this.html.length) return this.endOfChunkHit = !this.lastChunkWritten, o.EOF;
        let e = this.html.charCodeAt(this.pos);
        return e === o.CARRIAGE_RETURN ? (this.isEol = !0, this.skipNextNewLine = !0, o.LINE_FEED) : e === o.LINE_FEED && (this.isEol = !0, this.skipNextNewLine) ? (this.line--, this.skipNextNewLine = !1, this._addGap(), this.advance()) : (this.skipNextNewLine = !1, $r(e) && (e = this._processSurrogate(e)), this.handler.onParseError === null || e > 31 && e < 127 || e === o.LINE_FEED || e === o.CARRIAGE_RETURN || e > 159 && e < 64976 || this._checkForProblematicCharacters(e), e);
      }
      _checkForProblematicCharacters(e) {
        eu(e) ? this._err(C.controlCharacterInInputStream) : tu(e) && this._err(C.noncharacterInInputStream);
      }
      retreat(e) {
        for (this.pos -= e; this.pos < this.lastGapPos;) this.lastGapPos = this.gapStack.pop(), this.pos--;
        this.isEol = !1;
      }
    },
    mo = {};
  le(mo, {
    TokenType: () => v,
    getTokenAttr: () => A0
  });
  var v;
  (function (e) {
    e[e.CHARACTER = 0] = "CHARACTER", e[e.NULL_CHARACTER = 1] = "NULL_CHARACTER", e[e.WHITESPACE_CHARACTER = 2] = "WHITESPACE_CHARACTER", e[e.START_TAG = 3] = "START_TAG", e[e.END_TAG = 4] = "END_TAG", e[e.COMMENT = 5] = "COMMENT", e[e.DOCTYPE = 6] = "DOCTYPE", e[e.EOF = 7] = "EOF", e[e.HIBERNATION = 8] = "HIBERNATION";
  })(v || (v = {}));
  function A0(e, t) {
    for (let n = e.attrs.length - 1; n >= 0; n--) if (e.attrs[n].name === t) return e.attrs[n].value;
    return null;
  }
  var Do = new Uint16Array('\u1D41<\xD5\u0131\u028A\u049D\u057B\u05D0\u0675\u06DE\u07A2\u07D6\u080F\u0A4A\u0A91\u0DA1\u0E6D\u0F09\u0F26\u10CA\u1228\u12E1\u1415\u149D\u14C3\u14DF\u1525\0\0\0\0\0\0\u156B\u16CD\u198D\u1C12\u1DDD\u1F7E\u2060\u21B0\u228D\u23C0\u23FB\u2442\u2824\u2912\u2D08\u2E48\u2FCE\u3016\u32BA\u3639\u37AC\u38FE\u3A28\u3A71\u3AE0\u3B2E\u0800EMabcfglmnoprstu\\bfms\x7F\x84\x8B\x90\x95\x98\xA6\xB3\xB9\xC8\xCFlig\u803B\xC6\u40C6P\u803B&\u4026cute\u803B\xC1\u40C1reve;\u4102\u0100iyx}rc\u803B\xC2\u40C2;\u4410r;\uC000\u{1D504}rave\u803B\xC0\u40C0pha;\u4391acr;\u4100d;\u6A53\u0100gp\x9D\xA1on;\u4104f;\uC000\u{1D538}plyFunction;\u6061ing\u803B\xC5\u40C5\u0100cs\xBE\xC3r;\uC000\u{1D49C}ign;\u6254ilde\u803B\xC3\u40C3ml\u803B\xC4\u40C4\u0400aceforsu\xE5\xFB\xFE\u0117\u011C\u0122\u0127\u012A\u0100cr\xEA\xF2kslash;\u6216\u0176\xF6\xF8;\u6AE7ed;\u6306y;\u4411\u0180crt\u0105\u010B\u0114ause;\u6235noullis;\u612Ca;\u4392r;\uC000\u{1D505}pf;\uC000\u{1D539}eve;\u42D8c\xF2\u0113mpeq;\u624E\u0700HOacdefhilorsu\u014D\u0151\u0156\u0180\u019E\u01A2\u01B5\u01B7\u01BA\u01DC\u0215\u0273\u0278\u027Ecy;\u4427PY\u803B\xA9\u40A9\u0180cpy\u015D\u0162\u017Aute;\u4106\u0100;i\u0167\u0168\u62D2talDifferentialD;\u6145leys;\u612D\u0200aeio\u0189\u018E\u0194\u0198ron;\u410Cdil\u803B\xC7\u40C7rc;\u4108nint;\u6230ot;\u410A\u0100dn\u01A7\u01ADilla;\u40B8terDot;\u40B7\xF2\u017Fi;\u43A7rcle\u0200DMPT\u01C7\u01CB\u01D1\u01D6ot;\u6299inus;\u6296lus;\u6295imes;\u6297o\u0100cs\u01E2\u01F8kwiseContourIntegral;\u6232eCurly\u0100DQ\u0203\u020FoubleQuote;\u601Duote;\u6019\u0200lnpu\u021E\u0228\u0247\u0255on\u0100;e\u0225\u0226\u6237;\u6A74\u0180git\u022F\u0236\u023Aruent;\u6261nt;\u622FourIntegral;\u622E\u0100fr\u024C\u024E;\u6102oduct;\u6210nterClockwiseContourIntegral;\u6233oss;\u6A2Fcr;\uC000\u{1D49E}p\u0100;C\u0284\u0285\u62D3ap;\u624D\u0580DJSZacefios\u02A0\u02AC\u02B0\u02B4\u02B8\u02CB\u02D7\u02E1\u02E6\u0333\u048D\u0100;o\u0179\u02A5trahd;\u6911cy;\u4402cy;\u4405cy;\u440F\u0180grs\u02BF\u02C4\u02C7ger;\u6021r;\u61A1hv;\u6AE4\u0100ay\u02D0\u02D5ron;\u410E;\u4414l\u0100;t\u02DD\u02DE\u6207a;\u4394r;\uC000\u{1D507}\u0100af\u02EB\u0327\u0100cm\u02F0\u0322ritical\u0200ADGT\u0300\u0306\u0316\u031Ccute;\u40B4o\u0174\u030B\u030D;\u42D9bleAcute;\u42DDrave;\u4060ilde;\u42DCond;\u62C4ferentialD;\u6146\u0470\u033D\0\0\0\u0342\u0354\0\u0405f;\uC000\u{1D53B}\u0180;DE\u0348\u0349\u034D\u40A8ot;\u60DCqual;\u6250ble\u0300CDLRUV\u0363\u0372\u0382\u03CF\u03E2\u03F8ontourIntegra\xEC\u0239o\u0274\u0379\0\0\u037B\xBB\u0349nArrow;\u61D3\u0100eo\u0387\u03A4ft\u0180ART\u0390\u0396\u03A1rrow;\u61D0ightArrow;\u61D4e\xE5\u02CAng\u0100LR\u03AB\u03C4eft\u0100AR\u03B3\u03B9rrow;\u67F8ightArrow;\u67FAightArrow;\u67F9ight\u0100AT\u03D8\u03DErrow;\u61D2ee;\u62A8p\u0241\u03E9\0\0\u03EFrrow;\u61D1ownArrow;\u61D5erticalBar;\u6225n\u0300ABLRTa\u0412\u042A\u0430\u045E\u047F\u037Crrow\u0180;BU\u041D\u041E\u0422\u6193ar;\u6913pArrow;\u61F5reve;\u4311eft\u02D2\u043A\0\u0446\0\u0450ightVector;\u6950eeVector;\u695Eector\u0100;B\u0459\u045A\u61BDar;\u6956ight\u01D4\u0467\0\u0471eeVector;\u695Fector\u0100;B\u047A\u047B\u61C1ar;\u6957ee\u0100;A\u0486\u0487\u62A4rrow;\u61A7\u0100ct\u0492\u0497r;\uC000\u{1D49F}rok;\u4110\u0800NTacdfglmopqstux\u04BD\u04C0\u04C4\u04CB\u04DE\u04E2\u04E7\u04EE\u04F5\u0521\u052F\u0536\u0552\u055D\u0560\u0565G;\u414AH\u803B\xD0\u40D0cute\u803B\xC9\u40C9\u0180aiy\u04D2\u04D7\u04DCron;\u411Arc\u803B\xCA\u40CA;\u442Dot;\u4116r;\uC000\u{1D508}rave\u803B\xC8\u40C8ement;\u6208\u0100ap\u04FA\u04FEcr;\u4112ty\u0253\u0506\0\0\u0512mallSquare;\u65FBerySmallSquare;\u65AB\u0100gp\u0526\u052Aon;\u4118f;\uC000\u{1D53C}silon;\u4395u\u0100ai\u053C\u0549l\u0100;T\u0542\u0543\u6A75ilde;\u6242librium;\u61CC\u0100ci\u0557\u055Ar;\u6130m;\u6A73a;\u4397ml\u803B\xCB\u40CB\u0100ip\u056A\u056Fsts;\u6203onentialE;\u6147\u0280cfios\u0585\u0588\u058D\u05B2\u05CCy;\u4424r;\uC000\u{1D509}lled\u0253\u0597\0\0\u05A3mallSquare;\u65FCerySmallSquare;\u65AA\u0370\u05BA\0\u05BF\0\0\u05C4f;\uC000\u{1D53D}All;\u6200riertrf;\u6131c\xF2\u05CB\u0600JTabcdfgorst\u05E8\u05EC\u05EF\u05FA\u0600\u0612\u0616\u061B\u061D\u0623\u066C\u0672cy;\u4403\u803B>\u403Emma\u0100;d\u05F7\u05F8\u4393;\u43DCreve;\u411E\u0180eiy\u0607\u060C\u0610dil;\u4122rc;\u411C;\u4413ot;\u4120r;\uC000\u{1D50A};\u62D9pf;\uC000\u{1D53E}eater\u0300EFGLST\u0635\u0644\u064E\u0656\u065B\u0666qual\u0100;L\u063E\u063F\u6265ess;\u62DBullEqual;\u6267reater;\u6AA2ess;\u6277lantEqual;\u6A7Eilde;\u6273cr;\uC000\u{1D4A2};\u626B\u0400Aacfiosu\u0685\u068B\u0696\u069B\u069E\u06AA\u06BE\u06CARDcy;\u442A\u0100ct\u0690\u0694ek;\u42C7;\u405Eirc;\u4124r;\u610ClbertSpace;\u610B\u01F0\u06AF\0\u06B2f;\u610DizontalLine;\u6500\u0100ct\u06C3\u06C5\xF2\u06A9rok;\u4126mp\u0144\u06D0\u06D8ownHum\xF0\u012Fqual;\u624F\u0700EJOacdfgmnostu\u06FA\u06FE\u0703\u0707\u070E\u071A\u071E\u0721\u0728\u0744\u0778\u078B\u078F\u0795cy;\u4415lig;\u4132cy;\u4401cute\u803B\xCD\u40CD\u0100iy\u0713\u0718rc\u803B\xCE\u40CE;\u4418ot;\u4130r;\u6111rave\u803B\xCC\u40CC\u0180;ap\u0720\u072F\u073F\u0100cg\u0734\u0737r;\u412AinaryI;\u6148lie\xF3\u03DD\u01F4\u0749\0\u0762\u0100;e\u074D\u074E\u622C\u0100gr\u0753\u0758ral;\u622Bsection;\u62C2isible\u0100CT\u076C\u0772omma;\u6063imes;\u6062\u0180gpt\u077F\u0783\u0788on;\u412Ef;\uC000\u{1D540}a;\u4399cr;\u6110ilde;\u4128\u01EB\u079A\0\u079Ecy;\u4406l\u803B\xCF\u40CF\u0280cfosu\u07AC\u07B7\u07BC\u07C2\u07D0\u0100iy\u07B1\u07B5rc;\u4134;\u4419r;\uC000\u{1D50D}pf;\uC000\u{1D541}\u01E3\u07C7\0\u07CCr;\uC000\u{1D4A5}rcy;\u4408kcy;\u4404\u0380HJacfos\u07E4\u07E8\u07EC\u07F1\u07FD\u0802\u0808cy;\u4425cy;\u440Cppa;\u439A\u0100ey\u07F6\u07FBdil;\u4136;\u441Ar;\uC000\u{1D50E}pf;\uC000\u{1D542}cr;\uC000\u{1D4A6}\u0580JTaceflmost\u0825\u0829\u082C\u0850\u0863\u09B3\u09B8\u09C7\u09CD\u0A37\u0A47cy;\u4409\u803B<\u403C\u0280cmnpr\u0837\u083C\u0841\u0844\u084Dute;\u4139bda;\u439Bg;\u67EAlacetrf;\u6112r;\u619E\u0180aey\u0857\u085C\u0861ron;\u413Ddil;\u413B;\u441B\u0100fs\u0868\u0970t\u0500ACDFRTUVar\u087E\u08A9\u08B1\u08E0\u08E6\u08FC\u092F\u095B\u0390\u096A\u0100nr\u0883\u088FgleBracket;\u67E8row\u0180;BR\u0899\u089A\u089E\u6190ar;\u61E4ightArrow;\u61C6eiling;\u6308o\u01F5\u08B7\0\u08C3bleBracket;\u67E6n\u01D4\u08C8\0\u08D2eeVector;\u6961ector\u0100;B\u08DB\u08DC\u61C3ar;\u6959loor;\u630Aight\u0100AV\u08EF\u08F5rrow;\u6194ector;\u694E\u0100er\u0901\u0917e\u0180;AV\u0909\u090A\u0910\u62A3rrow;\u61A4ector;\u695Aiangle\u0180;BE\u0924\u0925\u0929\u62B2ar;\u69CFqual;\u62B4p\u0180DTV\u0937\u0942\u094CownVector;\u6951eeVector;\u6960ector\u0100;B\u0956\u0957\u61BFar;\u6958ector\u0100;B\u0965\u0966\u61BCar;\u6952ight\xE1\u039Cs\u0300EFGLST\u097E\u098B\u0995\u099D\u09A2\u09ADqualGreater;\u62DAullEqual;\u6266reater;\u6276ess;\u6AA1lantEqual;\u6A7Dilde;\u6272r;\uC000\u{1D50F}\u0100;e\u09BD\u09BE\u62D8ftarrow;\u61DAidot;\u413F\u0180npw\u09D4\u0A16\u0A1Bg\u0200LRlr\u09DE\u09F7\u0A02\u0A10eft\u0100AR\u09E6\u09ECrrow;\u67F5ightArrow;\u67F7ightArrow;\u67F6eft\u0100ar\u03B3\u0A0Aight\xE1\u03BFight\xE1\u03CAf;\uC000\u{1D543}er\u0100LR\u0A22\u0A2CeftArrow;\u6199ightArrow;\u6198\u0180cht\u0A3E\u0A40\u0A42\xF2\u084C;\u61B0rok;\u4141;\u626A\u0400acefiosu\u0A5A\u0A5D\u0A60\u0A77\u0A7C\u0A85\u0A8B\u0A8Ep;\u6905y;\u441C\u0100dl\u0A65\u0A6FiumSpace;\u605Flintrf;\u6133r;\uC000\u{1D510}nusPlus;\u6213pf;\uC000\u{1D544}c\xF2\u0A76;\u439C\u0480Jacefostu\u0AA3\u0AA7\u0AAD\u0AC0\u0B14\u0B19\u0D91\u0D97\u0D9Ecy;\u440Acute;\u4143\u0180aey\u0AB4\u0AB9\u0ABEron;\u4147dil;\u4145;\u441D\u0180gsw\u0AC7\u0AF0\u0B0Eative\u0180MTV\u0AD3\u0ADF\u0AE8ediumSpace;\u600Bhi\u0100cn\u0AE6\u0AD8\xEB\u0AD9eryThi\xEE\u0AD9ted\u0100GL\u0AF8\u0B06reaterGreate\xF2\u0673essLes\xF3\u0A48Line;\u400Ar;\uC000\u{1D511}\u0200Bnpt\u0B22\u0B28\u0B37\u0B3Areak;\u6060BreakingSpace;\u40A0f;\u6115\u0680;CDEGHLNPRSTV\u0B55\u0B56\u0B6A\u0B7C\u0BA1\u0BEB\u0C04\u0C5E\u0C84\u0CA6\u0CD8\u0D61\u0D85\u6AEC\u0100ou\u0B5B\u0B64ngruent;\u6262pCap;\u626DoubleVerticalBar;\u6226\u0180lqx\u0B83\u0B8A\u0B9Bement;\u6209ual\u0100;T\u0B92\u0B93\u6260ilde;\uC000\u2242\u0338ists;\u6204reater\u0380;EFGLST\u0BB6\u0BB7\u0BBD\u0BC9\u0BD3\u0BD8\u0BE5\u626Fqual;\u6271ullEqual;\uC000\u2267\u0338reater;\uC000\u226B\u0338ess;\u6279lantEqual;\uC000\u2A7E\u0338ilde;\u6275ump\u0144\u0BF2\u0BFDownHump;\uC000\u224E\u0338qual;\uC000\u224F\u0338e\u0100fs\u0C0A\u0C27tTriangle\u0180;BE\u0C1A\u0C1B\u0C21\u62EAar;\uC000\u29CF\u0338qual;\u62ECs\u0300;EGLST\u0C35\u0C36\u0C3C\u0C44\u0C4B\u0C58\u626Equal;\u6270reater;\u6278ess;\uC000\u226A\u0338lantEqual;\uC000\u2A7D\u0338ilde;\u6274ested\u0100GL\u0C68\u0C79reaterGreater;\uC000\u2AA2\u0338essLess;\uC000\u2AA1\u0338recedes\u0180;ES\u0C92\u0C93\u0C9B\u6280qual;\uC000\u2AAF\u0338lantEqual;\u62E0\u0100ei\u0CAB\u0CB9verseElement;\u620CghtTriangle\u0180;BE\u0CCB\u0CCC\u0CD2\u62EBar;\uC000\u29D0\u0338qual;\u62ED\u0100qu\u0CDD\u0D0CuareSu\u0100bp\u0CE8\u0CF9set\u0100;E\u0CF0\u0CF3\uC000\u228F\u0338qual;\u62E2erset\u0100;E\u0D03\u0D06\uC000\u2290\u0338qual;\u62E3\u0180bcp\u0D13\u0D24\u0D4Eset\u0100;E\u0D1B\u0D1E\uC000\u2282\u20D2qual;\u6288ceeds\u0200;EST\u0D32\u0D33\u0D3B\u0D46\u6281qual;\uC000\u2AB0\u0338lantEqual;\u62E1ilde;\uC000\u227F\u0338erset\u0100;E\u0D58\u0D5B\uC000\u2283\u20D2qual;\u6289ilde\u0200;EFT\u0D6E\u0D6F\u0D75\u0D7F\u6241qual;\u6244ullEqual;\u6247ilde;\u6249erticalBar;\u6224cr;\uC000\u{1D4A9}ilde\u803B\xD1\u40D1;\u439D\u0700Eacdfgmoprstuv\u0DBD\u0DC2\u0DC9\u0DD5\u0DDB\u0DE0\u0DE7\u0DFC\u0E02\u0E20\u0E22\u0E32\u0E3F\u0E44lig;\u4152cute\u803B\xD3\u40D3\u0100iy\u0DCE\u0DD3rc\u803B\xD4\u40D4;\u441Eblac;\u4150r;\uC000\u{1D512}rave\u803B\xD2\u40D2\u0180aei\u0DEE\u0DF2\u0DF6cr;\u414Cga;\u43A9cron;\u439Fpf;\uC000\u{1D546}enCurly\u0100DQ\u0E0E\u0E1AoubleQuote;\u601Cuote;\u6018;\u6A54\u0100cl\u0E27\u0E2Cr;\uC000\u{1D4AA}ash\u803B\xD8\u40D8i\u016C\u0E37\u0E3Cde\u803B\xD5\u40D5es;\u6A37ml\u803B\xD6\u40D6er\u0100BP\u0E4B\u0E60\u0100ar\u0E50\u0E53r;\u603Eac\u0100ek\u0E5A\u0E5C;\u63DEet;\u63B4arenthesis;\u63DC\u0480acfhilors\u0E7F\u0E87\u0E8A\u0E8F\u0E92\u0E94\u0E9D\u0EB0\u0EFCrtialD;\u6202y;\u441Fr;\uC000\u{1D513}i;\u43A6;\u43A0usMinus;\u40B1\u0100ip\u0EA2\u0EADncareplan\xE5\u069Df;\u6119\u0200;eio\u0EB9\u0EBA\u0EE0\u0EE4\u6ABBcedes\u0200;EST\u0EC8\u0EC9\u0ECF\u0EDA\u627Aqual;\u6AAFlantEqual;\u627Cilde;\u627Eme;\u6033\u0100dp\u0EE9\u0EEEuct;\u620Fortion\u0100;a\u0225\u0EF9l;\u621D\u0100ci\u0F01\u0F06r;\uC000\u{1D4AB};\u43A8\u0200Ufos\u0F11\u0F16\u0F1B\u0F1FOT\u803B"\u4022r;\uC000\u{1D514}pf;\u611Acr;\uC000\u{1D4AC}\u0600BEacefhiorsu\u0F3E\u0F43\u0F47\u0F60\u0F73\u0FA7\u0FAA\u0FAD\u1096\u10A9\u10B4\u10BEarr;\u6910G\u803B\xAE\u40AE\u0180cnr\u0F4E\u0F53\u0F56ute;\u4154g;\u67EBr\u0100;t\u0F5C\u0F5D\u61A0l;\u6916\u0180aey\u0F67\u0F6C\u0F71ron;\u4158dil;\u4156;\u4420\u0100;v\u0F78\u0F79\u611Cerse\u0100EU\u0F82\u0F99\u0100lq\u0F87\u0F8Eement;\u620Builibrium;\u61CBpEquilibrium;\u696Fr\xBB\u0F79o;\u43A1ght\u0400ACDFTUVa\u0FC1\u0FEB\u0FF3\u1022\u1028\u105B\u1087\u03D8\u0100nr\u0FC6\u0FD2gleBracket;\u67E9row\u0180;BL\u0FDC\u0FDD\u0FE1\u6192ar;\u61E5eftArrow;\u61C4eiling;\u6309o\u01F5\u0FF9\0\u1005bleBracket;\u67E7n\u01D4\u100A\0\u1014eeVector;\u695Dector\u0100;B\u101D\u101E\u61C2ar;\u6955loor;\u630B\u0100er\u102D\u1043e\u0180;AV\u1035\u1036\u103C\u62A2rrow;\u61A6ector;\u695Biangle\u0180;BE\u1050\u1051\u1055\u62B3ar;\u69D0qual;\u62B5p\u0180DTV\u1063\u106E\u1078ownVector;\u694FeeVector;\u695Cector\u0100;B\u1082\u1083\u61BEar;\u6954ector\u0100;B\u1091\u1092\u61C0ar;\u6953\u0100pu\u109B\u109Ef;\u611DndImplies;\u6970ightarrow;\u61DB\u0100ch\u10B9\u10BCr;\u611B;\u61B1leDelayed;\u69F4\u0680HOacfhimoqstu\u10E4\u10F1\u10F7\u10FD\u1119\u111E\u1151\u1156\u1161\u1167\u11B5\u11BB\u11BF\u0100Cc\u10E9\u10EEHcy;\u4429y;\u4428FTcy;\u442Ccute;\u415A\u0280;aeiy\u1108\u1109\u110E\u1113\u1117\u6ABCron;\u4160dil;\u415Erc;\u415C;\u4421r;\uC000\u{1D516}ort\u0200DLRU\u112A\u1134\u113E\u1149ownArrow\xBB\u041EeftArrow\xBB\u089AightArrow\xBB\u0FDDpArrow;\u6191gma;\u43A3allCircle;\u6218pf;\uC000\u{1D54A}\u0272\u116D\0\0\u1170t;\u621Aare\u0200;ISU\u117B\u117C\u1189\u11AF\u65A1ntersection;\u6293u\u0100bp\u118F\u119Eset\u0100;E\u1197\u1198\u628Fqual;\u6291erset\u0100;E\u11A8\u11A9\u6290qual;\u6292nion;\u6294cr;\uC000\u{1D4AE}ar;\u62C6\u0200bcmp\u11C8\u11DB\u1209\u120B\u0100;s\u11CD\u11CE\u62D0et\u0100;E\u11CD\u11D5qual;\u6286\u0100ch\u11E0\u1205eeds\u0200;EST\u11ED\u11EE\u11F4\u11FF\u627Bqual;\u6AB0lantEqual;\u627Dilde;\u627FTh\xE1\u0F8C;\u6211\u0180;es\u1212\u1213\u1223\u62D1rset\u0100;E\u121C\u121D\u6283qual;\u6287et\xBB\u1213\u0580HRSacfhiors\u123E\u1244\u1249\u1255\u125E\u1271\u1276\u129F\u12C2\u12C8\u12D1ORN\u803B\xDE\u40DEADE;\u6122\u0100Hc\u124E\u1252cy;\u440By;\u4426\u0100bu\u125A\u125C;\u4009;\u43A4\u0180aey\u1265\u126A\u126Fron;\u4164dil;\u4162;\u4422r;\uC000\u{1D517}\u0100ei\u127B\u1289\u01F2\u1280\0\u1287efore;\u6234a;\u4398\u0100cn\u128E\u1298kSpace;\uC000\u205F\u200ASpace;\u6009lde\u0200;EFT\u12AB\u12AC\u12B2\u12BC\u623Cqual;\u6243ullEqual;\u6245ilde;\u6248pf;\uC000\u{1D54B}ipleDot;\u60DB\u0100ct\u12D6\u12DBr;\uC000\u{1D4AF}rok;\u4166\u0AE1\u12F7\u130E\u131A\u1326\0\u132C\u1331\0\0\0\0\0\u1338\u133D\u1377\u1385\0\u13FF\u1404\u140A\u1410\u0100cr\u12FB\u1301ute\u803B\xDA\u40DAr\u0100;o\u1307\u1308\u619Fcir;\u6949r\u01E3\u1313\0\u1316y;\u440Eve;\u416C\u0100iy\u131E\u1323rc\u803B\xDB\u40DB;\u4423blac;\u4170r;\uC000\u{1D518}rave\u803B\xD9\u40D9acr;\u416A\u0100di\u1341\u1369er\u0100BP\u1348\u135D\u0100ar\u134D\u1350r;\u405Fac\u0100ek\u1357\u1359;\u63DFet;\u63B5arenthesis;\u63DDon\u0100;P\u1370\u1371\u62C3lus;\u628E\u0100gp\u137B\u137Fon;\u4172f;\uC000\u{1D54C}\u0400ADETadps\u1395\u13AE\u13B8\u13C4\u03E8\u13D2\u13D7\u13F3rrow\u0180;BD\u1150\u13A0\u13A4ar;\u6912ownArrow;\u61C5ownArrow;\u6195quilibrium;\u696Eee\u0100;A\u13CB\u13CC\u62A5rrow;\u61A5own\xE1\u03F3er\u0100LR\u13DE\u13E8eftArrow;\u6196ightArrow;\u6197i\u0100;l\u13F9\u13FA\u43D2on;\u43A5ing;\u416Ecr;\uC000\u{1D4B0}ilde;\u4168ml\u803B\xDC\u40DC\u0480Dbcdefosv\u1427\u142C\u1430\u1433\u143E\u1485\u148A\u1490\u1496ash;\u62ABar;\u6AEBy;\u4412ash\u0100;l\u143B\u143C\u62A9;\u6AE6\u0100er\u1443\u1445;\u62C1\u0180bty\u144C\u1450\u147Aar;\u6016\u0100;i\u144F\u1455cal\u0200BLST\u1461\u1465\u146A\u1474ar;\u6223ine;\u407Ceparator;\u6758ilde;\u6240ThinSpace;\u600Ar;\uC000\u{1D519}pf;\uC000\u{1D54D}cr;\uC000\u{1D4B1}dash;\u62AA\u0280cefos\u14A7\u14AC\u14B1\u14B6\u14BCirc;\u4174dge;\u62C0r;\uC000\u{1D51A}pf;\uC000\u{1D54E}cr;\uC000\u{1D4B2}\u0200fios\u14CB\u14D0\u14D2\u14D8r;\uC000\u{1D51B};\u439Epf;\uC000\u{1D54F}cr;\uC000\u{1D4B3}\u0480AIUacfosu\u14F1\u14F5\u14F9\u14FD\u1504\u150F\u1514\u151A\u1520cy;\u442Fcy;\u4407cy;\u442Ecute\u803B\xDD\u40DD\u0100iy\u1509\u150Drc;\u4176;\u442Br;\uC000\u{1D51C}pf;\uC000\u{1D550}cr;\uC000\u{1D4B4}ml;\u4178\u0400Hacdefos\u1535\u1539\u153F\u154B\u154F\u155D\u1560\u1564cy;\u4416cute;\u4179\u0100ay\u1544\u1549ron;\u417D;\u4417ot;\u417B\u01F2\u1554\0\u155BoWidt\xE8\u0AD9a;\u4396r;\u6128pf;\u6124cr;\uC000\u{1D4B5}\u0BE1\u1583\u158A\u1590\0\u15B0\u15B6\u15BF\0\0\0\0\u15C6\u15DB\u15EB\u165F\u166D\0\u1695\u169B\u16B2\u16B9\0\u16BEcute\u803B\xE1\u40E1reve;\u4103\u0300;Ediuy\u159C\u159D\u15A1\u15A3\u15A8\u15AD\u623E;\uC000\u223E\u0333;\u623Frc\u803B\xE2\u40E2te\u80BB\xB4\u0306;\u4430lig\u803B\xE6\u40E6\u0100;r\xB2\u15BA;\uC000\u{1D51E}rave\u803B\xE0\u40E0\u0100ep\u15CA\u15D6\u0100fp\u15CF\u15D4sym;\u6135\xE8\u15D3ha;\u43B1\u0100ap\u15DFc\u0100cl\u15E4\u15E7r;\u4101g;\u6A3F\u0264\u15F0\0\0\u160A\u0280;adsv\u15FA\u15FB\u15FF\u1601\u1607\u6227nd;\u6A55;\u6A5Clope;\u6A58;\u6A5A\u0380;elmrsz\u1618\u1619\u161B\u161E\u163F\u164F\u1659\u6220;\u69A4e\xBB\u1619sd\u0100;a\u1625\u1626\u6221\u0461\u1630\u1632\u1634\u1636\u1638\u163A\u163C\u163E;\u69A8;\u69A9;\u69AA;\u69AB;\u69AC;\u69AD;\u69AE;\u69AFt\u0100;v\u1645\u1646\u621Fb\u0100;d\u164C\u164D\u62BE;\u699D\u0100pt\u1654\u1657h;\u6222\xBB\xB9arr;\u637C\u0100gp\u1663\u1667on;\u4105f;\uC000\u{1D552}\u0380;Eaeiop\u12C1\u167B\u167D\u1682\u1684\u1687\u168A;\u6A70cir;\u6A6F;\u624Ad;\u624Bs;\u4027rox\u0100;e\u12C1\u1692\xF1\u1683ing\u803B\xE5\u40E5\u0180cty\u16A1\u16A6\u16A8r;\uC000\u{1D4B6};\u402Amp\u0100;e\u12C1\u16AF\xF1\u0288ilde\u803B\xE3\u40E3ml\u803B\xE4\u40E4\u0100ci\u16C2\u16C8onin\xF4\u0272nt;\u6A11\u0800Nabcdefiklnoprsu\u16ED\u16F1\u1730\u173C\u1743\u1748\u1778\u177D\u17E0\u17E6\u1839\u1850\u170D\u193D\u1948\u1970ot;\u6AED\u0100cr\u16F6\u171Ek\u0200ceps\u1700\u1705\u170D\u1713ong;\u624Cpsilon;\u43F6rime;\u6035im\u0100;e\u171A\u171B\u623Dq;\u62CD\u0176\u1722\u1726ee;\u62BDed\u0100;g\u172C\u172D\u6305e\xBB\u172Drk\u0100;t\u135C\u1737brk;\u63B6\u0100oy\u1701\u1741;\u4431quo;\u601E\u0280cmprt\u1753\u175B\u1761\u1764\u1768aus\u0100;e\u010A\u0109ptyv;\u69B0s\xE9\u170Cno\xF5\u0113\u0180ahw\u176F\u1771\u1773;\u43B2;\u6136een;\u626Cr;\uC000\u{1D51F}g\u0380costuvw\u178D\u179D\u17B3\u17C1\u17D5\u17DB\u17DE\u0180aiu\u1794\u1796\u179A\xF0\u0760rc;\u65EFp\xBB\u1371\u0180dpt\u17A4\u17A8\u17ADot;\u6A00lus;\u6A01imes;\u6A02\u0271\u17B9\0\0\u17BEcup;\u6A06ar;\u6605riangle\u0100du\u17CD\u17D2own;\u65BDp;\u65B3plus;\u6A04e\xE5\u1444\xE5\u14ADarow;\u690D\u0180ako\u17ED\u1826\u1835\u0100cn\u17F2\u1823k\u0180lst\u17FA\u05AB\u1802ozenge;\u69EBriangle\u0200;dlr\u1812\u1813\u1818\u181D\u65B4own;\u65BEeft;\u65C2ight;\u65B8k;\u6423\u01B1\u182B\0\u1833\u01B2\u182F\0\u1831;\u6592;\u65914;\u6593ck;\u6588\u0100eo\u183E\u184D\u0100;q\u1843\u1846\uC000=\u20E5uiv;\uC000\u2261\u20E5t;\u6310\u0200ptwx\u1859\u185E\u1867\u186Cf;\uC000\u{1D553}\u0100;t\u13CB\u1863om\xBB\u13CCtie;\u62C8\u0600DHUVbdhmptuv\u1885\u1896\u18AA\u18BB\u18D7\u18DB\u18EC\u18FF\u1905\u190A\u1910\u1921\u0200LRlr\u188E\u1890\u1892\u1894;\u6557;\u6554;\u6556;\u6553\u0280;DUdu\u18A1\u18A2\u18A4\u18A6\u18A8\u6550;\u6566;\u6569;\u6564;\u6567\u0200LRlr\u18B3\u18B5\u18B7\u18B9;\u655D;\u655A;\u655C;\u6559\u0380;HLRhlr\u18CA\u18CB\u18CD\u18CF\u18D1\u18D3\u18D5\u6551;\u656C;\u6563;\u6560;\u656B;\u6562;\u655Fox;\u69C9\u0200LRlr\u18E4\u18E6\u18E8\u18EA;\u6555;\u6552;\u6510;\u650C\u0280;DUdu\u06BD\u18F7\u18F9\u18FB\u18FD;\u6565;\u6568;\u652C;\u6534inus;\u629Flus;\u629Eimes;\u62A0\u0200LRlr\u1919\u191B\u191D\u191F;\u655B;\u6558;\u6518;\u6514\u0380;HLRhlr\u1930\u1931\u1933\u1935\u1937\u1939\u193B\u6502;\u656A;\u6561;\u655E;\u653C;\u6524;\u651C\u0100ev\u0123\u1942bar\u803B\xA6\u40A6\u0200ceio\u1951\u1956\u195A\u1960r;\uC000\u{1D4B7}mi;\u604Fm\u0100;e\u171A\u171Cl\u0180;bh\u1968\u1969\u196B\u405C;\u69C5sub;\u67C8\u016C\u1974\u197El\u0100;e\u1979\u197A\u6022t\xBB\u197Ap\u0180;Ee\u012F\u1985\u1987;\u6AAE\u0100;q\u06DC\u06DB\u0CE1\u19A7\0\u19E8\u1A11\u1A15\u1A32\0\u1A37\u1A50\0\0\u1AB4\0\0\u1AC1\0\0\u1B21\u1B2E\u1B4D\u1B52\0\u1BFD\0\u1C0C\u0180cpr\u19AD\u19B2\u19DDute;\u4107\u0300;abcds\u19BF\u19C0\u19C4\u19CA\u19D5\u19D9\u6229nd;\u6A44rcup;\u6A49\u0100au\u19CF\u19D2p;\u6A4Bp;\u6A47ot;\u6A40;\uC000\u2229\uFE00\u0100eo\u19E2\u19E5t;\u6041\xEE\u0693\u0200aeiu\u19F0\u19FB\u1A01\u1A05\u01F0\u19F5\0\u19F8s;\u6A4Don;\u410Ddil\u803B\xE7\u40E7rc;\u4109ps\u0100;s\u1A0C\u1A0D\u6A4Cm;\u6A50ot;\u410B\u0180dmn\u1A1B\u1A20\u1A26il\u80BB\xB8\u01ADptyv;\u69B2t\u8100\xA2;e\u1A2D\u1A2E\u40A2r\xE4\u01B2r;\uC000\u{1D520}\u0180cei\u1A3D\u1A40\u1A4Dy;\u4447ck\u0100;m\u1A47\u1A48\u6713ark\xBB\u1A48;\u43C7r\u0380;Ecefms\u1A5F\u1A60\u1A62\u1A6B\u1AA4\u1AAA\u1AAE\u65CB;\u69C3\u0180;el\u1A69\u1A6A\u1A6D\u42C6q;\u6257e\u0261\u1A74\0\0\u1A88rrow\u0100lr\u1A7C\u1A81eft;\u61BAight;\u61BB\u0280RSacd\u1A92\u1A94\u1A96\u1A9A\u1A9F\xBB\u0F47;\u64C8st;\u629Birc;\u629Aash;\u629Dnint;\u6A10id;\u6AEFcir;\u69C2ubs\u0100;u\u1ABB\u1ABC\u6663it\xBB\u1ABC\u02EC\u1AC7\u1AD4\u1AFA\0\u1B0Aon\u0100;e\u1ACD\u1ACE\u403A\u0100;q\xC7\xC6\u026D\u1AD9\0\0\u1AE2a\u0100;t\u1ADE\u1ADF\u402C;\u4040\u0180;fl\u1AE8\u1AE9\u1AEB\u6201\xEE\u1160e\u0100mx\u1AF1\u1AF6ent\xBB\u1AE9e\xF3\u024D\u01E7\u1AFE\0\u1B07\u0100;d\u12BB\u1B02ot;\u6A6Dn\xF4\u0246\u0180fry\u1B10\u1B14\u1B17;\uC000\u{1D554}o\xE4\u0254\u8100\xA9;s\u0155\u1B1Dr;\u6117\u0100ao\u1B25\u1B29rr;\u61B5ss;\u6717\u0100cu\u1B32\u1B37r;\uC000\u{1D4B8}\u0100bp\u1B3C\u1B44\u0100;e\u1B41\u1B42\u6ACF;\u6AD1\u0100;e\u1B49\u1B4A\u6AD0;\u6AD2dot;\u62EF\u0380delprvw\u1B60\u1B6C\u1B77\u1B82\u1BAC\u1BD4\u1BF9arr\u0100lr\u1B68\u1B6A;\u6938;\u6935\u0270\u1B72\0\0\u1B75r;\u62DEc;\u62DFarr\u0100;p\u1B7F\u1B80\u61B6;\u693D\u0300;bcdos\u1B8F\u1B90\u1B96\u1BA1\u1BA5\u1BA8\u622Arcap;\u6A48\u0100au\u1B9B\u1B9Ep;\u6A46p;\u6A4Aot;\u628Dr;\u6A45;\uC000\u222A\uFE00\u0200alrv\u1BB5\u1BBF\u1BDE\u1BE3rr\u0100;m\u1BBC\u1BBD\u61B7;\u693Cy\u0180evw\u1BC7\u1BD4\u1BD8q\u0270\u1BCE\0\0\u1BD2re\xE3\u1B73u\xE3\u1B75ee;\u62CEedge;\u62CFen\u803B\xA4\u40A4earrow\u0100lr\u1BEE\u1BF3eft\xBB\u1B80ight\xBB\u1BBDe\xE4\u1BDD\u0100ci\u1C01\u1C07onin\xF4\u01F7nt;\u6231lcty;\u632D\u0980AHabcdefhijlorstuwz\u1C38\u1C3B\u1C3F\u1C5D\u1C69\u1C75\u1C8A\u1C9E\u1CAC\u1CB7\u1CFB\u1CFF\u1D0D\u1D7B\u1D91\u1DAB\u1DBB\u1DC6\u1DCDr\xF2\u0381ar;\u6965\u0200glrs\u1C48\u1C4D\u1C52\u1C54ger;\u6020eth;\u6138\xF2\u1133h\u0100;v\u1C5A\u1C5B\u6010\xBB\u090A\u016B\u1C61\u1C67arow;\u690Fa\xE3\u0315\u0100ay\u1C6E\u1C73ron;\u410F;\u4434\u0180;ao\u0332\u1C7C\u1C84\u0100gr\u02BF\u1C81r;\u61CAtseq;\u6A77\u0180glm\u1C91\u1C94\u1C98\u803B\xB0\u40B0ta;\u43B4ptyv;\u69B1\u0100ir\u1CA3\u1CA8sht;\u697F;\uC000\u{1D521}ar\u0100lr\u1CB3\u1CB5\xBB\u08DC\xBB\u101E\u0280aegsv\u1CC2\u0378\u1CD6\u1CDC\u1CE0m\u0180;os\u0326\u1CCA\u1CD4nd\u0100;s\u0326\u1CD1uit;\u6666amma;\u43DDin;\u62F2\u0180;io\u1CE7\u1CE8\u1CF8\u40F7de\u8100\xF7;o\u1CE7\u1CF0ntimes;\u62C7n\xF8\u1CF7cy;\u4452c\u026F\u1D06\0\0\u1D0Arn;\u631Eop;\u630D\u0280lptuw\u1D18\u1D1D\u1D22\u1D49\u1D55lar;\u4024f;\uC000\u{1D555}\u0280;emps\u030B\u1D2D\u1D37\u1D3D\u1D42q\u0100;d\u0352\u1D33ot;\u6251inus;\u6238lus;\u6214quare;\u62A1blebarwedg\xE5\xFAn\u0180adh\u112E\u1D5D\u1D67ownarrow\xF3\u1C83arpoon\u0100lr\u1D72\u1D76ef\xF4\u1CB4igh\xF4\u1CB6\u0162\u1D7F\u1D85karo\xF7\u0F42\u026F\u1D8A\0\0\u1D8Ern;\u631Fop;\u630C\u0180cot\u1D98\u1DA3\u1DA6\u0100ry\u1D9D\u1DA1;\uC000\u{1D4B9};\u4455l;\u69F6rok;\u4111\u0100dr\u1DB0\u1DB4ot;\u62F1i\u0100;f\u1DBA\u1816\u65BF\u0100ah\u1DC0\u1DC3r\xF2\u0429a\xF2\u0FA6angle;\u69A6\u0100ci\u1DD2\u1DD5y;\u445Fgrarr;\u67FF\u0900Dacdefglmnopqrstux\u1E01\u1E09\u1E19\u1E38\u0578\u1E3C\u1E49\u1E61\u1E7E\u1EA5\u1EAF\u1EBD\u1EE1\u1F2A\u1F37\u1F44\u1F4E\u1F5A\u0100Do\u1E06\u1D34o\xF4\u1C89\u0100cs\u1E0E\u1E14ute\u803B\xE9\u40E9ter;\u6A6E\u0200aioy\u1E22\u1E27\u1E31\u1E36ron;\u411Br\u0100;c\u1E2D\u1E2E\u6256\u803B\xEA\u40EAlon;\u6255;\u444Dot;\u4117\u0100Dr\u1E41\u1E45ot;\u6252;\uC000\u{1D522}\u0180;rs\u1E50\u1E51\u1E57\u6A9Aave\u803B\xE8\u40E8\u0100;d\u1E5C\u1E5D\u6A96ot;\u6A98\u0200;ils\u1E6A\u1E6B\u1E72\u1E74\u6A99nters;\u63E7;\u6113\u0100;d\u1E79\u1E7A\u6A95ot;\u6A97\u0180aps\u1E85\u1E89\u1E97cr;\u4113ty\u0180;sv\u1E92\u1E93\u1E95\u6205et\xBB\u1E93p\u01001;\u1E9D\u1EA4\u0133\u1EA1\u1EA3;\u6004;\u6005\u6003\u0100gs\u1EAA\u1EAC;\u414Bp;\u6002\u0100gp\u1EB4\u1EB8on;\u4119f;\uC000\u{1D556}\u0180als\u1EC4\u1ECE\u1ED2r\u0100;s\u1ECA\u1ECB\u62D5l;\u69E3us;\u6A71i\u0180;lv\u1EDA\u1EDB\u1EDF\u43B5on\xBB\u1EDB;\u43F5\u0200csuv\u1EEA\u1EF3\u1F0B\u1F23\u0100io\u1EEF\u1E31rc\xBB\u1E2E\u0269\u1EF9\0\0\u1EFB\xED\u0548ant\u0100gl\u1F02\u1F06tr\xBB\u1E5Dess\xBB\u1E7A\u0180aei\u1F12\u1F16\u1F1Als;\u403Dst;\u625Fv\u0100;D\u0235\u1F20D;\u6A78parsl;\u69E5\u0100Da\u1F2F\u1F33ot;\u6253rr;\u6971\u0180cdi\u1F3E\u1F41\u1EF8r;\u612Fo\xF4\u0352\u0100ah\u1F49\u1F4B;\u43B7\u803B\xF0\u40F0\u0100mr\u1F53\u1F57l\u803B\xEB\u40EBo;\u60AC\u0180cip\u1F61\u1F64\u1F67l;\u4021s\xF4\u056E\u0100eo\u1F6C\u1F74ctatio\xEE\u0559nential\xE5\u0579\u09E1\u1F92\0\u1F9E\0\u1FA1\u1FA7\0\0\u1FC6\u1FCC\0\u1FD3\0\u1FE6\u1FEA\u2000\0\u2008\u205Allingdotse\xF1\u1E44y;\u4444male;\u6640\u0180ilr\u1FAD\u1FB3\u1FC1lig;\u8000\uFB03\u0269\u1FB9\0\0\u1FBDg;\u8000\uFB00ig;\u8000\uFB04;\uC000\u{1D523}lig;\u8000\uFB01lig;\uC000fj\u0180alt\u1FD9\u1FDC\u1FE1t;\u666Dig;\u8000\uFB02ns;\u65B1of;\u4192\u01F0\u1FEE\0\u1FF3f;\uC000\u{1D557}\u0100ak\u05BF\u1FF7\u0100;v\u1FFC\u1FFD\u62D4;\u6AD9artint;\u6A0D\u0100ao\u200C\u2055\u0100cs\u2011\u2052\u03B1\u201A\u2030\u2038\u2045\u2048\0\u2050\u03B2\u2022\u2025\u2027\u202A\u202C\0\u202E\u803B\xBD\u40BD;\u6153\u803B\xBC\u40BC;\u6155;\u6159;\u615B\u01B3\u2034\0\u2036;\u6154;\u6156\u02B4\u203E\u2041\0\0\u2043\u803B\xBE\u40BE;\u6157;\u615C5;\u6158\u01B6\u204C\0\u204E;\u615A;\u615D8;\u615El;\u6044wn;\u6322cr;\uC000\u{1D4BB}\u0880Eabcdefgijlnorstv\u2082\u2089\u209F\u20A5\u20B0\u20B4\u20F0\u20F5\u20FA\u20FF\u2103\u2112\u2138\u0317\u213E\u2152\u219E\u0100;l\u064D\u2087;\u6A8C\u0180cmp\u2090\u2095\u209Dute;\u41F5ma\u0100;d\u209C\u1CDA\u43B3;\u6A86reve;\u411F\u0100iy\u20AA\u20AErc;\u411D;\u4433ot;\u4121\u0200;lqs\u063E\u0642\u20BD\u20C9\u0180;qs\u063E\u064C\u20C4lan\xF4\u0665\u0200;cdl\u0665\u20D2\u20D5\u20E5c;\u6AA9ot\u0100;o\u20DC\u20DD\u6A80\u0100;l\u20E2\u20E3\u6A82;\u6A84\u0100;e\u20EA\u20ED\uC000\u22DB\uFE00s;\u6A94r;\uC000\u{1D524}\u0100;g\u0673\u061Bmel;\u6137cy;\u4453\u0200;Eaj\u065A\u210C\u210E\u2110;\u6A92;\u6AA5;\u6AA4\u0200Eaes\u211B\u211D\u2129\u2134;\u6269p\u0100;p\u2123\u2124\u6A8Arox\xBB\u2124\u0100;q\u212E\u212F\u6A88\u0100;q\u212E\u211Bim;\u62E7pf;\uC000\u{1D558}\u0100ci\u2143\u2146r;\u610Am\u0180;el\u066B\u214E\u2150;\u6A8E;\u6A90\u8300>;cdlqr\u05EE\u2160\u216A\u216E\u2173\u2179\u0100ci\u2165\u2167;\u6AA7r;\u6A7Aot;\u62D7Par;\u6995uest;\u6A7C\u0280adels\u2184\u216A\u2190\u0656\u219B\u01F0\u2189\0\u218Epro\xF8\u209Er;\u6978q\u0100lq\u063F\u2196les\xF3\u2088i\xED\u066B\u0100en\u21A3\u21ADrtneqq;\uC000\u2269\uFE00\xC5\u21AA\u0500Aabcefkosy\u21C4\u21C7\u21F1\u21F5\u21FA\u2218\u221D\u222F\u2268\u227Dr\xF2\u03A0\u0200ilmr\u21D0\u21D4\u21D7\u21DBrs\xF0\u1484f\xBB\u2024il\xF4\u06A9\u0100dr\u21E0\u21E4cy;\u444A\u0180;cw\u08F4\u21EB\u21EFir;\u6948;\u61ADar;\u610Firc;\u4125\u0180alr\u2201\u220E\u2213rts\u0100;u\u2209\u220A\u6665it\xBB\u220Alip;\u6026con;\u62B9r;\uC000\u{1D525}s\u0100ew\u2223\u2229arow;\u6925arow;\u6926\u0280amopr\u223A\u223E\u2243\u225E\u2263rr;\u61FFtht;\u623Bk\u0100lr\u2249\u2253eftarrow;\u61A9ightarrow;\u61AAf;\uC000\u{1D559}bar;\u6015\u0180clt\u226F\u2274\u2278r;\uC000\u{1D4BD}as\xE8\u21F4rok;\u4127\u0100bp\u2282\u2287ull;\u6043hen\xBB\u1C5B\u0AE1\u22A3\0\u22AA\0\u22B8\u22C5\u22CE\0\u22D5\u22F3\0\0\u22F8\u2322\u2367\u2362\u237F\0\u2386\u23AA\u23B4cute\u803B\xED\u40ED\u0180;iy\u0771\u22B0\u22B5rc\u803B\xEE\u40EE;\u4438\u0100cx\u22BC\u22BFy;\u4435cl\u803B\xA1\u40A1\u0100fr\u039F\u22C9;\uC000\u{1D526}rave\u803B\xEC\u40EC\u0200;ino\u073E\u22DD\u22E9\u22EE\u0100in\u22E2\u22E6nt;\u6A0Ct;\u622Dfin;\u69DCta;\u6129lig;\u4133\u0180aop\u22FE\u231A\u231D\u0180cgt\u2305\u2308\u2317r;\u412B\u0180elp\u071F\u230F\u2313in\xE5\u078Ear\xF4\u0720h;\u4131f;\u62B7ed;\u41B5\u0280;cfot\u04F4\u232C\u2331\u233D\u2341are;\u6105in\u0100;t\u2338\u2339\u621Eie;\u69DDdo\xF4\u2319\u0280;celp\u0757\u234C\u2350\u235B\u2361al;\u62BA\u0100gr\u2355\u2359er\xF3\u1563\xE3\u234Darhk;\u6A17rod;\u6A3C\u0200cgpt\u236F\u2372\u2376\u237By;\u4451on;\u412Ff;\uC000\u{1D55A}a;\u43B9uest\u803B\xBF\u40BF\u0100ci\u238A\u238Fr;\uC000\u{1D4BE}n\u0280;Edsv\u04F4\u239B\u239D\u23A1\u04F3;\u62F9ot;\u62F5\u0100;v\u23A6\u23A7\u62F4;\u62F3\u0100;i\u0777\u23AElde;\u4129\u01EB\u23B8\0\u23BCcy;\u4456l\u803B\xEF\u40EF\u0300cfmosu\u23CC\u23D7\u23DC\u23E1\u23E7\u23F5\u0100iy\u23D1\u23D5rc;\u4135;\u4439r;\uC000\u{1D527}ath;\u4237pf;\uC000\u{1D55B}\u01E3\u23EC\0\u23F1r;\uC000\u{1D4BF}rcy;\u4458kcy;\u4454\u0400acfghjos\u240B\u2416\u2422\u2427\u242D\u2431\u2435\u243Bppa\u0100;v\u2413\u2414\u43BA;\u43F0\u0100ey\u241B\u2420dil;\u4137;\u443Ar;\uC000\u{1D528}reen;\u4138cy;\u4445cy;\u445Cpf;\uC000\u{1D55C}cr;\uC000\u{1D4C0}\u0B80ABEHabcdefghjlmnoprstuv\u2470\u2481\u2486\u248D\u2491\u250E\u253D\u255A\u2580\u264E\u265E\u2665\u2679\u267D\u269A\u26B2\u26D8\u275D\u2768\u278B\u27C0\u2801\u2812\u0180art\u2477\u247A\u247Cr\xF2\u09C6\xF2\u0395ail;\u691Barr;\u690E\u0100;g\u0994\u248B;\u6A8Bar;\u6962\u0963\u24A5\0\u24AA\0\u24B1\0\0\0\0\0\u24B5\u24BA\0\u24C6\u24C8\u24CD\0\u24F9ute;\u413Amptyv;\u69B4ra\xEE\u084Cbda;\u43BBg\u0180;dl\u088E\u24C1\u24C3;\u6991\xE5\u088E;\u6A85uo\u803B\xAB\u40ABr\u0400;bfhlpst\u0899\u24DE\u24E6\u24E9\u24EB\u24EE\u24F1\u24F5\u0100;f\u089D\u24E3s;\u691Fs;\u691D\xEB\u2252p;\u61ABl;\u6939im;\u6973l;\u61A2\u0180;ae\u24FF\u2500\u2504\u6AABil;\u6919\u0100;s\u2509\u250A\u6AAD;\uC000\u2AAD\uFE00\u0180abr\u2515\u2519\u251Drr;\u690Crk;\u6772\u0100ak\u2522\u252Cc\u0100ek\u2528\u252A;\u407B;\u405B\u0100es\u2531\u2533;\u698Bl\u0100du\u2539\u253B;\u698F;\u698D\u0200aeuy\u2546\u254B\u2556\u2558ron;\u413E\u0100di\u2550\u2554il;\u413C\xEC\u08B0\xE2\u2529;\u443B\u0200cqrs\u2563\u2566\u256D\u257Da;\u6936uo\u0100;r\u0E19\u1746\u0100du\u2572\u2577har;\u6967shar;\u694Bh;\u61B2\u0280;fgqs\u258B\u258C\u0989\u25F3\u25FF\u6264t\u0280ahlrt\u2598\u25A4\u25B7\u25C2\u25E8rrow\u0100;t\u0899\u25A1a\xE9\u24F6arpoon\u0100du\u25AF\u25B4own\xBB\u045Ap\xBB\u0966eftarrows;\u61C7ight\u0180ahs\u25CD\u25D6\u25DErrow\u0100;s\u08F4\u08A7arpoon\xF3\u0F98quigarro\xF7\u21F0hreetimes;\u62CB\u0180;qs\u258B\u0993\u25FAlan\xF4\u09AC\u0280;cdgs\u09AC\u260A\u260D\u261D\u2628c;\u6AA8ot\u0100;o\u2614\u2615\u6A7F\u0100;r\u261A\u261B\u6A81;\u6A83\u0100;e\u2622\u2625\uC000\u22DA\uFE00s;\u6A93\u0280adegs\u2633\u2639\u263D\u2649\u264Bppro\xF8\u24C6ot;\u62D6q\u0100gq\u2643\u2645\xF4\u0989gt\xF2\u248C\xF4\u099Bi\xED\u09B2\u0180ilr\u2655\u08E1\u265Asht;\u697C;\uC000\u{1D529}\u0100;E\u099C\u2663;\u6A91\u0161\u2669\u2676r\u0100du\u25B2\u266E\u0100;l\u0965\u2673;\u696Alk;\u6584cy;\u4459\u0280;acht\u0A48\u2688\u268B\u2691\u2696r\xF2\u25C1orne\xF2\u1D08ard;\u696Bri;\u65FA\u0100io\u269F\u26A4dot;\u4140ust\u0100;a\u26AC\u26AD\u63B0che\xBB\u26AD\u0200Eaes\u26BB\u26BD\u26C9\u26D4;\u6268p\u0100;p\u26C3\u26C4\u6A89rox\xBB\u26C4\u0100;q\u26CE\u26CF\u6A87\u0100;q\u26CE\u26BBim;\u62E6\u0400abnoptwz\u26E9\u26F4\u26F7\u271A\u272F\u2741\u2747\u2750\u0100nr\u26EE\u26F1g;\u67ECr;\u61FDr\xEB\u08C1g\u0180lmr\u26FF\u270D\u2714eft\u0100ar\u09E6\u2707ight\xE1\u09F2apsto;\u67FCight\xE1\u09FDparrow\u0100lr\u2725\u2729ef\xF4\u24EDight;\u61AC\u0180afl\u2736\u2739\u273Dr;\u6985;\uC000\u{1D55D}us;\u6A2Dimes;\u6A34\u0161\u274B\u274Fst;\u6217\xE1\u134E\u0180;ef\u2757\u2758\u1800\u65CAnge\xBB\u2758ar\u0100;l\u2764\u2765\u4028t;\u6993\u0280achmt\u2773\u2776\u277C\u2785\u2787r\xF2\u08A8orne\xF2\u1D8Car\u0100;d\u0F98\u2783;\u696D;\u600Eri;\u62BF\u0300achiqt\u2798\u279D\u0A40\u27A2\u27AE\u27BBquo;\u6039r;\uC000\u{1D4C1}m\u0180;eg\u09B2\u27AA\u27AC;\u6A8D;\u6A8F\u0100bu\u252A\u27B3o\u0100;r\u0E1F\u27B9;\u601Arok;\u4142\u8400<;cdhilqr\u082B\u27D2\u2639\u27DC\u27E0\u27E5\u27EA\u27F0\u0100ci\u27D7\u27D9;\u6AA6r;\u6A79re\xE5\u25F2mes;\u62C9arr;\u6976uest;\u6A7B\u0100Pi\u27F5\u27F9ar;\u6996\u0180;ef\u2800\u092D\u181B\u65C3r\u0100du\u2807\u280Dshar;\u694Ahar;\u6966\u0100en\u2817\u2821rtneqq;\uC000\u2268\uFE00\xC5\u281E\u0700Dacdefhilnopsu\u2840\u2845\u2882\u288E\u2893\u28A0\u28A5\u28A8\u28DA\u28E2\u28E4\u0A83\u28F3\u2902Dot;\u623A\u0200clpr\u284E\u2852\u2863\u287Dr\u803B\xAF\u40AF\u0100et\u2857\u2859;\u6642\u0100;e\u285E\u285F\u6720se\xBB\u285F\u0100;s\u103B\u2868to\u0200;dlu\u103B\u2873\u2877\u287Bow\xEE\u048Cef\xF4\u090F\xF0\u13D1ker;\u65AE\u0100oy\u2887\u288Cmma;\u6A29;\u443Cash;\u6014asuredangle\xBB\u1626r;\uC000\u{1D52A}o;\u6127\u0180cdn\u28AF\u28B4\u28C9ro\u803B\xB5\u40B5\u0200;acd\u1464\u28BD\u28C0\u28C4s\xF4\u16A7ir;\u6AF0ot\u80BB\xB7\u01B5us\u0180;bd\u28D2\u1903\u28D3\u6212\u0100;u\u1D3C\u28D8;\u6A2A\u0163\u28DE\u28E1p;\u6ADB\xF2\u2212\xF0\u0A81\u0100dp\u28E9\u28EEels;\u62A7f;\uC000\u{1D55E}\u0100ct\u28F8\u28FDr;\uC000\u{1D4C2}pos\xBB\u159D\u0180;lm\u2909\u290A\u290D\u43BCtimap;\u62B8\u0C00GLRVabcdefghijlmoprstuvw\u2942\u2953\u297E\u2989\u2998\u29DA\u29E9\u2A15\u2A1A\u2A58\u2A5D\u2A83\u2A95\u2AA4\u2AA8\u2B04\u2B07\u2B44\u2B7F\u2BAE\u2C34\u2C67\u2C7C\u2CE9\u0100gt\u2947\u294B;\uC000\u22D9\u0338\u0100;v\u2950\u0BCF\uC000\u226B\u20D2\u0180elt\u295A\u2972\u2976ft\u0100ar\u2961\u2967rrow;\u61CDightarrow;\u61CE;\uC000\u22D8\u0338\u0100;v\u297B\u0C47\uC000\u226A\u20D2ightarrow;\u61CF\u0100Dd\u298E\u2993ash;\u62AFash;\u62AE\u0280bcnpt\u29A3\u29A7\u29AC\u29B1\u29CCla\xBB\u02DEute;\u4144g;\uC000\u2220\u20D2\u0280;Eiop\u0D84\u29BC\u29C0\u29C5\u29C8;\uC000\u2A70\u0338d;\uC000\u224B\u0338s;\u4149ro\xF8\u0D84ur\u0100;a\u29D3\u29D4\u666El\u0100;s\u29D3\u0B38\u01F3\u29DF\0\u29E3p\u80BB\xA0\u0B37mp\u0100;e\u0BF9\u0C00\u0280aeouy\u29F4\u29FE\u2A03\u2A10\u2A13\u01F0\u29F9\0\u29FB;\u6A43on;\u4148dil;\u4146ng\u0100;d\u0D7E\u2A0Aot;\uC000\u2A6D\u0338p;\u6A42;\u443Dash;\u6013\u0380;Aadqsx\u0B92\u2A29\u2A2D\u2A3B\u2A41\u2A45\u2A50rr;\u61D7r\u0100hr\u2A33\u2A36k;\u6924\u0100;o\u13F2\u13F0ot;\uC000\u2250\u0338ui\xF6\u0B63\u0100ei\u2A4A\u2A4Ear;\u6928\xED\u0B98ist\u0100;s\u0BA0\u0B9Fr;\uC000\u{1D52B}\u0200Eest\u0BC5\u2A66\u2A79\u2A7C\u0180;qs\u0BBC\u2A6D\u0BE1\u0180;qs\u0BBC\u0BC5\u2A74lan\xF4\u0BE2i\xED\u0BEA\u0100;r\u0BB6\u2A81\xBB\u0BB7\u0180Aap\u2A8A\u2A8D\u2A91r\xF2\u2971rr;\u61AEar;\u6AF2\u0180;sv\u0F8D\u2A9C\u0F8C\u0100;d\u2AA1\u2AA2\u62FC;\u62FAcy;\u445A\u0380AEadest\u2AB7\u2ABA\u2ABE\u2AC2\u2AC5\u2AF6\u2AF9r\xF2\u2966;\uC000\u2266\u0338rr;\u619Ar;\u6025\u0200;fqs\u0C3B\u2ACE\u2AE3\u2AEFt\u0100ar\u2AD4\u2AD9rro\xF7\u2AC1ightarro\xF7\u2A90\u0180;qs\u0C3B\u2ABA\u2AEAlan\xF4\u0C55\u0100;s\u0C55\u2AF4\xBB\u0C36i\xED\u0C5D\u0100;r\u0C35\u2AFEi\u0100;e\u0C1A\u0C25i\xE4\u0D90\u0100pt\u2B0C\u2B11f;\uC000\u{1D55F}\u8180\xAC;in\u2B19\u2B1A\u2B36\u40ACn\u0200;Edv\u0B89\u2B24\u2B28\u2B2E;\uC000\u22F9\u0338ot;\uC000\u22F5\u0338\u01E1\u0B89\u2B33\u2B35;\u62F7;\u62F6i\u0100;v\u0CB8\u2B3C\u01E1\u0CB8\u2B41\u2B43;\u62FE;\u62FD\u0180aor\u2B4B\u2B63\u2B69r\u0200;ast\u0B7B\u2B55\u2B5A\u2B5Flle\xEC\u0B7Bl;\uC000\u2AFD\u20E5;\uC000\u2202\u0338lint;\u6A14\u0180;ce\u0C92\u2B70\u2B73u\xE5\u0CA5\u0100;c\u0C98\u2B78\u0100;e\u0C92\u2B7D\xF1\u0C98\u0200Aait\u2B88\u2B8B\u2B9D\u2BA7r\xF2\u2988rr\u0180;cw\u2B94\u2B95\u2B99\u619B;\uC000\u2933\u0338;\uC000\u219D\u0338ghtarrow\xBB\u2B95ri\u0100;e\u0CCB\u0CD6\u0380chimpqu\u2BBD\u2BCD\u2BD9\u2B04\u0B78\u2BE4\u2BEF\u0200;cer\u0D32\u2BC6\u0D37\u2BC9u\xE5\u0D45;\uC000\u{1D4C3}ort\u026D\u2B05\0\0\u2BD6ar\xE1\u2B56m\u0100;e\u0D6E\u2BDF\u0100;q\u0D74\u0D73su\u0100bp\u2BEB\u2BED\xE5\u0CF8\xE5\u0D0B\u0180bcp\u2BF6\u2C11\u2C19\u0200;Ees\u2BFF\u2C00\u0D22\u2C04\u6284;\uC000\u2AC5\u0338et\u0100;e\u0D1B\u2C0Bq\u0100;q\u0D23\u2C00c\u0100;e\u0D32\u2C17\xF1\u0D38\u0200;Ees\u2C22\u2C23\u0D5F\u2C27\u6285;\uC000\u2AC6\u0338et\u0100;e\u0D58\u2C2Eq\u0100;q\u0D60\u2C23\u0200gilr\u2C3D\u2C3F\u2C45\u2C47\xEC\u0BD7lde\u803B\xF1\u40F1\xE7\u0C43iangle\u0100lr\u2C52\u2C5Ceft\u0100;e\u0C1A\u2C5A\xF1\u0C26ight\u0100;e\u0CCB\u2C65\xF1\u0CD7\u0100;m\u2C6C\u2C6D\u43BD\u0180;es\u2C74\u2C75\u2C79\u4023ro;\u6116p;\u6007\u0480DHadgilrs\u2C8F\u2C94\u2C99\u2C9E\u2CA3\u2CB0\u2CB6\u2CD3\u2CE3ash;\u62ADarr;\u6904p;\uC000\u224D\u20D2ash;\u62AC\u0100et\u2CA8\u2CAC;\uC000\u2265\u20D2;\uC000>\u20D2nfin;\u69DE\u0180Aet\u2CBD\u2CC1\u2CC5rr;\u6902;\uC000\u2264\u20D2\u0100;r\u2CCA\u2CCD\uC000<\u20D2ie;\uC000\u22B4\u20D2\u0100At\u2CD8\u2CDCrr;\u6903rie;\uC000\u22B5\u20D2im;\uC000\u223C\u20D2\u0180Aan\u2CF0\u2CF4\u2D02rr;\u61D6r\u0100hr\u2CFA\u2CFDk;\u6923\u0100;o\u13E7\u13E5ear;\u6927\u1253\u1A95\0\0\0\0\0\0\0\0\0\0\0\0\0\u2D2D\0\u2D38\u2D48\u2D60\u2D65\u2D72\u2D84\u1B07\0\0\u2D8D\u2DAB\0\u2DC8\u2DCE\0\u2DDC\u2E19\u2E2B\u2E3E\u2E43\u0100cs\u2D31\u1A97ute\u803B\xF3\u40F3\u0100iy\u2D3C\u2D45r\u0100;c\u1A9E\u2D42\u803B\xF4\u40F4;\u443E\u0280abios\u1AA0\u2D52\u2D57\u01C8\u2D5Alac;\u4151v;\u6A38old;\u69BClig;\u4153\u0100cr\u2D69\u2D6Dir;\u69BF;\uC000\u{1D52C}\u036F\u2D79\0\0\u2D7C\0\u2D82n;\u42DBave\u803B\xF2\u40F2;\u69C1\u0100bm\u2D88\u0DF4ar;\u69B5\u0200acit\u2D95\u2D98\u2DA5\u2DA8r\xF2\u1A80\u0100ir\u2D9D\u2DA0r;\u69BEoss;\u69BBn\xE5\u0E52;\u69C0\u0180aei\u2DB1\u2DB5\u2DB9cr;\u414Dga;\u43C9\u0180cdn\u2DC0\u2DC5\u01CDron;\u43BF;\u69B6pf;\uC000\u{1D560}\u0180ael\u2DD4\u2DD7\u01D2r;\u69B7rp;\u69B9\u0380;adiosv\u2DEA\u2DEB\u2DEE\u2E08\u2E0D\u2E10\u2E16\u6228r\xF2\u1A86\u0200;efm\u2DF7\u2DF8\u2E02\u2E05\u6A5Dr\u0100;o\u2DFE\u2DFF\u6134f\xBB\u2DFF\u803B\xAA\u40AA\u803B\xBA\u40BAgof;\u62B6r;\u6A56lope;\u6A57;\u6A5B\u0180clo\u2E1F\u2E21\u2E27\xF2\u2E01ash\u803B\xF8\u40F8l;\u6298i\u016C\u2E2F\u2E34de\u803B\xF5\u40F5es\u0100;a\u01DB\u2E3As;\u6A36ml\u803B\xF6\u40F6bar;\u633D\u0AE1\u2E5E\0\u2E7D\0\u2E80\u2E9D\0\u2EA2\u2EB9\0\0\u2ECB\u0E9C\0\u2F13\0\0\u2F2B\u2FBC\0\u2FC8r\u0200;ast\u0403\u2E67\u2E72\u0E85\u8100\xB6;l\u2E6D\u2E6E\u40B6le\xEC\u0403\u0269\u2E78\0\0\u2E7Bm;\u6AF3;\u6AFDy;\u443Fr\u0280cimpt\u2E8B\u2E8F\u2E93\u1865\u2E97nt;\u4025od;\u402Eil;\u6030enk;\u6031r;\uC000\u{1D52D}\u0180imo\u2EA8\u2EB0\u2EB4\u0100;v\u2EAD\u2EAE\u43C6;\u43D5ma\xF4\u0A76ne;\u660E\u0180;tv\u2EBF\u2EC0\u2EC8\u43C0chfork\xBB\u1FFD;\u43D6\u0100au\u2ECF\u2EDFn\u0100ck\u2ED5\u2EDDk\u0100;h\u21F4\u2EDB;\u610E\xF6\u21F4s\u0480;abcdemst\u2EF3\u2EF4\u1908\u2EF9\u2EFD\u2F04\u2F06\u2F0A\u2F0E\u402Bcir;\u6A23ir;\u6A22\u0100ou\u1D40\u2F02;\u6A25;\u6A72n\u80BB\xB1\u0E9Dim;\u6A26wo;\u6A27\u0180ipu\u2F19\u2F20\u2F25ntint;\u6A15f;\uC000\u{1D561}nd\u803B\xA3\u40A3\u0500;Eaceinosu\u0EC8\u2F3F\u2F41\u2F44\u2F47\u2F81\u2F89\u2F92\u2F7E\u2FB6;\u6AB3p;\u6AB7u\xE5\u0ED9\u0100;c\u0ECE\u2F4C\u0300;acens\u0EC8\u2F59\u2F5F\u2F66\u2F68\u2F7Eppro\xF8\u2F43urlye\xF1\u0ED9\xF1\u0ECE\u0180aes\u2F6F\u2F76\u2F7Approx;\u6AB9qq;\u6AB5im;\u62E8i\xED\u0EDFme\u0100;s\u2F88\u0EAE\u6032\u0180Eas\u2F78\u2F90\u2F7A\xF0\u2F75\u0180dfp\u0EEC\u2F99\u2FAF\u0180als\u2FA0\u2FA5\u2FAAlar;\u632Eine;\u6312urf;\u6313\u0100;t\u0EFB\u2FB4\xEF\u0EFBrel;\u62B0\u0100ci\u2FC0\u2FC5r;\uC000\u{1D4C5};\u43C8ncsp;\u6008\u0300fiopsu\u2FDA\u22E2\u2FDF\u2FE5\u2FEB\u2FF1r;\uC000\u{1D52E}pf;\uC000\u{1D562}rime;\u6057cr;\uC000\u{1D4C6}\u0180aeo\u2FF8\u3009\u3013t\u0100ei\u2FFE\u3005rnion\xF3\u06B0nt;\u6A16st\u0100;e\u3010\u3011\u403F\xF1\u1F19\xF4\u0F14\u0A80ABHabcdefhilmnoprstux\u3040\u3051\u3055\u3059\u30E0\u310E\u312B\u3147\u3162\u3172\u318E\u3206\u3215\u3224\u3229\u3258\u326E\u3272\u3290\u32B0\u32B7\u0180art\u3047\u304A\u304Cr\xF2\u10B3\xF2\u03DDail;\u691Car\xF2\u1C65ar;\u6964\u0380cdenqrt\u3068\u3075\u3078\u307F\u308F\u3094\u30CC\u0100eu\u306D\u3071;\uC000\u223D\u0331te;\u4155i\xE3\u116Emptyv;\u69B3g\u0200;del\u0FD1\u3089\u308B\u308D;\u6992;\u69A5\xE5\u0FD1uo\u803B\xBB\u40BBr\u0580;abcfhlpstw\u0FDC\u30AC\u30AF\u30B7\u30B9\u30BC\u30BE\u30C0\u30C3\u30C7\u30CAp;\u6975\u0100;f\u0FE0\u30B4s;\u6920;\u6933s;\u691E\xEB\u225D\xF0\u272El;\u6945im;\u6974l;\u61A3;\u619D\u0100ai\u30D1\u30D5il;\u691Ao\u0100;n\u30DB\u30DC\u6236al\xF3\u0F1E\u0180abr\u30E7\u30EA\u30EEr\xF2\u17E5rk;\u6773\u0100ak\u30F3\u30FDc\u0100ek\u30F9\u30FB;\u407D;\u405D\u0100es\u3102\u3104;\u698Cl\u0100du\u310A\u310C;\u698E;\u6990\u0200aeuy\u3117\u311C\u3127\u3129ron;\u4159\u0100di\u3121\u3125il;\u4157\xEC\u0FF2\xE2\u30FA;\u4440\u0200clqs\u3134\u3137\u313D\u3144a;\u6937dhar;\u6969uo\u0100;r\u020E\u020Dh;\u61B3\u0180acg\u314E\u315F\u0F44l\u0200;ips\u0F78\u3158\u315B\u109Cn\xE5\u10BBar\xF4\u0FA9t;\u65AD\u0180ilr\u3169\u1023\u316Esht;\u697D;\uC000\u{1D52F}\u0100ao\u3177\u3186r\u0100du\u317D\u317F\xBB\u047B\u0100;l\u1091\u3184;\u696C\u0100;v\u318B\u318C\u43C1;\u43F1\u0180gns\u3195\u31F9\u31FCht\u0300ahlrst\u31A4\u31B0\u31C2\u31D8\u31E4\u31EErrow\u0100;t\u0FDC\u31ADa\xE9\u30C8arpoon\u0100du\u31BB\u31BFow\xEE\u317Ep\xBB\u1092eft\u0100ah\u31CA\u31D0rrow\xF3\u0FEAarpoon\xF3\u0551ightarrows;\u61C9quigarro\xF7\u30CBhreetimes;\u62CCg;\u42DAingdotse\xF1\u1F32\u0180ahm\u320D\u3210\u3213r\xF2\u0FEAa\xF2\u0551;\u600Foust\u0100;a\u321E\u321F\u63B1che\xBB\u321Fmid;\u6AEE\u0200abpt\u3232\u323D\u3240\u3252\u0100nr\u3237\u323Ag;\u67EDr;\u61FEr\xEB\u1003\u0180afl\u3247\u324A\u324Er;\u6986;\uC000\u{1D563}us;\u6A2Eimes;\u6A35\u0100ap\u325D\u3267r\u0100;g\u3263\u3264\u4029t;\u6994olint;\u6A12ar\xF2\u31E3\u0200achq\u327B\u3280\u10BC\u3285quo;\u603Ar;\uC000\u{1D4C7}\u0100bu\u30FB\u328Ao\u0100;r\u0214\u0213\u0180hir\u3297\u329B\u32A0re\xE5\u31F8mes;\u62CAi\u0200;efl\u32AA\u1059\u1821\u32AB\u65B9tri;\u69CEluhar;\u6968;\u611E\u0D61\u32D5\u32DB\u32DF\u332C\u3338\u3371\0\u337A\u33A4\0\0\u33EC\u33F0\0\u3428\u3448\u345A\u34AD\u34B1\u34CA\u34F1\0\u3616\0\0\u3633cute;\u415Bqu\xEF\u27BA\u0500;Eaceinpsy\u11ED\u32F3\u32F5\u32FF\u3302\u330B\u330F\u331F\u3326\u3329;\u6AB4\u01F0\u32FA\0\u32FC;\u6AB8on;\u4161u\xE5\u11FE\u0100;d\u11F3\u3307il;\u415Frc;\u415D\u0180Eas\u3316\u3318\u331B;\u6AB6p;\u6ABAim;\u62E9olint;\u6A13i\xED\u1204;\u4441ot\u0180;be\u3334\u1D47\u3335\u62C5;\u6A66\u0380Aacmstx\u3346\u334A\u3357\u335B\u335E\u3363\u336Drr;\u61D8r\u0100hr\u3350\u3352\xEB\u2228\u0100;o\u0A36\u0A34t\u803B\xA7\u40A7i;\u403Bwar;\u6929m\u0100in\u3369\xF0nu\xF3\xF1t;\u6736r\u0100;o\u3376\u2055\uC000\u{1D530}\u0200acoy\u3382\u3386\u3391\u33A0rp;\u666F\u0100hy\u338B\u338Fcy;\u4449;\u4448rt\u026D\u3399\0\0\u339Ci\xE4\u1464ara\xEC\u2E6F\u803B\xAD\u40AD\u0100gm\u33A8\u33B4ma\u0180;fv\u33B1\u33B2\u33B2\u43C3;\u43C2\u0400;deglnpr\u12AB\u33C5\u33C9\u33CE\u33D6\u33DE\u33E1\u33E6ot;\u6A6A\u0100;q\u12B1\u12B0\u0100;E\u33D3\u33D4\u6A9E;\u6AA0\u0100;E\u33DB\u33DC\u6A9D;\u6A9Fe;\u6246lus;\u6A24arr;\u6972ar\xF2\u113D\u0200aeit\u33F8\u3408\u340F\u3417\u0100ls\u33FD\u3404lsetm\xE9\u336Ahp;\u6A33parsl;\u69E4\u0100dl\u1463\u3414e;\u6323\u0100;e\u341C\u341D\u6AAA\u0100;s\u3422\u3423\u6AAC;\uC000\u2AAC\uFE00\u0180flp\u342E\u3433\u3442tcy;\u444C\u0100;b\u3438\u3439\u402F\u0100;a\u343E\u343F\u69C4r;\u633Ff;\uC000\u{1D564}a\u0100dr\u344D\u0402es\u0100;u\u3454\u3455\u6660it\xBB\u3455\u0180csu\u3460\u3479\u349F\u0100au\u3465\u346Fp\u0100;s\u1188\u346B;\uC000\u2293\uFE00p\u0100;s\u11B4\u3475;\uC000\u2294\uFE00u\u0100bp\u347F\u348F\u0180;es\u1197\u119C\u3486et\u0100;e\u1197\u348D\xF1\u119D\u0180;es\u11A8\u11AD\u3496et\u0100;e\u11A8\u349D\xF1\u11AE\u0180;af\u117B\u34A6\u05B0r\u0165\u34AB\u05B1\xBB\u117Car\xF2\u1148\u0200cemt\u34B9\u34BE\u34C2\u34C5r;\uC000\u{1D4C8}tm\xEE\xF1i\xEC\u3415ar\xE6\u11BE\u0100ar\u34CE\u34D5r\u0100;f\u34D4\u17BF\u6606\u0100an\u34DA\u34EDight\u0100ep\u34E3\u34EApsilo\xEE\u1EE0h\xE9\u2EAFs\xBB\u2852\u0280bcmnp\u34FB\u355E\u1209\u358B\u358E\u0480;Edemnprs\u350E\u350F\u3511\u3515\u351E\u3523\u352C\u3531\u3536\u6282;\u6AC5ot;\u6ABD\u0100;d\u11DA\u351Aot;\u6AC3ult;\u6AC1\u0100Ee\u3528\u352A;\u6ACB;\u628Alus;\u6ABFarr;\u6979\u0180eiu\u353D\u3552\u3555t\u0180;en\u350E\u3545\u354Bq\u0100;q\u11DA\u350Feq\u0100;q\u352B\u3528m;\u6AC7\u0100bp\u355A\u355C;\u6AD5;\u6AD3c\u0300;acens\u11ED\u356C\u3572\u3579\u357B\u3326ppro\xF8\u32FAurlye\xF1\u11FE\xF1\u11F3\u0180aes\u3582\u3588\u331Bppro\xF8\u331Aq\xF1\u3317g;\u666A\u0680123;Edehlmnps\u35A9\u35AC\u35AF\u121C\u35B2\u35B4\u35C0\u35C9\u35D5\u35DA\u35DF\u35E8\u35ED\u803B\xB9\u40B9\u803B\xB2\u40B2\u803B\xB3\u40B3;\u6AC6\u0100os\u35B9\u35BCt;\u6ABEub;\u6AD8\u0100;d\u1222\u35C5ot;\u6AC4s\u0100ou\u35CF\u35D2l;\u67C9b;\u6AD7arr;\u697Bult;\u6AC2\u0100Ee\u35E4\u35E6;\u6ACC;\u628Blus;\u6AC0\u0180eiu\u35F4\u3609\u360Ct\u0180;en\u121C\u35FC\u3602q\u0100;q\u1222\u35B2eq\u0100;q\u35E7\u35E4m;\u6AC8\u0100bp\u3611\u3613;\u6AD4;\u6AD6\u0180Aan\u361C\u3620\u362Drr;\u61D9r\u0100hr\u3626\u3628\xEB\u222E\u0100;o\u0A2B\u0A29war;\u692Alig\u803B\xDF\u40DF\u0BE1\u3651\u365D\u3660\u12CE\u3673\u3679\0\u367E\u36C2\0\0\0\0\0\u36DB\u3703\0\u3709\u376C\0\0\0\u3787\u0272\u3656\0\0\u365Bget;\u6316;\u43C4r\xEB\u0E5F\u0180aey\u3666\u366B\u3670ron;\u4165dil;\u4163;\u4442lrec;\u6315r;\uC000\u{1D531}\u0200eiko\u3686\u369D\u36B5\u36BC\u01F2\u368B\0\u3691e\u01004f\u1284\u1281a\u0180;sv\u3698\u3699\u369B\u43B8ym;\u43D1\u0100cn\u36A2\u36B2k\u0100as\u36A8\u36AEppro\xF8\u12C1im\xBB\u12ACs\xF0\u129E\u0100as\u36BA\u36AE\xF0\u12C1rn\u803B\xFE\u40FE\u01EC\u031F\u36C6\u22E7es\u8180\xD7;bd\u36CF\u36D0\u36D8\u40D7\u0100;a\u190F\u36D5r;\u6A31;\u6A30\u0180eps\u36E1\u36E3\u3700\xE1\u2A4D\u0200;bcf\u0486\u36EC\u36F0\u36F4ot;\u6336ir;\u6AF1\u0100;o\u36F9\u36FC\uC000\u{1D565}rk;\u6ADA\xE1\u3362rime;\u6034\u0180aip\u370F\u3712\u3764d\xE5\u1248\u0380adempst\u3721\u374D\u3740\u3751\u3757\u375C\u375Fngle\u0280;dlqr\u3730\u3731\u3736\u3740\u3742\u65B5own\xBB\u1DBBeft\u0100;e\u2800\u373E\xF1\u092E;\u625Cight\u0100;e\u32AA\u374B\xF1\u105Aot;\u65ECinus;\u6A3Alus;\u6A39b;\u69CDime;\u6A3Bezium;\u63E2\u0180cht\u3772\u377D\u3781\u0100ry\u3777\u377B;\uC000\u{1D4C9};\u4446cy;\u445Brok;\u4167\u0100io\u378B\u378Ex\xF4\u1777head\u0100lr\u3797\u37A0eftarro\xF7\u084Fightarrow\xBB\u0F5D\u0900AHabcdfghlmoprstuw\u37D0\u37D3\u37D7\u37E4\u37F0\u37FC\u380E\u381C\u3823\u3834\u3851\u385D\u386B\u38A9\u38CC\u38D2\u38EA\u38F6r\xF2\u03EDar;\u6963\u0100cr\u37DC\u37E2ute\u803B\xFA\u40FA\xF2\u1150r\u01E3\u37EA\0\u37EDy;\u445Eve;\u416D\u0100iy\u37F5\u37FArc\u803B\xFB\u40FB;\u4443\u0180abh\u3803\u3806\u380Br\xF2\u13ADlac;\u4171a\xF2\u13C3\u0100ir\u3813\u3818sht;\u697E;\uC000\u{1D532}rave\u803B\xF9\u40F9\u0161\u3827\u3831r\u0100lr\u382C\u382E\xBB\u0957\xBB\u1083lk;\u6580\u0100ct\u3839\u384D\u026F\u383F\0\0\u384Arn\u0100;e\u3845\u3846\u631Cr\xBB\u3846op;\u630Fri;\u65F8\u0100al\u3856\u385Acr;\u416B\u80BB\xA8\u0349\u0100gp\u3862\u3866on;\u4173f;\uC000\u{1D566}\u0300adhlsu\u114B\u3878\u387D\u1372\u3891\u38A0own\xE1\u13B3arpoon\u0100lr\u3888\u388Cef\xF4\u382Digh\xF4\u382Fi\u0180;hl\u3899\u389A\u389C\u43C5\xBB\u13FAon\xBB\u389Aparrows;\u61C8\u0180cit\u38B0\u38C4\u38C8\u026F\u38B6\0\0\u38C1rn\u0100;e\u38BC\u38BD\u631Dr\xBB\u38BDop;\u630Eng;\u416Fri;\u65F9cr;\uC000\u{1D4CA}\u0180dir\u38D9\u38DD\u38E2ot;\u62F0lde;\u4169i\u0100;f\u3730\u38E8\xBB\u1813\u0100am\u38EF\u38F2r\xF2\u38A8l\u803B\xFC\u40FCangle;\u69A7\u0780ABDacdeflnoprsz\u391C\u391F\u3929\u392D\u39B5\u39B8\u39BD\u39DF\u39E4\u39E8\u39F3\u39F9\u39FD\u3A01\u3A20r\xF2\u03F7ar\u0100;v\u3926\u3927\u6AE8;\u6AE9as\xE8\u03E1\u0100nr\u3932\u3937grt;\u699C\u0380eknprst\u34E3\u3946\u394B\u3952\u395D\u3964\u3996app\xE1\u2415othin\xE7\u1E96\u0180hir\u34EB\u2EC8\u3959op\xF4\u2FB5\u0100;h\u13B7\u3962\xEF\u318D\u0100iu\u3969\u396Dgm\xE1\u33B3\u0100bp\u3972\u3984setneq\u0100;q\u397D\u3980\uC000\u228A\uFE00;\uC000\u2ACB\uFE00setneq\u0100;q\u398F\u3992\uC000\u228B\uFE00;\uC000\u2ACC\uFE00\u0100hr\u399B\u399Fet\xE1\u369Ciangle\u0100lr\u39AA\u39AFeft\xBB\u0925ight\xBB\u1051y;\u4432ash\xBB\u1036\u0180elr\u39C4\u39D2\u39D7\u0180;be\u2DEA\u39CB\u39CFar;\u62BBq;\u625Alip;\u62EE\u0100bt\u39DC\u1468a\xF2\u1469r;\uC000\u{1D533}tr\xE9\u39AEsu\u0100bp\u39EF\u39F1\xBB\u0D1C\xBB\u0D59pf;\uC000\u{1D567}ro\xF0\u0EFBtr\xE9\u39B4\u0100cu\u3A06\u3A0Br;\uC000\u{1D4CB}\u0100bp\u3A10\u3A18n\u0100Ee\u3980\u3A16\xBB\u397En\u0100Ee\u3992\u3A1E\xBB\u3990igzag;\u699A\u0380cefoprs\u3A36\u3A3B\u3A56\u3A5B\u3A54\u3A61\u3A6Airc;\u4175\u0100di\u3A40\u3A51\u0100bg\u3A45\u3A49ar;\u6A5Fe\u0100;q\u15FA\u3A4F;\u6259erp;\u6118r;\uC000\u{1D534}pf;\uC000\u{1D568}\u0100;e\u1479\u3A66at\xE8\u1479cr;\uC000\u{1D4CC}\u0AE3\u178E\u3A87\0\u3A8B\0\u3A90\u3A9B\0\0\u3A9D\u3AA8\u3AAB\u3AAF\0\0\u3AC3\u3ACE\0\u3AD8\u17DC\u17DFtr\xE9\u17D1r;\uC000\u{1D535}\u0100Aa\u3A94\u3A97r\xF2\u03C3r\xF2\u09F6;\u43BE\u0100Aa\u3AA1\u3AA4r\xF2\u03B8r\xF2\u09EBa\xF0\u2713is;\u62FB\u0180dpt\u17A4\u3AB5\u3ABE\u0100fl\u3ABA\u17A9;\uC000\u{1D569}im\xE5\u17B2\u0100Aa\u3AC7\u3ACAr\xF2\u03CEr\xF2\u0A01\u0100cq\u3AD2\u17B8r;\uC000\u{1D4CD}\u0100pt\u17D6\u3ADCr\xE9\u17D4\u0400acefiosu\u3AF0\u3AFD\u3B08\u3B0C\u3B11\u3B15\u3B1B\u3B21c\u0100uy\u3AF6\u3AFBte\u803B\xFD\u40FD;\u444F\u0100iy\u3B02\u3B06rc;\u4177;\u444Bn\u803B\xA5\u40A5r;\uC000\u{1D536}cy;\u4457pf;\uC000\u{1D56A}cr;\uC000\u{1D4CE}\u0100cm\u3B26\u3B29y;\u444El\u803B\xFF\u40FF\u0500acdefhiosw\u3B42\u3B48\u3B54\u3B58\u3B64\u3B69\u3B6D\u3B74\u3B7A\u3B80cute;\u417A\u0100ay\u3B4D\u3B52ron;\u417E;\u4437ot;\u417C\u0100et\u3B5D\u3B61tr\xE6\u155Fa;\u43B6r;\uC000\u{1D537}cy;\u4436grarr;\u61DDpf;\uC000\u{1D56B}cr;\uC000\u{1D4CF}\u0100jn\u3B85\u3B87;\u600Dj;\u600C'.split("").map(e => e.charCodeAt(0))),
    c0,
    go = new Map([[0, 65533], [128, 8364], [130, 8218], [131, 402], [132, 8222], [133, 8230], [134, 8224], [135, 8225], [136, 710], [137, 8240], [138, 352], [139, 8249], [140, 338], [142, 381], [145, 8216], [146, 8217], [147, 8220], [148, 8221], [149, 8226], [150, 8211], [151, 8212], [152, 732], [153, 8482], [154, 353], [155, 8250], [156, 339], [158, 382], [159, 376]]),
    B1 = (c0 = String.fromCodePoint) !== null && c0 !== void 0 ? c0 : function (e) {
      let t = "";
      return e > 65535 && (e -= 65536, t += String.fromCharCode(e >>> 10 & 1023 | 55296), e = 56320 | e & 1023), t += String.fromCharCode(e), t;
    };
  function To(e) {
    var t;
    return e >= 55296 && e <= 57343 || e > 1114111 ? 65533 : (t = go.get(e)) !== null && t !== void 0 ? t : e;
  }
  var z;
  (function (e) {
    e[e.NUM = 35] = "NUM", e[e.SEMI = 59] = "SEMI", e[e.EQUALS = 61] = "EQUALS", e[e.ZERO = 48] = "ZERO", e[e.NINE = 57] = "NINE", e[e.LOWER_A = 97] = "LOWER_A", e[e.LOWER_F = 102] = "LOWER_F", e[e.LOWER_X = 120] = "LOWER_X", e[e.LOWER_Z = 122] = "LOWER_Z", e[e.UPPER_A = 65] = "UPPER_A", e[e.UPPER_F = 70] = "UPPER_F", e[e.UPPER_Z = 90] = "UPPER_Z";
  })(z || (z = {}));
  var Fo = 32,
    Ue;
  (function (e) {
    e[e.VALUE_LENGTH = 49152] = "VALUE_LENGTH", e[e.BRANCH_LENGTH = 16256] = "BRANCH_LENGTH", e[e.JUMP_TABLE = 127] = "JUMP_TABLE";
  })(Ue || (Ue = {}));
  function l0(e) {
    return e >= z.ZERO && e <= z.NINE;
  }
  function _o(e) {
    return e >= z.UPPER_A && e <= z.UPPER_F || e >= z.LOWER_A && e <= z.LOWER_F;
  }
  function yo(e) {
    return e >= z.UPPER_A && e <= z.UPPER_Z || e >= z.LOWER_A && e <= z.LOWER_Z || l0(e);
  }
  function bo(e) {
    return e === z.EQUALS || yo(e);
  }
  var $;
  (function (e) {
    e[e.EntityStart = 0] = "EntityStart", e[e.NumericStart = 1] = "NumericStart", e[e.NumericDecimal = 2] = "NumericDecimal", e[e.NumericHex = 3] = "NumericHex", e[e.NamedEntity = 4] = "NamedEntity";
  })($ || ($ = {}));
  var Ie;
  (function (e) {
    e[e.Legacy = 0] = "Legacy", e[e.Strict = 1] = "Strict", e[e.Attribute = 2] = "Attribute";
  })(Ie || (Ie = {}));
  var So = class {
    constructor(e, t, n) {
      this.decodeTree = e, this.emitCodePoint = t, this.errors = n, this.state = $.EntityStart, this.consumed = 1, this.result = 0, this.treeIndex = 0, this.excess = 1, this.decodeMode = Ie.Strict;
    }
    startEntity(e) {
      this.decodeMode = e, this.state = $.EntityStart, this.result = 0, this.treeIndex = 0, this.excess = 1, this.consumed = 1;
    }
    write(e, t) {
      switch (this.state) {
        case $.EntityStart:
          return e.charCodeAt(t) === z.NUM ? (this.state = $.NumericStart, this.consumed += 1, this.stateNumericStart(e, t + 1)) : (this.state = $.NamedEntity, this.stateNamedEntity(e, t));
        case $.NumericStart:
          return this.stateNumericStart(e, t);
        case $.NumericDecimal:
          return this.stateNumericDecimal(e, t);
        case $.NumericHex:
          return this.stateNumericHex(e, t);
        case $.NamedEntity:
          return this.stateNamedEntity(e, t);
      }
    }
    stateNumericStart(e, t) {
      return t >= e.length ? -1 : (e.charCodeAt(t) | Fo) === z.LOWER_X ? (this.state = $.NumericHex, this.consumed += 1, this.stateNumericHex(e, t + 1)) : (this.state = $.NumericDecimal, this.stateNumericDecimal(e, t));
    }
    addToNumericResult(e, t, n, r) {
      if (t !== n) {
        let a = n - t;
        this.result = this.result * Math.pow(r, a) + Number.parseInt(e.substr(t, a), r), this.consumed += a;
      }
    }
    stateNumericHex(e, t) {
      let n = t;
      for (; t < e.length;) {
        let r = e.charCodeAt(t);
        if (l0(r) || _o(r)) t += 1;else return this.addToNumericResult(e, n, t, 16), this.emitNumericEntity(r, 3);
      }
      return this.addToNumericResult(e, n, t, 16), -1;
    }
    stateNumericDecimal(e, t) {
      let n = t;
      for (; t < e.length;) {
        let r = e.charCodeAt(t);
        if (l0(r)) t += 1;else return this.addToNumericResult(e, n, t, 10), this.emitNumericEntity(r, 2);
      }
      return this.addToNumericResult(e, n, t, 10), -1;
    }
    emitNumericEntity(e, t) {
      var n;
      if (this.consumed <= t) return (n = this.errors) === null || n === void 0 || n.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
      if (e === z.SEMI) this.consumed += 1;else if (this.decodeMode === Ie.Strict) return 0;
      return this.emitCodePoint(To(this.result), this.consumed), this.errors && (e !== z.SEMI && this.errors.missingSemicolonAfterCharacterReference(), this.errors.validateNumericCharacterReference(this.result)), this.consumed;
    }
    stateNamedEntity(e, t) {
      let n = this.decodeTree,
        r = n[this.treeIndex],
        a = (r & Ue.VALUE_LENGTH) >> 14;
      for (; t < e.length; t++, this.excess++) {
        let i = e.charCodeAt(t);
        if (this.treeIndex = Io(n, r, this.treeIndex + Math.max(1, a), i), this.treeIndex < 0) return this.result === 0 || this.decodeMode === Ie.Attribute && (a === 0 || bo(i)) ? 0 : this.emitNotTerminatedNamedEntity();
        if (r = n[this.treeIndex], a = (r & Ue.VALUE_LENGTH) >> 14, a !== 0) {
          if (i === z.SEMI) return this.emitNamedEntityData(this.treeIndex, a, this.consumed + this.excess);
          this.decodeMode !== Ie.Strict && (this.result = this.treeIndex, this.consumed += this.excess, this.excess = 0);
        }
      }
      return -1;
    }
    emitNotTerminatedNamedEntity() {
      var e;
      let t = this.result,
        n = this.decodeTree,
        r = (n[t] & Ue.VALUE_LENGTH) >> 14;
      return this.emitNamedEntityData(t, r, this.consumed), (e = this.errors) === null || e === void 0 || e.missingSemicolonAfterCharacterReference(), this.consumed;
    }
    emitNamedEntityData(e, t, n) {
      let r = this.decodeTree;
      return this.emitCodePoint(t === 1 ? r[e] & ~Ue.VALUE_LENGTH : r[e + 1], n), t === 3 && this.emitCodePoint(r[e + 2], n), n;
    }
    end() {
      var e;
      switch (this.state) {
        case $.NamedEntity:
          return this.result !== 0 && (this.decodeMode !== Ie.Attribute || this.result === this.treeIndex) ? this.emitNotTerminatedNamedEntity() : 0;
        case $.NumericDecimal:
          return this.emitNumericEntity(0, 2);
        case $.NumericHex:
          return this.emitNumericEntity(0, 3);
        case $.NumericStart:
          return (e = this.errors) === null || e === void 0 || e.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
        case $.EntityStart:
          return 0;
      }
    }
  };
  function Io(e, t, n, r) {
    let a = (t & Ue.BRANCH_LENGTH) >> 7,
      i = t & Ue.JUMP_TABLE;
    if (a === 0) return i !== 0 && r === i ? n : -1;
    if (i) {
      let c = r - i;
      return c < 0 || c >= a ? -1 : e[n + c] - 1;
    }
    let s = n,
      l = s + a - 1;
    for (; s <= l;) {
      let c = s + l >>> 1,
        E = e[c];
      if (E < r) s = c + 1;else if (E > r) l = c - 1;else return e[c + a];
    }
    return -1;
  }
  var nu = {};
  le(nu, {
    ATTRS: () => We,
    DOCUMENT_MODE: () => de,
    NS: () => p,
    NUMBERED_HEADERS: () => nn,
    SPECIAL_ELEMENTS: () => ru,
    TAG_ID: () => u,
    TAG_NAMES: () => d,
    getTagID: () => _t,
    hasUnescapedText: () => uu
  });
  var p;
  (function (e) {
    e.HTML = "http://www.w3.org/1999/xhtml", e.MATHML = "http://www.w3.org/1998/Math/MathML", e.SVG = "http://www.w3.org/2000/svg", e.XLINK = "http://www.w3.org/1999/xlink", e.XML = "http://www.w3.org/XML/1998/namespace", e.XMLNS = "http://www.w3.org/2000/xmlns/";
  })(p || (p = {}));
  var We;
  (function (e) {
    e.TYPE = "type", e.ACTION = "action", e.ENCODING = "encoding", e.PROMPT = "prompt", e.NAME = "name", e.COLOR = "color", e.FACE = "face", e.SIZE = "size";
  })(We || (We = {}));
  var de;
  (function (e) {
    e.NO_QUIRKS = "no-quirks", e.QUIRKS = "quirks", e.LIMITED_QUIRKS = "limited-quirks";
  })(de || (de = {}));
  var d;
  (function (e) {
    e.A = "a", e.ADDRESS = "address", e.ANNOTATION_XML = "annotation-xml", e.APPLET = "applet", e.AREA = "area", e.ARTICLE = "article", e.ASIDE = "aside", e.B = "b", e.BASE = "base", e.BASEFONT = "basefont", e.BGSOUND = "bgsound", e.BIG = "big", e.BLOCKQUOTE = "blockquote", e.BODY = "body", e.BR = "br", e.BUTTON = "button", e.CAPTION = "caption", e.CENTER = "center", e.CODE = "code", e.COL = "col", e.COLGROUP = "colgroup", e.DD = "dd", e.DESC = "desc", e.DETAILS = "details", e.DIALOG = "dialog", e.DIR = "dir", e.DIV = "div", e.DL = "dl", e.DT = "dt", e.EM = "em", e.EMBED = "embed", e.FIELDSET = "fieldset", e.FIGCAPTION = "figcaption", e.FIGURE = "figure", e.FONT = "font", e.FOOTER = "footer", e.FOREIGN_OBJECT = "foreignObject", e.FORM = "form", e.FRAME = "frame", e.FRAMESET = "frameset", e.H1 = "h1", e.H2 = "h2", e.H3 = "h3", e.H4 = "h4", e.H5 = "h5", e.H6 = "h6", e.HEAD = "head", e.HEADER = "header", e.HGROUP = "hgroup", e.HR = "hr", e.HTML = "html", e.I = "i", e.IMG = "img", e.IMAGE = "image", e.INPUT = "input", e.IFRAME = "iframe", e.KEYGEN = "keygen", e.LABEL = "label", e.LI = "li", e.LINK = "link", e.LISTING = "listing", e.MAIN = "main", e.MALIGNMARK = "malignmark", e.MARQUEE = "marquee", e.MATH = "math", e.MENU = "menu", e.META = "meta", e.MGLYPH = "mglyph", e.MI = "mi", e.MO = "mo", e.MN = "mn", e.MS = "ms", e.MTEXT = "mtext", e.NAV = "nav", e.NOBR = "nobr", e.NOFRAMES = "noframes", e.NOEMBED = "noembed", e.NOSCRIPT = "noscript", e.OBJECT = "object", e.OL = "ol", e.OPTGROUP = "optgroup", e.OPTION = "option", e.P = "p", e.PARAM = "param", e.PLAINTEXT = "plaintext", e.PRE = "pre", e.RB = "rb", e.RP = "rp", e.RT = "rt", e.RTC = "rtc", e.RUBY = "ruby", e.S = "s", e.SCRIPT = "script", e.SEARCH = "search", e.SECTION = "section", e.SELECT = "select", e.SOURCE = "source", e.SMALL = "small", e.SPAN = "span", e.STRIKE = "strike", e.STRONG = "strong", e.STYLE = "style", e.SUB = "sub", e.SUMMARY = "summary", e.SUP = "sup", e.TABLE = "table", e.TBODY = "tbody", e.TEMPLATE = "template", e.TEXTAREA = "textarea", e.TFOOT = "tfoot", e.TD = "td", e.TH = "th", e.THEAD = "thead", e.TITLE = "title", e.TR = "tr", e.TRACK = "track", e.TT = "tt", e.U = "u", e.UL = "ul", e.SVG = "svg", e.VAR = "var", e.WBR = "wbr", e.XMP = "xmp";
  })(d || (d = {}));
  var u;
  (function (e) {
    e[e.UNKNOWN = 0] = "UNKNOWN", e[e.A = 1] = "A", e[e.ADDRESS = 2] = "ADDRESS", e[e.ANNOTATION_XML = 3] = "ANNOTATION_XML", e[e.APPLET = 4] = "APPLET", e[e.AREA = 5] = "AREA", e[e.ARTICLE = 6] = "ARTICLE", e[e.ASIDE = 7] = "ASIDE", e[e.B = 8] = "B", e[e.BASE = 9] = "BASE", e[e.BASEFONT = 10] = "BASEFONT", e[e.BGSOUND = 11] = "BGSOUND", e[e.BIG = 12] = "BIG", e[e.BLOCKQUOTE = 13] = "BLOCKQUOTE", e[e.BODY = 14] = "BODY", e[e.BR = 15] = "BR", e[e.BUTTON = 16] = "BUTTON", e[e.CAPTION = 17] = "CAPTION", e[e.CENTER = 18] = "CENTER", e[e.CODE = 19] = "CODE", e[e.COL = 20] = "COL", e[e.COLGROUP = 21] = "COLGROUP", e[e.DD = 22] = "DD", e[e.DESC = 23] = "DESC", e[e.DETAILS = 24] = "DETAILS", e[e.DIALOG = 25] = "DIALOG", e[e.DIR = 26] = "DIR", e[e.DIV = 27] = "DIV", e[e.DL = 28] = "DL", e[e.DT = 29] = "DT", e[e.EM = 30] = "EM", e[e.EMBED = 31] = "EMBED", e[e.FIELDSET = 32] = "FIELDSET", e[e.FIGCAPTION = 33] = "FIGCAPTION", e[e.FIGURE = 34] = "FIGURE", e[e.FONT = 35] = "FONT", e[e.FOOTER = 36] = "FOOTER", e[e.FOREIGN_OBJECT = 37] = "FOREIGN_OBJECT", e[e.FORM = 38] = "FORM", e[e.FRAME = 39] = "FRAME", e[e.FRAMESET = 40] = "FRAMESET", e[e.H1 = 41] = "H1", e[e.H2 = 42] = "H2", e[e.H3 = 43] = "H3", e[e.H4 = 44] = "H4", e[e.H5 = 45] = "H5", e[e.H6 = 46] = "H6", e[e.HEAD = 47] = "HEAD", e[e.HEADER = 48] = "HEADER", e[e.HGROUP = 49] = "HGROUP", e[e.HR = 50] = "HR", e[e.HTML = 51] = "HTML", e[e.I = 52] = "I", e[e.IMG = 53] = "IMG", e[e.IMAGE = 54] = "IMAGE", e[e.INPUT = 55] = "INPUT", e[e.IFRAME = 56] = "IFRAME", e[e.KEYGEN = 57] = "KEYGEN", e[e.LABEL = 58] = "LABEL", e[e.LI = 59] = "LI", e[e.LINK = 60] = "LINK", e[e.LISTING = 61] = "LISTING", e[e.MAIN = 62] = "MAIN", e[e.MALIGNMARK = 63] = "MALIGNMARK", e[e.MARQUEE = 64] = "MARQUEE", e[e.MATH = 65] = "MATH", e[e.MENU = 66] = "MENU", e[e.META = 67] = "META", e[e.MGLYPH = 68] = "MGLYPH", e[e.MI = 69] = "MI", e[e.MO = 70] = "MO", e[e.MN = 71] = "MN", e[e.MS = 72] = "MS", e[e.MTEXT = 73] = "MTEXT", e[e.NAV = 74] = "NAV", e[e.NOBR = 75] = "NOBR", e[e.NOFRAMES = 76] = "NOFRAMES", e[e.NOEMBED = 77] = "NOEMBED", e[e.NOSCRIPT = 78] = "NOSCRIPT", e[e.OBJECT = 79] = "OBJECT", e[e.OL = 80] = "OL", e[e.OPTGROUP = 81] = "OPTGROUP", e[e.OPTION = 82] = "OPTION", e[e.P = 83] = "P", e[e.PARAM = 84] = "PARAM", e[e.PLAINTEXT = 85] = "PLAINTEXT", e[e.PRE = 86] = "PRE", e[e.RB = 87] = "RB", e[e.RP = 88] = "RP", e[e.RT = 89] = "RT", e[e.RTC = 90] = "RTC", e[e.RUBY = 91] = "RUBY", e[e.S = 92] = "S", e[e.SCRIPT = 93] = "SCRIPT", e[e.SEARCH = 94] = "SEARCH", e[e.SECTION = 95] = "SECTION", e[e.SELECT = 96] = "SELECT", e[e.SOURCE = 97] = "SOURCE", e[e.SMALL = 98] = "SMALL", e[e.SPAN = 99] = "SPAN", e[e.STRIKE = 100] = "STRIKE", e[e.STRONG = 101] = "STRONG", e[e.STYLE = 102] = "STYLE", e[e.SUB = 103] = "SUB", e[e.SUMMARY = 104] = "SUMMARY", e[e.SUP = 105] = "SUP", e[e.TABLE = 106] = "TABLE", e[e.TBODY = 107] = "TBODY", e[e.TEMPLATE = 108] = "TEMPLATE", e[e.TEXTAREA = 109] = "TEXTAREA", e[e.TFOOT = 110] = "TFOOT", e[e.TD = 111] = "TD", e[e.TH = 112] = "TH", e[e.THEAD = 113] = "THEAD", e[e.TITLE = 114] = "TITLE", e[e.TR = 115] = "TR", e[e.TRACK = 116] = "TRACK", e[e.TT = 117] = "TT", e[e.U = 118] = "U", e[e.UL = 119] = "UL", e[e.SVG = 120] = "SVG", e[e.VAR = 121] = "VAR", e[e.WBR = 122] = "WBR", e[e.XMP = 123] = "XMP";
  })(u || (u = {}));
  var vo = new Map([[d.A, u.A], [d.ADDRESS, u.ADDRESS], [d.ANNOTATION_XML, u.ANNOTATION_XML], [d.APPLET, u.APPLET], [d.AREA, u.AREA], [d.ARTICLE, u.ARTICLE], [d.ASIDE, u.ASIDE], [d.B, u.B], [d.BASE, u.BASE], [d.BASEFONT, u.BASEFONT], [d.BGSOUND, u.BGSOUND], [d.BIG, u.BIG], [d.BLOCKQUOTE, u.BLOCKQUOTE], [d.BODY, u.BODY], [d.BR, u.BR], [d.BUTTON, u.BUTTON], [d.CAPTION, u.CAPTION], [d.CENTER, u.CENTER], [d.CODE, u.CODE], [d.COL, u.COL], [d.COLGROUP, u.COLGROUP], [d.DD, u.DD], [d.DESC, u.DESC], [d.DETAILS, u.DETAILS], [d.DIALOG, u.DIALOG], [d.DIR, u.DIR], [d.DIV, u.DIV], [d.DL, u.DL], [d.DT, u.DT], [d.EM, u.EM], [d.EMBED, u.EMBED], [d.FIELDSET, u.FIELDSET], [d.FIGCAPTION, u.FIGCAPTION], [d.FIGURE, u.FIGURE], [d.FONT, u.FONT], [d.FOOTER, u.FOOTER], [d.FOREIGN_OBJECT, u.FOREIGN_OBJECT], [d.FORM, u.FORM], [d.FRAME, u.FRAME], [d.FRAMESET, u.FRAMESET], [d.H1, u.H1], [d.H2, u.H2], [d.H3, u.H3], [d.H4, u.H4], [d.H5, u.H5], [d.H6, u.H6], [d.HEAD, u.HEAD], [d.HEADER, u.HEADER], [d.HGROUP, u.HGROUP], [d.HR, u.HR], [d.HTML, u.HTML], [d.I, u.I], [d.IMG, u.IMG], [d.IMAGE, u.IMAGE], [d.INPUT, u.INPUT], [d.IFRAME, u.IFRAME], [d.KEYGEN, u.KEYGEN], [d.LABEL, u.LABEL], [d.LI, u.LI], [d.LINK, u.LINK], [d.LISTING, u.LISTING], [d.MAIN, u.MAIN], [d.MALIGNMARK, u.MALIGNMARK], [d.MARQUEE, u.MARQUEE], [d.MATH, u.MATH], [d.MENU, u.MENU], [d.META, u.META], [d.MGLYPH, u.MGLYPH], [d.MI, u.MI], [d.MO, u.MO], [d.MN, u.MN], [d.MS, u.MS], [d.MTEXT, u.MTEXT], [d.NAV, u.NAV], [d.NOBR, u.NOBR], [d.NOFRAMES, u.NOFRAMES], [d.NOEMBED, u.NOEMBED], [d.NOSCRIPT, u.NOSCRIPT], [d.OBJECT, u.OBJECT], [d.OL, u.OL], [d.OPTGROUP, u.OPTGROUP], [d.OPTION, u.OPTION], [d.P, u.P], [d.PARAM, u.PARAM], [d.PLAINTEXT, u.PLAINTEXT], [d.PRE, u.PRE], [d.RB, u.RB], [d.RP, u.RP], [d.RT, u.RT], [d.RTC, u.RTC], [d.RUBY, u.RUBY], [d.S, u.S], [d.SCRIPT, u.SCRIPT], [d.SEARCH, u.SEARCH], [d.SECTION, u.SECTION], [d.SELECT, u.SELECT], [d.SOURCE, u.SOURCE], [d.SMALL, u.SMALL], [d.SPAN, u.SPAN], [d.STRIKE, u.STRIKE], [d.STRONG, u.STRONG], [d.STYLE, u.STYLE], [d.SUB, u.SUB], [d.SUMMARY, u.SUMMARY], [d.SUP, u.SUP], [d.TABLE, u.TABLE], [d.TBODY, u.TBODY], [d.TEMPLATE, u.TEMPLATE], [d.TEXTAREA, u.TEXTAREA], [d.TFOOT, u.TFOOT], [d.TD, u.TD], [d.TH, u.TH], [d.THEAD, u.THEAD], [d.TITLE, u.TITLE], [d.TR, u.TR], [d.TRACK, u.TRACK], [d.TT, u.TT], [d.U, u.U], [d.UL, u.UL], [d.SVG, u.SVG], [d.VAR, u.VAR], [d.WBR, u.WBR], [d.XMP, u.XMP]]);
  function _t(e) {
    var t;
    return (t = vo.get(e)) !== null && t !== void 0 ? t : u.UNKNOWN;
  }
  var B = u,
    ru = {
      [p.HTML]: new Set([B.ADDRESS, B.APPLET, B.AREA, B.ARTICLE, B.ASIDE, B.BASE, B.BASEFONT, B.BGSOUND, B.BLOCKQUOTE, B.BODY, B.BR, B.BUTTON, B.CAPTION, B.CENTER, B.COL, B.COLGROUP, B.DD, B.DETAILS, B.DIR, B.DIV, B.DL, B.DT, B.EMBED, B.FIELDSET, B.FIGCAPTION, B.FIGURE, B.FOOTER, B.FORM, B.FRAME, B.FRAMESET, B.H1, B.H2, B.H3, B.H4, B.H5, B.H6, B.HEAD, B.HEADER, B.HGROUP, B.HR, B.HTML, B.IFRAME, B.IMG, B.INPUT, B.LI, B.LINK, B.LISTING, B.MAIN, B.MARQUEE, B.MENU, B.META, B.NAV, B.NOEMBED, B.NOFRAMES, B.NOSCRIPT, B.OBJECT, B.OL, B.P, B.PARAM, B.PLAINTEXT, B.PRE, B.SCRIPT, B.SECTION, B.SELECT, B.SOURCE, B.STYLE, B.SUMMARY, B.TABLE, B.TBODY, B.TD, B.TEMPLATE, B.TEXTAREA, B.TFOOT, B.TH, B.THEAD, B.TITLE, B.TR, B.TRACK, B.UL, B.WBR, B.XMP]),
      [p.MATHML]: new Set([B.MI, B.MO, B.MN, B.MS, B.MTEXT, B.ANNOTATION_XML]),
      [p.SVG]: new Set([B.TITLE, B.FOREIGN_OBJECT, B.DESC]),
      [p.XLINK]: new Set(),
      [p.XML]: new Set(),
      [p.XMLNS]: new Set()
    },
    nn = new Set([B.H1, B.H2, B.H3, B.H4, B.H5, B.H6]),
    No = new Set([d.STYLE, d.SCRIPT, d.XMP, d.IFRAME, d.NOEMBED, d.NOFRAMES, d.PLAINTEXT]);
  function uu(e, t) {
    return No.has(e) || t && e === d.NOSCRIPT;
  }
  var A;
  (function (e) {
    e[e.DATA = 0] = "DATA", e[e.RCDATA = 1] = "RCDATA", e[e.RAWTEXT = 2] = "RAWTEXT", e[e.SCRIPT_DATA = 3] = "SCRIPT_DATA", e[e.PLAINTEXT = 4] = "PLAINTEXT", e[e.TAG_OPEN = 5] = "TAG_OPEN", e[e.END_TAG_OPEN = 6] = "END_TAG_OPEN", e[e.TAG_NAME = 7] = "TAG_NAME", e[e.RCDATA_LESS_THAN_SIGN = 8] = "RCDATA_LESS_THAN_SIGN", e[e.RCDATA_END_TAG_OPEN = 9] = "RCDATA_END_TAG_OPEN", e[e.RCDATA_END_TAG_NAME = 10] = "RCDATA_END_TAG_NAME", e[e.RAWTEXT_LESS_THAN_SIGN = 11] = "RAWTEXT_LESS_THAN_SIGN", e[e.RAWTEXT_END_TAG_OPEN = 12] = "RAWTEXT_END_TAG_OPEN", e[e.RAWTEXT_END_TAG_NAME = 13] = "RAWTEXT_END_TAG_NAME", e[e.SCRIPT_DATA_LESS_THAN_SIGN = 14] = "SCRIPT_DATA_LESS_THAN_SIGN", e[e.SCRIPT_DATA_END_TAG_OPEN = 15] = "SCRIPT_DATA_END_TAG_OPEN", e[e.SCRIPT_DATA_END_TAG_NAME = 16] = "SCRIPT_DATA_END_TAG_NAME", e[e.SCRIPT_DATA_ESCAPE_START = 17] = "SCRIPT_DATA_ESCAPE_START", e[e.SCRIPT_DATA_ESCAPE_START_DASH = 18] = "SCRIPT_DATA_ESCAPE_START_DASH", e[e.SCRIPT_DATA_ESCAPED = 19] = "SCRIPT_DATA_ESCAPED", e[e.SCRIPT_DATA_ESCAPED_DASH = 20] = "SCRIPT_DATA_ESCAPED_DASH", e[e.SCRIPT_DATA_ESCAPED_DASH_DASH = 21] = "SCRIPT_DATA_ESCAPED_DASH_DASH", e[e.SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN = 22] = "SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN", e[e.SCRIPT_DATA_ESCAPED_END_TAG_OPEN = 23] = "SCRIPT_DATA_ESCAPED_END_TAG_OPEN", e[e.SCRIPT_DATA_ESCAPED_END_TAG_NAME = 24] = "SCRIPT_DATA_ESCAPED_END_TAG_NAME", e[e.SCRIPT_DATA_DOUBLE_ESCAPE_START = 25] = "SCRIPT_DATA_DOUBLE_ESCAPE_START", e[e.SCRIPT_DATA_DOUBLE_ESCAPED = 26] = "SCRIPT_DATA_DOUBLE_ESCAPED", e[e.SCRIPT_DATA_DOUBLE_ESCAPED_DASH = 27] = "SCRIPT_DATA_DOUBLE_ESCAPED_DASH", e[e.SCRIPT_DATA_DOUBLE_ESCAPED_DASH_DASH = 28] = "SCRIPT_DATA_DOUBLE_ESCAPED_DASH_DASH", e[e.SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN = 29] = "SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN", e[e.SCRIPT_DATA_DOUBLE_ESCAPE_END = 30] = "SCRIPT_DATA_DOUBLE_ESCAPE_END", e[e.BEFORE_ATTRIBUTE_NAME = 31] = "BEFORE_ATTRIBUTE_NAME", e[e.ATTRIBUTE_NAME = 32] = "ATTRIBUTE_NAME", e[e.AFTER_ATTRIBUTE_NAME = 33] = "AFTER_ATTRIBUTE_NAME", e[e.BEFORE_ATTRIBUTE_VALUE = 34] = "BEFORE_ATTRIBUTE_VALUE", e[e.ATTRIBUTE_VALUE_DOUBLE_QUOTED = 35] = "ATTRIBUTE_VALUE_DOUBLE_QUOTED", e[e.ATTRIBUTE_VALUE_SINGLE_QUOTED = 36] = "ATTRIBUTE_VALUE_SINGLE_QUOTED", e[e.ATTRIBUTE_VALUE_UNQUOTED = 37] = "ATTRIBUTE_VALUE_UNQUOTED", e[e.AFTER_ATTRIBUTE_VALUE_QUOTED = 38] = "AFTER_ATTRIBUTE_VALUE_QUOTED", e[e.SELF_CLOSING_START_TAG = 39] = "SELF_CLOSING_START_TAG", e[e.BOGUS_COMMENT = 40] = "BOGUS_COMMENT", e[e.MARKUP_DECLARATION_OPEN = 41] = "MARKUP_DECLARATION_OPEN", e[e.COMMENT_START = 42] = "COMMENT_START", e[e.COMMENT_START_DASH = 43] = "COMMENT_START_DASH", e[e.COMMENT = 44] = "COMMENT", e[e.COMMENT_LESS_THAN_SIGN = 45] = "COMMENT_LESS_THAN_SIGN", e[e.COMMENT_LESS_THAN_SIGN_BANG = 46] = "COMMENT_LESS_THAN_SIGN_BANG", e[e.COMMENT_LESS_THAN_SIGN_BANG_DASH = 47] = "COMMENT_LESS_THAN_SIGN_BANG_DASH", e[e.COMMENT_LESS_THAN_SIGN_BANG_DASH_DASH = 48] = "COMMENT_LESS_THAN_SIGN_BANG_DASH_DASH", e[e.COMMENT_END_DASH = 49] = "COMMENT_END_DASH", e[e.COMMENT_END = 50] = "COMMENT_END", e[e.COMMENT_END_BANG = 51] = "COMMENT_END_BANG", e[e.DOCTYPE = 52] = "DOCTYPE", e[e.BEFORE_DOCTYPE_NAME = 53] = "BEFORE_DOCTYPE_NAME", e[e.DOCTYPE_NAME = 54] = "DOCTYPE_NAME", e[e.AFTER_DOCTYPE_NAME = 55] = "AFTER_DOCTYPE_NAME", e[e.AFTER_DOCTYPE_PUBLIC_KEYWORD = 56] = "AFTER_DOCTYPE_PUBLIC_KEYWORD", e[e.BEFORE_DOCTYPE_PUBLIC_IDENTIFIER = 57] = "BEFORE_DOCTYPE_PUBLIC_IDENTIFIER", e[e.DOCTYPE_PUBLIC_IDENTIFIER_DOUBLE_QUOTED = 58] = "DOCTYPE_PUBLIC_IDENTIFIER_DOUBLE_QUOTED", e[e.DOCTYPE_PUBLIC_IDENTIFIER_SINGLE_QUOTED = 59] = "DOCTYPE_PUBLIC_IDENTIFIER_SINGLE_QUOTED", e[e.AFTER_DOCTYPE_PUBLIC_IDENTIFIER = 60] = "AFTER_DOCTYPE_PUBLIC_IDENTIFIER", e[e.BETWEEN_DOCTYPE_PUBLIC_AND_SYSTEM_IDENTIFIERS = 61] = "BETWEEN_DOCTYPE_PUBLIC_AND_SYSTEM_IDENTIFIERS", e[e.AFTER_DOCTYPE_SYSTEM_KEYWORD = 62] = "AFTER_DOCTYPE_SYSTEM_KEYWORD", e[e.BEFORE_DOCTYPE_SYSTEM_IDENTIFIER = 63] = "BEFORE_DOCTYPE_SYSTEM_IDENTIFIER", e[e.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED = 64] = "DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED", e[e.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED = 65] = "DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED", e[e.AFTER_DOCTYPE_SYSTEM_IDENTIFIER = 66] = "AFTER_DOCTYPE_SYSTEM_IDENTIFIER", e[e.BOGUS_DOCTYPE = 67] = "BOGUS_DOCTYPE", e[e.CDATA_SECTION = 68] = "CDATA_SECTION", e[e.CDATA_SECTION_BRACKET = 69] = "CDATA_SECTION_BRACKET", e[e.CDATA_SECTION_END = 70] = "CDATA_SECTION_END", e[e.CHARACTER_REFERENCE = 71] = "CHARACTER_REFERENCE", e[e.AMBIGUOUS_AMPERSAND = 72] = "AMBIGUOUS_AMPERSAND";
  })(A || (A = {}));
  var Ae = {
    DATA: A.DATA,
    RCDATA: A.RCDATA,
    RAWTEXT: A.RAWTEXT,
    SCRIPT_DATA: A.SCRIPT_DATA,
    PLAINTEXT: A.PLAINTEXT,
    CDATA_SECTION: A.CDATA_SECTION
  };
  function ko(e) {
    return e >= o.DIGIT_0 && e <= o.DIGIT_9;
  }
  function yt(e) {
    return e >= o.LATIN_CAPITAL_A && e <= o.LATIN_CAPITAL_Z;
  }
  function wo(e) {
    return e >= o.LATIN_SMALL_A && e <= o.LATIN_SMALL_Z;
  }
  function Ge(e) {
    return wo(e) || yt(e);
  }
  function au(e) {
    return Ge(e) || ko(e);
  }
  function rn(e) {
    return e + 32;
  }
  function iu(e) {
    return e === o.SPACE || e === o.LINE_FEED || e === o.TABULATION || e === o.FORM_FEED;
  }
  function su(e) {
    return iu(e) || e === o.SOLIDUS || e === o.GREATER_THAN_SIGN;
  }
  function Ro(e) {
    return e === o.NULL ? C.nullCharacterReference : e > 1114111 ? C.characterReferenceOutsideUnicodeRange : $r(e) ? C.surrogateCharacterReference : tu(e) ? C.noncharacterCharacterReference : eu(e) || e === o.CARRIAGE_RETURN ? C.controlCharacterReference : null;
  }
  var Oo = class {
      constructor(e, t) {
        this.options = e, this.handler = t, this.paused = !1, this.inLoop = !1, this.inForeignNode = !1, this.lastStartTagName = "", this.active = !1, this.state = A.DATA, this.returnState = A.DATA, this.entityStartPos = 0, this.consumedAfterSnapshot = -1, this.currentCharacterToken = null, this.currentToken = null, this.currentAttr = {
          name: "",
          value: ""
        }, this.preprocessor = new Bo(t), this.currentLocation = this.getCurrentLocation(-1), this.entityDecoder = new So(Do, (n, r) => {
          this.preprocessor.pos = this.entityStartPos + r - 1, this._flushCodePointConsumedAsCharacterReference(n);
        }, t.onParseError ? {
          missingSemicolonAfterCharacterReference: () => {
            this._err(C.missingSemicolonAfterCharacterReference, 1);
          },
          absenceOfDigitsInNumericCharacterReference: n => {
            this._err(C.absenceOfDigitsInNumericCharacterReference, this.entityStartPos - this.preprocessor.pos + n);
          },
          validateNumericCharacterReference: n => {
            let r = Ro(n);
            r && this._err(r, 1);
          }
        } : void 0);
      }
      _err(e, t = 0) {
        var n, r;
        (r = (n = this.handler).onParseError) === null || r === void 0 || r.call(n, this.preprocessor.getError(e, t));
      }
      getCurrentLocation(e) {
        return this.options.sourceCodeLocationInfo ? {
          startLine: this.preprocessor.line,
          startCol: this.preprocessor.col - e,
          startOffset: this.preprocessor.offset - e,
          endLine: -1,
          endCol: -1,
          endOffset: -1
        } : null;
      }
      _runParsingLoop() {
        if (!this.inLoop) {
          for (this.inLoop = !0; this.active && !this.paused;) {
            this.consumedAfterSnapshot = 0;
            let e = this._consume();
            this._ensureHibernation() || this._callState(e);
          }
          this.inLoop = !1;
        }
      }
      pause() {
        this.paused = !0;
      }
      resume(e) {
        if (!this.paused) throw new Error("Parser was already resumed");
        this.paused = !1, !this.inLoop && (this._runParsingLoop(), this.paused || e == null || e());
      }
      write(e, t, n) {
        this.active = !0, this.preprocessor.write(e, t), this._runParsingLoop(), this.paused || n == null || n();
      }
      insertHtmlAtCurrentPos(e) {
        this.active = !0, this.preprocessor.insertHtmlAtCurrentPos(e), this._runParsingLoop();
      }
      _ensureHibernation() {
        return this.preprocessor.endOfChunkHit ? (this.preprocessor.retreat(this.consumedAfterSnapshot), this.consumedAfterSnapshot = 0, this.active = !1, !0) : !1;
      }
      _consume() {
        return this.consumedAfterSnapshot++, this.preprocessor.advance();
      }
      _advanceBy(e) {
        this.consumedAfterSnapshot += e;
        for (let t = 0; t < e; t++) this.preprocessor.advance();
      }
      _consumeSequenceIfMatch(e, t) {
        return this.preprocessor.startsWith(e, t) ? (this._advanceBy(e.length - 1), !0) : !1;
      }
      _createStartTagToken() {
        this.currentToken = {
          type: v.START_TAG,
          tagName: "",
          tagID: u.UNKNOWN,
          selfClosing: !1,
          ackSelfClosing: !1,
          attrs: [],
          location: this.getCurrentLocation(1)
        };
      }
      _createEndTagToken() {
        this.currentToken = {
          type: v.END_TAG,
          tagName: "",
          tagID: u.UNKNOWN,
          selfClosing: !1,
          ackSelfClosing: !1,
          attrs: [],
          location: this.getCurrentLocation(2)
        };
      }
      _createCommentToken(e) {
        this.currentToken = {
          type: v.COMMENT,
          data: "",
          location: this.getCurrentLocation(e)
        };
      }
      _createDoctypeToken(e) {
        this.currentToken = {
          type: v.DOCTYPE,
          name: e,
          forceQuirks: !1,
          publicId: null,
          systemId: null,
          location: this.currentLocation
        };
      }
      _createCharacterToken(e, t) {
        this.currentCharacterToken = {
          type: e,
          chars: t,
          location: this.currentLocation
        };
      }
      _createAttr(e) {
        this.currentAttr = {
          name: e,
          value: ""
        }, this.currentLocation = this.getCurrentLocation(0);
      }
      _leaveAttrName() {
        var e, t;
        let n = this.currentToken;
        if (A0(n, this.currentAttr.name) === null) {
          if (n.attrs.push(this.currentAttr), n.location && this.currentLocation) {
            let r = (e = (t = n.location).attrs) !== null && e !== void 0 ? e : t.attrs = Object.create(null);
            r[this.currentAttr.name] = this.currentLocation, this._leaveAttrValue();
          }
        } else this._err(C.duplicateAttribute);
      }
      _leaveAttrValue() {
        this.currentLocation && (this.currentLocation.endLine = this.preprocessor.line, this.currentLocation.endCol = this.preprocessor.col, this.currentLocation.endOffset = this.preprocessor.offset);
      }
      prepareToken(e) {
        this._emitCurrentCharacterToken(e.location), this.currentToken = null, e.location && (e.location.endLine = this.preprocessor.line, e.location.endCol = this.preprocessor.col + 1, e.location.endOffset = this.preprocessor.offset + 1), this.currentLocation = this.getCurrentLocation(-1);
      }
      emitCurrentTagToken() {
        let e = this.currentToken;
        this.prepareToken(e), e.tagID = _t(e.tagName), e.type === v.START_TAG ? (this.lastStartTagName = e.tagName, this.handler.onStartTag(e)) : (e.attrs.length > 0 && this._err(C.endTagWithAttributes), e.selfClosing && this._err(C.endTagWithTrailingSolidus), this.handler.onEndTag(e)), this.preprocessor.dropParsedChunk();
      }
      emitCurrentComment(e) {
        this.prepareToken(e), this.handler.onComment(e), this.preprocessor.dropParsedChunk();
      }
      emitCurrentDoctype(e) {
        this.prepareToken(e), this.handler.onDoctype(e), this.preprocessor.dropParsedChunk();
      }
      _emitCurrentCharacterToken(e) {
        if (this.currentCharacterToken) {
          switch (e && this.currentCharacterToken.location && (this.currentCharacterToken.location.endLine = e.startLine, this.currentCharacterToken.location.endCol = e.startCol, this.currentCharacterToken.location.endOffset = e.startOffset), this.currentCharacterToken.type) {
            case v.CHARACTER:
              {
                this.handler.onCharacter(this.currentCharacterToken);
                break;
              }
            case v.NULL_CHARACTER:
              {
                this.handler.onNullCharacter(this.currentCharacterToken);
                break;
              }
            case v.WHITESPACE_CHARACTER:
              {
                this.handler.onWhitespaceCharacter(this.currentCharacterToken);
                break;
              }
          }
          this.currentCharacterToken = null;
        }
      }
      _emitEOFToken() {
        let e = this.getCurrentLocation(0);
        e && (e.endLine = e.startLine, e.endCol = e.startCol, e.endOffset = e.startOffset), this._emitCurrentCharacterToken(e), this.handler.onEof({
          type: v.EOF,
          location: e
        }), this.active = !1;
      }
      _appendCharToCurrentCharacterToken(e, t) {
        if (this.currentCharacterToken) if (this.currentCharacterToken.type === e) {
          this.currentCharacterToken.chars += t;
          return;
        } else this.currentLocation = this.getCurrentLocation(0), this._emitCurrentCharacterToken(this.currentLocation), this.preprocessor.dropParsedChunk();
        this._createCharacterToken(e, t);
      }
      _emitCodePoint(e) {
        let t = iu(e) ? v.WHITESPACE_CHARACTER : e === o.NULL ? v.NULL_CHARACTER : v.CHARACTER;
        this._appendCharToCurrentCharacterToken(t, String.fromCodePoint(e));
      }
      _emitChars(e) {
        this._appendCharToCurrentCharacterToken(v.CHARACTER, e);
      }
      _startCharacterReference() {
        this.returnState = this.state, this.state = A.CHARACTER_REFERENCE, this.entityStartPos = this.preprocessor.pos, this.entityDecoder.startEntity(this._isCharacterReferenceInAttribute() ? Ie.Attribute : Ie.Legacy);
      }
      _isCharacterReferenceInAttribute() {
        return this.returnState === A.ATTRIBUTE_VALUE_DOUBLE_QUOTED || this.returnState === A.ATTRIBUTE_VALUE_SINGLE_QUOTED || this.returnState === A.ATTRIBUTE_VALUE_UNQUOTED;
      }
      _flushCodePointConsumedAsCharacterReference(e) {
        this._isCharacterReferenceInAttribute() ? this.currentAttr.value += String.fromCodePoint(e) : this._emitCodePoint(e);
      }
      _callState(e) {
        switch (this.state) {
          case A.DATA:
            {
              this._stateData(e);
              break;
            }
          case A.RCDATA:
            {
              this._stateRcdata(e);
              break;
            }
          case A.RAWTEXT:
            {
              this._stateRawtext(e);
              break;
            }
          case A.SCRIPT_DATA:
            {
              this._stateScriptData(e);
              break;
            }
          case A.PLAINTEXT:
            {
              this._statePlaintext(e);
              break;
            }
          case A.TAG_OPEN:
            {
              this._stateTagOpen(e);
              break;
            }
          case A.END_TAG_OPEN:
            {
              this._stateEndTagOpen(e);
              break;
            }
          case A.TAG_NAME:
            {
              this._stateTagName(e);
              break;
            }
          case A.RCDATA_LESS_THAN_SIGN:
            {
              this._stateRcdataLessThanSign(e);
              break;
            }
          case A.RCDATA_END_TAG_OPEN:
            {
              this._stateRcdataEndTagOpen(e);
              break;
            }
          case A.RCDATA_END_TAG_NAME:
            {
              this._stateRcdataEndTagName(e);
              break;
            }
          case A.RAWTEXT_LESS_THAN_SIGN:
            {
              this._stateRawtextLessThanSign(e);
              break;
            }
          case A.RAWTEXT_END_TAG_OPEN:
            {
              this._stateRawtextEndTagOpen(e);
              break;
            }
          case A.RAWTEXT_END_TAG_NAME:
            {
              this._stateRawtextEndTagName(e);
              break;
            }
          case A.SCRIPT_DATA_LESS_THAN_SIGN:
            {
              this._stateScriptDataLessThanSign(e);
              break;
            }
          case A.SCRIPT_DATA_END_TAG_OPEN:
            {
              this._stateScriptDataEndTagOpen(e);
              break;
            }
          case A.SCRIPT_DATA_END_TAG_NAME:
            {
              this._stateScriptDataEndTagName(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPE_START:
            {
              this._stateScriptDataEscapeStart(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPE_START_DASH:
            {
              this._stateScriptDataEscapeStartDash(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED:
            {
              this._stateScriptDataEscaped(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED_DASH:
            {
              this._stateScriptDataEscapedDash(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED_DASH_DASH:
            {
              this._stateScriptDataEscapedDashDash(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN:
            {
              this._stateScriptDataEscapedLessThanSign(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED_END_TAG_OPEN:
            {
              this._stateScriptDataEscapedEndTagOpen(e);
              break;
            }
          case A.SCRIPT_DATA_ESCAPED_END_TAG_NAME:
            {
              this._stateScriptDataEscapedEndTagName(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPE_START:
            {
              this._stateScriptDataDoubleEscapeStart(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPED:
            {
              this._stateScriptDataDoubleEscaped(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPED_DASH:
            {
              this._stateScriptDataDoubleEscapedDash(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPED_DASH_DASH:
            {
              this._stateScriptDataDoubleEscapedDashDash(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN:
            {
              this._stateScriptDataDoubleEscapedLessThanSign(e);
              break;
            }
          case A.SCRIPT_DATA_DOUBLE_ESCAPE_END:
            {
              this._stateScriptDataDoubleEscapeEnd(e);
              break;
            }
          case A.BEFORE_ATTRIBUTE_NAME:
            {
              this._stateBeforeAttributeName(e);
              break;
            }
          case A.ATTRIBUTE_NAME:
            {
              this._stateAttributeName(e);
              break;
            }
          case A.AFTER_ATTRIBUTE_NAME:
            {
              this._stateAfterAttributeName(e);
              break;
            }
          case A.BEFORE_ATTRIBUTE_VALUE:
            {
              this._stateBeforeAttributeValue(e);
              break;
            }
          case A.ATTRIBUTE_VALUE_DOUBLE_QUOTED:
            {
              this._stateAttributeValueDoubleQuoted(e);
              break;
            }
          case A.ATTRIBUTE_VALUE_SINGLE_QUOTED:
            {
              this._stateAttributeValueSingleQuoted(e);
              break;
            }
          case A.ATTRIBUTE_VALUE_UNQUOTED:
            {
              this._stateAttributeValueUnquoted(e);
              break;
            }
          case A.AFTER_ATTRIBUTE_VALUE_QUOTED:
            {
              this._stateAfterAttributeValueQuoted(e);
              break;
            }
          case A.SELF_CLOSING_START_TAG:
            {
              this._stateSelfClosingStartTag(e);
              break;
            }
          case A.BOGUS_COMMENT:
            {
              this._stateBogusComment(e);
              break;
            }
          case A.MARKUP_DECLARATION_OPEN:
            {
              this._stateMarkupDeclarationOpen(e);
              break;
            }
          case A.COMMENT_START:
            {
              this._stateCommentStart(e);
              break;
            }
          case A.COMMENT_START_DASH:
            {
              this._stateCommentStartDash(e);
              break;
            }
          case A.COMMENT:
            {
              this._stateComment(e);
              break;
            }
          case A.COMMENT_LESS_THAN_SIGN:
            {
              this._stateCommentLessThanSign(e);
              break;
            }
          case A.COMMENT_LESS_THAN_SIGN_BANG:
            {
              this._stateCommentLessThanSignBang(e);
              break;
            }
          case A.COMMENT_LESS_THAN_SIGN_BANG_DASH:
            {
              this._stateCommentLessThanSignBangDash(e);
              break;
            }
          case A.COMMENT_LESS_THAN_SIGN_BANG_DASH_DASH:
            {
              this._stateCommentLessThanSignBangDashDash(e);
              break;
            }
          case A.COMMENT_END_DASH:
            {
              this._stateCommentEndDash(e);
              break;
            }
          case A.COMMENT_END:
            {
              this._stateCommentEnd(e);
              break;
            }
          case A.COMMENT_END_BANG:
            {
              this._stateCommentEndBang(e);
              break;
            }
          case A.DOCTYPE:
            {
              this._stateDoctype(e);
              break;
            }
          case A.BEFORE_DOCTYPE_NAME:
            {
              this._stateBeforeDoctypeName(e);
              break;
            }
          case A.DOCTYPE_NAME:
            {
              this._stateDoctypeName(e);
              break;
            }
          case A.AFTER_DOCTYPE_NAME:
            {
              this._stateAfterDoctypeName(e);
              break;
            }
          case A.AFTER_DOCTYPE_PUBLIC_KEYWORD:
            {
              this._stateAfterDoctypePublicKeyword(e);
              break;
            }
          case A.BEFORE_DOCTYPE_PUBLIC_IDENTIFIER:
            {
              this._stateBeforeDoctypePublicIdentifier(e);
              break;
            }
          case A.DOCTYPE_PUBLIC_IDENTIFIER_DOUBLE_QUOTED:
            {
              this._stateDoctypePublicIdentifierDoubleQuoted(e);
              break;
            }
          case A.DOCTYPE_PUBLIC_IDENTIFIER_SINGLE_QUOTED:
            {
              this._stateDoctypePublicIdentifierSingleQuoted(e);
              break;
            }
          case A.AFTER_DOCTYPE_PUBLIC_IDENTIFIER:
            {
              this._stateAfterDoctypePublicIdentifier(e);
              break;
            }
          case A.BETWEEN_DOCTYPE_PUBLIC_AND_SYSTEM_IDENTIFIERS:
            {
              this._stateBetweenDoctypePublicAndSystemIdentifiers(e);
              break;
            }
          case A.AFTER_DOCTYPE_SYSTEM_KEYWORD:
            {
              this._stateAfterDoctypeSystemKeyword(e);
              break;
            }
          case A.BEFORE_DOCTYPE_SYSTEM_IDENTIFIER:
            {
              this._stateBeforeDoctypeSystemIdentifier(e);
              break;
            }
          case A.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED:
            {
              this._stateDoctypeSystemIdentifierDoubleQuoted(e);
              break;
            }
          case A.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED:
            {
              this._stateDoctypeSystemIdentifierSingleQuoted(e);
              break;
            }
          case A.AFTER_DOCTYPE_SYSTEM_IDENTIFIER:
            {
              this._stateAfterDoctypeSystemIdentifier(e);
              break;
            }
          case A.BOGUS_DOCTYPE:
            {
              this._stateBogusDoctype(e);
              break;
            }
          case A.CDATA_SECTION:
            {
              this._stateCdataSection(e);
              break;
            }
          case A.CDATA_SECTION_BRACKET:
            {
              this._stateCdataSectionBracket(e);
              break;
            }
          case A.CDATA_SECTION_END:
            {
              this._stateCdataSectionEnd(e);
              break;
            }
          case A.CHARACTER_REFERENCE:
            {
              this._stateCharacterReference();
              break;
            }
          case A.AMBIGUOUS_AMPERSAND:
            {
              this._stateAmbiguousAmpersand(e);
              break;
            }
          default:
            throw new Error("Unknown state");
        }
      }
      _stateData(e) {
        switch (e) {
          case o.LESS_THAN_SIGN:
            {
              this.state = A.TAG_OPEN;
              break;
            }
          case o.AMPERSAND:
            {
              this._startCharacterReference();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitCodePoint(e);
              break;
            }
          case o.EOF:
            {
              this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateRcdata(e) {
        switch (e) {
          case o.AMPERSAND:
            {
              this._startCharacterReference();
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.RCDATA_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateRawtext(e) {
        switch (e) {
          case o.LESS_THAN_SIGN:
            {
              this.state = A.RAWTEXT_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateScriptData(e) {
        switch (e) {
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _statePlaintext(e) {
        switch (e) {
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateTagOpen(e) {
        if (Ge(e)) this._createStartTagToken(), this.state = A.TAG_NAME, this._stateTagName(e);else switch (e) {
          case o.EXCLAMATION_MARK:
            {
              this.state = A.MARKUP_DECLARATION_OPEN;
              break;
            }
          case o.SOLIDUS:
            {
              this.state = A.END_TAG_OPEN;
              break;
            }
          case o.QUESTION_MARK:
            {
              this._err(C.unexpectedQuestionMarkInsteadOfTagName), this._createCommentToken(1), this.state = A.BOGUS_COMMENT, this._stateBogusComment(e);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofBeforeTagName), this._emitChars("<"), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.invalidFirstCharacterOfTagName), this._emitChars("<"), this.state = A.DATA, this._stateData(e);
        }
      }
      _stateEndTagOpen(e) {
        if (Ge(e)) this._createEndTagToken(), this.state = A.TAG_NAME, this._stateTagName(e);else switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingEndTagName), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofBeforeTagName), this._emitChars("</"), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.invalidFirstCharacterOfTagName), this._createCommentToken(2), this.state = A.BOGUS_COMMENT, this._stateBogusComment(e);
        }
      }
      _stateTagName(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.BEFORE_ATTRIBUTE_NAME;
              break;
            }
          case o.SOLIDUS:
            {
              this.state = A.SELF_CLOSING_START_TAG;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.tagName += H;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            t.tagName += String.fromCodePoint(yt(e) ? rn(e) : e);
        }
      }
      _stateRcdataLessThanSign(e) {
        e === o.SOLIDUS ? this.state = A.RCDATA_END_TAG_OPEN : (this._emitChars("<"), this.state = A.RCDATA, this._stateRcdata(e));
      }
      _stateRcdataEndTagOpen(e) {
        Ge(e) ? (this.state = A.RCDATA_END_TAG_NAME, this._stateRcdataEndTagName(e)) : (this._emitChars("</"), this.state = A.RCDATA, this._stateRcdata(e));
      }
      handleSpecialEndTag(e) {
        if (!this.preprocessor.startsWith(this.lastStartTagName, !1)) return !this._ensureHibernation();
        this._createEndTagToken();
        let t = this.currentToken;
        switch (t.tagName = this.lastStartTagName, this.preprocessor.peek(this.lastStartTagName.length)) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            return this._advanceBy(this.lastStartTagName.length), this.state = A.BEFORE_ATTRIBUTE_NAME, !1;
          case o.SOLIDUS:
            return this._advanceBy(this.lastStartTagName.length), this.state = A.SELF_CLOSING_START_TAG, !1;
          case o.GREATER_THAN_SIGN:
            return this._advanceBy(this.lastStartTagName.length), this.emitCurrentTagToken(), this.state = A.DATA, !1;
          default:
            return !this._ensureHibernation();
        }
      }
      _stateRcdataEndTagName(e) {
        this.handleSpecialEndTag(e) && (this._emitChars("</"), this.state = A.RCDATA, this._stateRcdata(e));
      }
      _stateRawtextLessThanSign(e) {
        e === o.SOLIDUS ? this.state = A.RAWTEXT_END_TAG_OPEN : (this._emitChars("<"), this.state = A.RAWTEXT, this._stateRawtext(e));
      }
      _stateRawtextEndTagOpen(e) {
        Ge(e) ? (this.state = A.RAWTEXT_END_TAG_NAME, this._stateRawtextEndTagName(e)) : (this._emitChars("</"), this.state = A.RAWTEXT, this._stateRawtext(e));
      }
      _stateRawtextEndTagName(e) {
        this.handleSpecialEndTag(e) && (this._emitChars("</"), this.state = A.RAWTEXT, this._stateRawtext(e));
      }
      _stateScriptDataLessThanSign(e) {
        switch (e) {
          case o.SOLIDUS:
            {
              this.state = A.SCRIPT_DATA_END_TAG_OPEN;
              break;
            }
          case o.EXCLAMATION_MARK:
            {
              this.state = A.SCRIPT_DATA_ESCAPE_START, this._emitChars("<!");
              break;
            }
          default:
            this._emitChars("<"), this.state = A.SCRIPT_DATA, this._stateScriptData(e);
        }
      }
      _stateScriptDataEndTagOpen(e) {
        Ge(e) ? (this.state = A.SCRIPT_DATA_END_TAG_NAME, this._stateScriptDataEndTagName(e)) : (this._emitChars("</"), this.state = A.SCRIPT_DATA, this._stateScriptData(e));
      }
      _stateScriptDataEndTagName(e) {
        this.handleSpecialEndTag(e) && (this._emitChars("</"), this.state = A.SCRIPT_DATA, this._stateScriptData(e));
      }
      _stateScriptDataEscapeStart(e) {
        e === o.HYPHEN_MINUS ? (this.state = A.SCRIPT_DATA_ESCAPE_START_DASH, this._emitChars("-")) : (this.state = A.SCRIPT_DATA, this._stateScriptData(e));
      }
      _stateScriptDataEscapeStartDash(e) {
        e === o.HYPHEN_MINUS ? (this.state = A.SCRIPT_DATA_ESCAPED_DASH_DASH, this._emitChars("-")) : (this.state = A.SCRIPT_DATA, this._stateScriptData(e));
      }
      _stateScriptDataEscaped(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.SCRIPT_DATA_ESCAPED_DASH, this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateScriptDataEscapedDash(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.SCRIPT_DATA_ESCAPED_DASH_DASH, this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.state = A.SCRIPT_DATA_ESCAPED, this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this.state = A.SCRIPT_DATA_ESCAPED, this._emitCodePoint(e);
        }
      }
      _stateScriptDataEscapedDashDash(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_ESCAPED_LESS_THAN_SIGN;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA, this._emitChars(">");
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.state = A.SCRIPT_DATA_ESCAPED, this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this.state = A.SCRIPT_DATA_ESCAPED, this._emitCodePoint(e);
        }
      }
      _stateScriptDataEscapedLessThanSign(e) {
        e === o.SOLIDUS ? this.state = A.SCRIPT_DATA_ESCAPED_END_TAG_OPEN : Ge(e) ? (this._emitChars("<"), this.state = A.SCRIPT_DATA_DOUBLE_ESCAPE_START, this._stateScriptDataDoubleEscapeStart(e)) : (this._emitChars("<"), this.state = A.SCRIPT_DATA_ESCAPED, this._stateScriptDataEscaped(e));
      }
      _stateScriptDataEscapedEndTagOpen(e) {
        Ge(e) ? (this.state = A.SCRIPT_DATA_ESCAPED_END_TAG_NAME, this._stateScriptDataEscapedEndTagName(e)) : (this._emitChars("</"), this.state = A.SCRIPT_DATA_ESCAPED, this._stateScriptDataEscaped(e));
      }
      _stateScriptDataEscapedEndTagName(e) {
        this.handleSpecialEndTag(e) && (this._emitChars("</"), this.state = A.SCRIPT_DATA_ESCAPED, this._stateScriptDataEscaped(e));
      }
      _stateScriptDataDoubleEscapeStart(e) {
        if (this.preprocessor.startsWith(oe.SCRIPT, !1) && su(this.preprocessor.peek(oe.SCRIPT.length))) {
          this._emitCodePoint(e);
          for (let t = 0; t < oe.SCRIPT.length; t++) this._emitCodePoint(this._consume());
          this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED;
        } else this._ensureHibernation() || (this.state = A.SCRIPT_DATA_ESCAPED, this._stateScriptDataEscaped(e));
      }
      _stateScriptDataDoubleEscaped(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED_DASH, this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN, this._emitChars("<");
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateScriptDataDoubleEscapedDash(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED_DASH_DASH, this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN, this._emitChars("<");
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._emitCodePoint(e);
        }
      }
      _stateScriptDataDoubleEscapedDashDash(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this._emitChars("-");
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED_LESS_THAN_SIGN, this._emitChars("<");
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.SCRIPT_DATA, this._emitChars(">");
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._emitChars(H);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInScriptHtmlCommentLikeText), this._emitEOFToken();
              break;
            }
          default:
            this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._emitCodePoint(e);
        }
      }
      _stateScriptDataDoubleEscapedLessThanSign(e) {
        e === o.SOLIDUS ? (this.state = A.SCRIPT_DATA_DOUBLE_ESCAPE_END, this._emitChars("/")) : (this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._stateScriptDataDoubleEscaped(e));
      }
      _stateScriptDataDoubleEscapeEnd(e) {
        if (this.preprocessor.startsWith(oe.SCRIPT, !1) && su(this.preprocessor.peek(oe.SCRIPT.length))) {
          this._emitCodePoint(e);
          for (let t = 0; t < oe.SCRIPT.length; t++) this._emitCodePoint(this._consume());
          this.state = A.SCRIPT_DATA_ESCAPED;
        } else this._ensureHibernation() || (this.state = A.SCRIPT_DATA_DOUBLE_ESCAPED, this._stateScriptDataDoubleEscaped(e));
      }
      _stateBeforeAttributeName(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.SOLIDUS:
          case o.GREATER_THAN_SIGN:
          case o.EOF:
            {
              this.state = A.AFTER_ATTRIBUTE_NAME, this._stateAfterAttributeName(e);
              break;
            }
          case o.EQUALS_SIGN:
            {
              this._err(C.unexpectedEqualsSignBeforeAttributeName), this._createAttr("="), this.state = A.ATTRIBUTE_NAME;
              break;
            }
          default:
            this._createAttr(""), this.state = A.ATTRIBUTE_NAME, this._stateAttributeName(e);
        }
      }
      _stateAttributeName(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
          case o.SOLIDUS:
          case o.GREATER_THAN_SIGN:
          case o.EOF:
            {
              this._leaveAttrName(), this.state = A.AFTER_ATTRIBUTE_NAME, this._stateAfterAttributeName(e);
              break;
            }
          case o.EQUALS_SIGN:
            {
              this._leaveAttrName(), this.state = A.BEFORE_ATTRIBUTE_VALUE;
              break;
            }
          case o.QUOTATION_MARK:
          case o.APOSTROPHE:
          case o.LESS_THAN_SIGN:
            {
              this._err(C.unexpectedCharacterInAttributeName), this.currentAttr.name += String.fromCodePoint(e);
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.currentAttr.name += H;
              break;
            }
          default:
            this.currentAttr.name += String.fromCodePoint(yt(e) ? rn(e) : e);
        }
      }
      _stateAfterAttributeName(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.SOLIDUS:
            {
              this.state = A.SELF_CLOSING_START_TAG;
              break;
            }
          case o.EQUALS_SIGN:
            {
              this.state = A.BEFORE_ATTRIBUTE_VALUE;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this._createAttr(""), this.state = A.ATTRIBUTE_NAME, this._stateAttributeName(e);
        }
      }
      _stateBeforeAttributeValue(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.QUOTATION_MARK:
            {
              this.state = A.ATTRIBUTE_VALUE_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              this.state = A.ATTRIBUTE_VALUE_SINGLE_QUOTED;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingAttributeValue), this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          default:
            this.state = A.ATTRIBUTE_VALUE_UNQUOTED, this._stateAttributeValueUnquoted(e);
        }
      }
      _stateAttributeValueDoubleQuoted(e) {
        switch (e) {
          case o.QUOTATION_MARK:
            {
              this.state = A.AFTER_ATTRIBUTE_VALUE_QUOTED;
              break;
            }
          case o.AMPERSAND:
            {
              this._startCharacterReference();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.currentAttr.value += H;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this.currentAttr.value += String.fromCodePoint(e);
        }
      }
      _stateAttributeValueSingleQuoted(e) {
        switch (e) {
          case o.APOSTROPHE:
            {
              this.state = A.AFTER_ATTRIBUTE_VALUE_QUOTED;
              break;
            }
          case o.AMPERSAND:
            {
              this._startCharacterReference();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.currentAttr.value += H;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this.currentAttr.value += String.fromCodePoint(e);
        }
      }
      _stateAttributeValueUnquoted(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this._leaveAttrValue(), this.state = A.BEFORE_ATTRIBUTE_NAME;
              break;
            }
          case o.AMPERSAND:
            {
              this._startCharacterReference();
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._leaveAttrValue(), this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this.currentAttr.value += H;
              break;
            }
          case o.QUOTATION_MARK:
          case o.APOSTROPHE:
          case o.LESS_THAN_SIGN:
          case o.EQUALS_SIGN:
          case o.GRAVE_ACCENT:
            {
              this._err(C.unexpectedCharacterInUnquotedAttributeValue), this.currentAttr.value += String.fromCodePoint(e);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this.currentAttr.value += String.fromCodePoint(e);
        }
      }
      _stateAfterAttributeValueQuoted(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this._leaveAttrValue(), this.state = A.BEFORE_ATTRIBUTE_NAME;
              break;
            }
          case o.SOLIDUS:
            {
              this._leaveAttrValue(), this.state = A.SELF_CLOSING_START_TAG;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._leaveAttrValue(), this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingWhitespaceBetweenAttributes), this.state = A.BEFORE_ATTRIBUTE_NAME, this._stateBeforeAttributeName(e);
        }
      }
      _stateSelfClosingStartTag(e) {
        switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              let t = this.currentToken;
              t.selfClosing = !0, this.state = A.DATA, this.emitCurrentTagToken();
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInTag), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.unexpectedSolidusInTag), this.state = A.BEFORE_ATTRIBUTE_NAME, this._stateBeforeAttributeName(e);
        }
      }
      _stateBogusComment(e) {
        let t = this.currentToken;
        switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentComment(t);
              break;
            }
          case o.EOF:
            {
              this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.data += H;
              break;
            }
          default:
            t.data += String.fromCodePoint(e);
        }
      }
      _stateMarkupDeclarationOpen(e) {
        this._consumeSequenceIfMatch(oe.DASH_DASH, !0) ? (this._createCommentToken(oe.DASH_DASH.length + 1), this.state = A.COMMENT_START) : this._consumeSequenceIfMatch(oe.DOCTYPE, !1) ? (this.currentLocation = this.getCurrentLocation(oe.DOCTYPE.length + 1), this.state = A.DOCTYPE) : this._consumeSequenceIfMatch(oe.CDATA_START, !0) ? this.inForeignNode ? this.state = A.CDATA_SECTION : (this._err(C.cdataInHtmlContent), this._createCommentToken(oe.CDATA_START.length + 1), this.currentToken.data = "[CDATA[", this.state = A.BOGUS_COMMENT) : this._ensureHibernation() || (this._err(C.incorrectlyOpenedComment), this._createCommentToken(2), this.state = A.BOGUS_COMMENT, this._stateBogusComment(e));
      }
      _stateCommentStart(e) {
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.COMMENT_START_DASH;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptClosingOfEmptyComment), this.state = A.DATA;
              let t = this.currentToken;
              this.emitCurrentComment(t);
              break;
            }
          default:
            this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateCommentStartDash(e) {
        let t = this.currentToken;
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.COMMENT_END;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptClosingOfEmptyComment), this.state = A.DATA, this.emitCurrentComment(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInComment), this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          default:
            t.data += "-", this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateComment(e) {
        let t = this.currentToken;
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.COMMENT_END_DASH;
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              t.data += "<", this.state = A.COMMENT_LESS_THAN_SIGN;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.data += H;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInComment), this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          default:
            t.data += String.fromCodePoint(e);
        }
      }
      _stateCommentLessThanSign(e) {
        let t = this.currentToken;
        switch (e) {
          case o.EXCLAMATION_MARK:
            {
              t.data += "!", this.state = A.COMMENT_LESS_THAN_SIGN_BANG;
              break;
            }
          case o.LESS_THAN_SIGN:
            {
              t.data += "<";
              break;
            }
          default:
            this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateCommentLessThanSignBang(e) {
        e === o.HYPHEN_MINUS ? this.state = A.COMMENT_LESS_THAN_SIGN_BANG_DASH : (this.state = A.COMMENT, this._stateComment(e));
      }
      _stateCommentLessThanSignBangDash(e) {
        e === o.HYPHEN_MINUS ? this.state = A.COMMENT_LESS_THAN_SIGN_BANG_DASH_DASH : (this.state = A.COMMENT_END_DASH, this._stateCommentEndDash(e));
      }
      _stateCommentLessThanSignBangDashDash(e) {
        e !== o.GREATER_THAN_SIGN && e !== o.EOF && this._err(C.nestedComment), this.state = A.COMMENT_END, this._stateCommentEnd(e);
      }
      _stateCommentEndDash(e) {
        let t = this.currentToken;
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              this.state = A.COMMENT_END;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInComment), this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          default:
            t.data += "-", this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateCommentEnd(e) {
        let t = this.currentToken;
        switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentComment(t);
              break;
            }
          case o.EXCLAMATION_MARK:
            {
              this.state = A.COMMENT_END_BANG;
              break;
            }
          case o.HYPHEN_MINUS:
            {
              t.data += "-";
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInComment), this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          default:
            t.data += "--", this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateCommentEndBang(e) {
        let t = this.currentToken;
        switch (e) {
          case o.HYPHEN_MINUS:
            {
              t.data += "--!", this.state = A.COMMENT_END_DASH;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.incorrectlyClosedComment), this.state = A.DATA, this.emitCurrentComment(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInComment), this.emitCurrentComment(t), this._emitEOFToken();
              break;
            }
          default:
            t.data += "--!", this.state = A.COMMENT, this._stateComment(e);
        }
      }
      _stateDoctype(e) {
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.BEFORE_DOCTYPE_NAME;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.BEFORE_DOCTYPE_NAME, this._stateBeforeDoctypeName(e);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), this._createDoctypeToken(null);
              let t = this.currentToken;
              t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingWhitespaceBeforeDoctypeName), this.state = A.BEFORE_DOCTYPE_NAME, this._stateBeforeDoctypeName(e);
        }
      }
      _stateBeforeDoctypeName(e) {
        if (yt(e)) this._createDoctypeToken(String.fromCharCode(rn(e))), this.state = A.DOCTYPE_NAME;else switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), this._createDoctypeToken(H), this.state = A.DOCTYPE_NAME;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingDoctypeName), this._createDoctypeToken(null);
              let t = this.currentToken;
              t.forceQuirks = !0, this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), this._createDoctypeToken(null);
              let t = this.currentToken;
              t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._createDoctypeToken(String.fromCodePoint(e)), this.state = A.DOCTYPE_NAME;
        }
      }
      _stateDoctypeName(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.AFTER_DOCTYPE_NAME;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.name += H;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            t.name += String.fromCodePoint(yt(e) ? rn(e) : e);
        }
      }
      _stateAfterDoctypeName(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._consumeSequenceIfMatch(oe.PUBLIC, !1) ? this.state = A.AFTER_DOCTYPE_PUBLIC_KEYWORD : this._consumeSequenceIfMatch(oe.SYSTEM, !1) ? this.state = A.AFTER_DOCTYPE_SYSTEM_KEYWORD : this._ensureHibernation() || (this._err(C.invalidCharacterSequenceAfterDoctypeName), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e));
        }
      }
      _stateAfterDoctypePublicKeyword(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.BEFORE_DOCTYPE_PUBLIC_IDENTIFIER;
              break;
            }
          case o.QUOTATION_MARK:
            {
              this._err(C.missingWhitespaceAfterDoctypePublicKeyword), t.publicId = "", this.state = A.DOCTYPE_PUBLIC_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              this._err(C.missingWhitespaceAfterDoctypePublicKeyword), t.publicId = "", this.state = A.DOCTYPE_PUBLIC_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingDoctypePublicIdentifier), t.forceQuirks = !0, this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypePublicIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateBeforeDoctypePublicIdentifier(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.QUOTATION_MARK:
            {
              t.publicId = "", this.state = A.DOCTYPE_PUBLIC_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              t.publicId = "", this.state = A.DOCTYPE_PUBLIC_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingDoctypePublicIdentifier), t.forceQuirks = !0, this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypePublicIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateDoctypePublicIdentifierDoubleQuoted(e) {
        let t = this.currentToken;
        switch (e) {
          case o.QUOTATION_MARK:
            {
              this.state = A.AFTER_DOCTYPE_PUBLIC_IDENTIFIER;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.publicId += H;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptDoctypePublicIdentifier), t.forceQuirks = !0, this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            t.publicId += String.fromCodePoint(e);
        }
      }
      _stateDoctypePublicIdentifierSingleQuoted(e) {
        let t = this.currentToken;
        switch (e) {
          case o.APOSTROPHE:
            {
              this.state = A.AFTER_DOCTYPE_PUBLIC_IDENTIFIER;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.publicId += H;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptDoctypePublicIdentifier), t.forceQuirks = !0, this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            t.publicId += String.fromCodePoint(e);
        }
      }
      _stateAfterDoctypePublicIdentifier(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.BETWEEN_DOCTYPE_PUBLIC_AND_SYSTEM_IDENTIFIERS;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.QUOTATION_MARK:
            {
              this._err(C.missingWhitespaceBetweenDoctypePublicAndSystemIdentifiers), t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              this._err(C.missingWhitespaceBetweenDoctypePublicAndSystemIdentifiers), t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateBetweenDoctypePublicAndSystemIdentifiers(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.GREATER_THAN_SIGN:
            {
              this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.QUOTATION_MARK:
            {
              t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateAfterDoctypeSystemKeyword(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            {
              this.state = A.BEFORE_DOCTYPE_SYSTEM_IDENTIFIER;
              break;
            }
          case o.QUOTATION_MARK:
            {
              this._err(C.missingWhitespaceAfterDoctypeSystemKeyword), t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              this._err(C.missingWhitespaceAfterDoctypeSystemKeyword), t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateBeforeDoctypeSystemIdentifier(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.QUOTATION_MARK:
            {
              t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_DOUBLE_QUOTED;
              break;
            }
          case o.APOSTROPHE:
            {
              t.systemId = "", this.state = A.DOCTYPE_SYSTEM_IDENTIFIER_SINGLE_QUOTED;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.missingDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.DATA, this.emitCurrentDoctype(t);
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.missingQuoteBeforeDoctypeSystemIdentifier), t.forceQuirks = !0, this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateDoctypeSystemIdentifierDoubleQuoted(e) {
        let t = this.currentToken;
        switch (e) {
          case o.QUOTATION_MARK:
            {
              this.state = A.AFTER_DOCTYPE_SYSTEM_IDENTIFIER;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.systemId += H;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptDoctypeSystemIdentifier), t.forceQuirks = !0, this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            t.systemId += String.fromCodePoint(e);
        }
      }
      _stateDoctypeSystemIdentifierSingleQuoted(e) {
        let t = this.currentToken;
        switch (e) {
          case o.APOSTROPHE:
            {
              this.state = A.AFTER_DOCTYPE_SYSTEM_IDENTIFIER;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter), t.systemId += H;
              break;
            }
          case o.GREATER_THAN_SIGN:
            {
              this._err(C.abruptDoctypeSystemIdentifier), t.forceQuirks = !0, this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            t.systemId += String.fromCodePoint(e);
        }
      }
      _stateAfterDoctypeSystemIdentifier(e) {
        let t = this.currentToken;
        switch (e) {
          case o.SPACE:
          case o.LINE_FEED:
          case o.TABULATION:
          case o.FORM_FEED:
            break;
          case o.GREATER_THAN_SIGN:
            {
              this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInDoctype), t.forceQuirks = !0, this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
            this._err(C.unexpectedCharacterAfterDoctypeSystemIdentifier), this.state = A.BOGUS_DOCTYPE, this._stateBogusDoctype(e);
        }
      }
      _stateBogusDoctype(e) {
        let t = this.currentToken;
        switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              this.emitCurrentDoctype(t), this.state = A.DATA;
              break;
            }
          case o.NULL:
            {
              this._err(C.unexpectedNullCharacter);
              break;
            }
          case o.EOF:
            {
              this.emitCurrentDoctype(t), this._emitEOFToken();
              break;
            }
          default:
        }
      }
      _stateCdataSection(e) {
        switch (e) {
          case o.RIGHT_SQUARE_BRACKET:
            {
              this.state = A.CDATA_SECTION_BRACKET;
              break;
            }
          case o.EOF:
            {
              this._err(C.eofInCdata), this._emitEOFToken();
              break;
            }
          default:
            this._emitCodePoint(e);
        }
      }
      _stateCdataSectionBracket(e) {
        e === o.RIGHT_SQUARE_BRACKET ? this.state = A.CDATA_SECTION_END : (this._emitChars("]"), this.state = A.CDATA_SECTION, this._stateCdataSection(e));
      }
      _stateCdataSectionEnd(e) {
        switch (e) {
          case o.GREATER_THAN_SIGN:
            {
              this.state = A.DATA;
              break;
            }
          case o.RIGHT_SQUARE_BRACKET:
            {
              this._emitChars("]");
              break;
            }
          default:
            this._emitChars("]]"), this.state = A.CDATA_SECTION, this._stateCdataSection(e);
        }
      }
      _stateCharacterReference() {
        let e = this.entityDecoder.write(this.preprocessor.html, this.preprocessor.pos);
        if (e < 0) if (this.preprocessor.lastChunkWritten) e = this.entityDecoder.end();else {
          this.active = !1, this.preprocessor.pos = this.preprocessor.html.length - 1, this.consumedAfterSnapshot = 0, this.preprocessor.endOfChunkHit = !0;
          return;
        }
        e === 0 ? (this.preprocessor.pos = this.entityStartPos, this._flushCodePointConsumedAsCharacterReference(o.AMPERSAND), this.state = !this._isCharacterReferenceInAttribute() && au(this.preprocessor.peek(1)) ? A.AMBIGUOUS_AMPERSAND : this.returnState) : this.state = this.returnState;
      }
      _stateAmbiguousAmpersand(e) {
        au(e) ? this._flushCodePointConsumedAsCharacterReference(e) : (e === o.SEMICOLON && this._err(C.unknownNamedCharacterReference), this.state = this.returnState, this._callState(e));
      }
    },
    ou = new Set([u.DD, u.DT, u.LI, u.OPTGROUP, u.OPTION, u.P, u.RB, u.RP, u.RT, u.RTC]),
    Au = new Set([...ou, u.CAPTION, u.COLGROUP, u.TBODY, u.TD, u.TFOOT, u.TH, u.THEAD, u.TR]),
    un = new Set([u.APPLET, u.CAPTION, u.HTML, u.MARQUEE, u.OBJECT, u.TABLE, u.TD, u.TEMPLATE, u.TH]),
    Lo = new Set([...un, u.OL, u.UL]),
    Mo = new Set([...un, u.BUTTON]),
    cu = new Set([u.ANNOTATION_XML, u.MI, u.MN, u.MO, u.MS, u.MTEXT]),
    lu = new Set([u.DESC, u.FOREIGN_OBJECT, u.TITLE]),
    Ho = new Set([u.TR, u.TEMPLATE, u.HTML]),
    Po = new Set([u.TBODY, u.TFOOT, u.THEAD, u.TEMPLATE, u.HTML]),
    Uo = new Set([u.TABLE, u.TEMPLATE, u.HTML]),
    Wo = new Set([u.TD, u.TH]),
    Go = class {
      get currentTmplContentOrNode() {
        return this._isInTemplate() ? this.treeAdapter.getTemplateContent(this.current) : this.current;
      }
      constructor(e, t, n) {
        this.treeAdapter = t, this.handler = n, this.items = [], this.tagIDs = [], this.stackTop = -1, this.tmplCount = 0, this.currentTagId = u.UNKNOWN, this.current = e;
      }
      _indexOf(e) {
        return this.items.lastIndexOf(e, this.stackTop);
      }
      _isInTemplate() {
        return this.currentTagId === u.TEMPLATE && this.treeAdapter.getNamespaceURI(this.current) === p.HTML;
      }
      _updateCurrentElement() {
        this.current = this.items[this.stackTop], this.currentTagId = this.tagIDs[this.stackTop];
      }
      push(e, t) {
        this.stackTop++, this.items[this.stackTop] = e, this.current = e, this.tagIDs[this.stackTop] = t, this.currentTagId = t, this._isInTemplate() && this.tmplCount++, this.handler.onItemPush(e, t, !0);
      }
      pop() {
        let e = this.current;
        this.tmplCount > 0 && this._isInTemplate() && this.tmplCount--, this.stackTop--, this._updateCurrentElement(), this.handler.onItemPop(e, !0);
      }
      replace(e, t) {
        let n = this._indexOf(e);
        this.items[n] = t, n === this.stackTop && (this.current = t);
      }
      insertAfter(e, t, n) {
        let r = this._indexOf(e) + 1;
        this.items.splice(r, 0, t), this.tagIDs.splice(r, 0, n), this.stackTop++, r === this.stackTop && this._updateCurrentElement(), this.current && this.currentTagId !== void 0 && this.handler.onItemPush(this.current, this.currentTagId, r === this.stackTop);
      }
      popUntilTagNamePopped(e) {
        let t = this.stackTop + 1;
        do t = this.tagIDs.lastIndexOf(e, t - 1); while (t > 0 && this.treeAdapter.getNamespaceURI(this.items[t]) !== p.HTML);
        this.shortenToLength(Math.max(t, 0));
      }
      shortenToLength(e) {
        for (; this.stackTop >= e;) {
          let t = this.current;
          this.tmplCount > 0 && this._isInTemplate() && (this.tmplCount -= 1), this.stackTop--, this._updateCurrentElement(), this.handler.onItemPop(t, this.stackTop < e);
        }
      }
      popUntilElementPopped(e) {
        let t = this._indexOf(e);
        this.shortenToLength(Math.max(t, 0));
      }
      popUntilPopped(e, t) {
        let n = this._indexOfTagNames(e, t);
        this.shortenToLength(Math.max(n, 0));
      }
      popUntilNumberedHeaderPopped() {
        this.popUntilPopped(nn, p.HTML);
      }
      popUntilTableCellPopped() {
        this.popUntilPopped(Wo, p.HTML);
      }
      popAllUpToHtmlElement() {
        this.tmplCount = 0, this.shortenToLength(1);
      }
      _indexOfTagNames(e, t) {
        for (let n = this.stackTop; n >= 0; n--) if (e.has(this.tagIDs[n]) && this.treeAdapter.getNamespaceURI(this.items[n]) === t) return n;
        return -1;
      }
      clearBackTo(e, t) {
        let n = this._indexOfTagNames(e, t);
        this.shortenToLength(n + 1);
      }
      clearBackToTableContext() {
        this.clearBackTo(Uo, p.HTML);
      }
      clearBackToTableBodyContext() {
        this.clearBackTo(Po, p.HTML);
      }
      clearBackToTableRowContext() {
        this.clearBackTo(Ho, p.HTML);
      }
      remove(e) {
        let t = this._indexOf(e);
        t >= 0 && (t === this.stackTop ? this.pop() : (this.items.splice(t, 1), this.tagIDs.splice(t, 1), this.stackTop--, this._updateCurrentElement(), this.handler.onItemPop(e, !1)));
      }
      tryPeekProperlyNestedBodyElement() {
        return this.stackTop >= 1 && this.tagIDs[1] === u.BODY ? this.items[1] : null;
      }
      contains(e) {
        return this._indexOf(e) > -1;
      }
      getCommonAncestor(e) {
        let t = this._indexOf(e) - 1;
        return t >= 0 ? this.items[t] : null;
      }
      isRootHtmlElementCurrent() {
        return this.stackTop === 0 && this.tagIDs[0] === u.HTML;
      }
      hasInDynamicScope(e, t) {
        for (let n = this.stackTop; n >= 0; n--) {
          let r = this.tagIDs[n];
          switch (this.treeAdapter.getNamespaceURI(this.items[n])) {
            case p.HTML:
              {
                if (r === e) return !0;
                if (t.has(r)) return !1;
                break;
              }
            case p.SVG:
              {
                if (lu.has(r)) return !1;
                break;
              }
            case p.MATHML:
              {
                if (cu.has(r)) return !1;
                break;
              }
          }
        }
        return !0;
      }
      hasInScope(e) {
        return this.hasInDynamicScope(e, un);
      }
      hasInListItemScope(e) {
        return this.hasInDynamicScope(e, Lo);
      }
      hasInButtonScope(e) {
        return this.hasInDynamicScope(e, Mo);
      }
      hasNumberedHeaderInScope() {
        for (let e = this.stackTop; e >= 0; e--) {
          let t = this.tagIDs[e];
          switch (this.treeAdapter.getNamespaceURI(this.items[e])) {
            case p.HTML:
              {
                if (nn.has(t)) return !0;
                if (un.has(t)) return !1;
                break;
              }
            case p.SVG:
              {
                if (lu.has(t)) return !1;
                break;
              }
            case p.MATHML:
              {
                if (cu.has(t)) return !1;
                break;
              }
          }
        }
        return !0;
      }
      hasInTableScope(e) {
        for (let t = this.stackTop; t >= 0; t--) if (this.treeAdapter.getNamespaceURI(this.items[t]) === p.HTML) switch (this.tagIDs[t]) {
          case e:
            return !0;
          case u.TABLE:
          case u.HTML:
            return !1;
        }
        return !0;
      }
      hasTableBodyContextInTableScope() {
        for (let e = this.stackTop; e >= 0; e--) if (this.treeAdapter.getNamespaceURI(this.items[e]) === p.HTML) switch (this.tagIDs[e]) {
          case u.TBODY:
          case u.THEAD:
          case u.TFOOT:
            return !0;
          case u.TABLE:
          case u.HTML:
            return !1;
        }
        return !0;
      }
      hasInSelectScope(e) {
        for (let t = this.stackTop; t >= 0; t--) if (this.treeAdapter.getNamespaceURI(this.items[t]) === p.HTML) switch (this.tagIDs[t]) {
          case e:
            return !0;
          case u.OPTION:
          case u.OPTGROUP:
            break;
          default:
            return !1;
        }
        return !0;
      }
      generateImpliedEndTags() {
        for (; this.currentTagId !== void 0 && ou.has(this.currentTagId);) this.pop();
      }
      generateImpliedEndTagsThoroughly() {
        for (; this.currentTagId !== void 0 && Au.has(this.currentTagId);) this.pop();
      }
      generateImpliedEndTagsWithExclusion(e) {
        for (; this.currentTagId !== void 0 && this.currentTagId !== e && Au.has(this.currentTagId);) this.pop();
      }
    },
    ge;
  (function (e) {
    e[e.Marker = 0] = "Marker", e[e.Element = 1] = "Element";
  })(ge || (ge = {}));
  var hu = {
      type: ge.Marker
    },
    Ko = class {
      constructor(e) {
        this.treeAdapter = e, this.entries = [], this.bookmark = null;
      }
      _getNoahArkConditionCandidates(e, t) {
        let n = [],
          r = t.length,
          a = this.treeAdapter.getTagName(e),
          i = this.treeAdapter.getNamespaceURI(e);
        for (let s = 0; s < this.entries.length; s++) {
          let l = this.entries[s];
          if (l.type === ge.Marker) break;
          let c = l.element;
          if (this.treeAdapter.getTagName(c) === a && this.treeAdapter.getNamespaceURI(c) === i) {
            let E = this.treeAdapter.getAttrList(c);
            E.length === r && n.push({
              idx: s,
              attrs: E
            });
          }
        }
        return n;
      }
      _ensureNoahArkCondition(e) {
        if (this.entries.length < 3) return;
        let t = this.treeAdapter.getAttrList(e),
          n = this._getNoahArkConditionCandidates(e, t);
        if (n.length < 3) return;
        let r = new Map(t.map(i => [i.name, i.value])),
          a = 0;
        for (let i = 0; i < n.length; i++) {
          let s = n[i];
          s.attrs.every(l => r.get(l.name) === l.value) && (a += 1, a >= 3 && this.entries.splice(s.idx, 1));
        }
      }
      insertMarker() {
        this.entries.unshift(hu);
      }
      pushElement(e, t) {
        this._ensureNoahArkCondition(e), this.entries.unshift({
          type: ge.Element,
          element: e,
          token: t
        });
      }
      insertElementAfterBookmark(e, t) {
        let n = this.entries.indexOf(this.bookmark);
        this.entries.splice(n, 0, {
          type: ge.Element,
          element: e,
          token: t
        });
      }
      removeEntry(e) {
        let t = this.entries.indexOf(e);
        t !== -1 && this.entries.splice(t, 1);
      }
      clearToLastMarker() {
        let e = this.entries.indexOf(hu);
        e === -1 ? this.entries.length = 0 : this.entries.splice(0, e + 1);
      }
      getElementEntryInScopeWithTagName(e) {
        let t = this.entries.find(n => n.type === ge.Marker || this.treeAdapter.getTagName(n.element) === e);
        return t && t.type === ge.Element ? t : null;
      }
      getElementEntry(e) {
        return this.entries.find(t => t.type === ge.Element && t.element === e);
      }
    },
    ve = {
      createDocument() {
        return {
          nodeName: "#document",
          mode: de.NO_QUIRKS,
          childNodes: []
        };
      },
      createDocumentFragment() {
        return {
          nodeName: "#document-fragment",
          childNodes: []
        };
      },
      createElement(e, t, n) {
        return {
          nodeName: e,
          tagName: e,
          attrs: n,
          namespaceURI: t,
          childNodes: [],
          parentNode: null
        };
      },
      createCommentNode(e) {
        return {
          nodeName: "#comment",
          data: e,
          parentNode: null
        };
      },
      createTextNode(e) {
        return {
          nodeName: "#text",
          value: e,
          parentNode: null
        };
      },
      appendChild(e, t) {
        e.childNodes.push(t), t.parentNode = e;
      },
      insertBefore(e, t, n) {
        let r = e.childNodes.indexOf(n);
        e.childNodes.splice(r, 0, t), t.parentNode = e;
      },
      setTemplateContent(e, t) {
        e.content = t;
      },
      getTemplateContent(e) {
        return e.content;
      },
      setDocumentType(e, t, n, r) {
        let a = e.childNodes.find(i => i.nodeName === "#documentType");
        if (a) a.name = t, a.publicId = n, a.systemId = r;else {
          let i = {
            nodeName: "#documentType",
            name: t,
            publicId: n,
            systemId: r,
            parentNode: null
          };
          ve.appendChild(e, i);
        }
      },
      setDocumentMode(e, t) {
        e.mode = t;
      },
      getDocumentMode(e) {
        return e.mode;
      },
      detachNode(e) {
        if (e.parentNode) {
          let t = e.parentNode.childNodes.indexOf(e);
          e.parentNode.childNodes.splice(t, 1), e.parentNode = null;
        }
      },
      insertText(e, t) {
        if (e.childNodes.length > 0) {
          let n = e.childNodes[e.childNodes.length - 1];
          if (ve.isTextNode(n)) {
            n.value += t;
            return;
          }
        }
        ve.appendChild(e, ve.createTextNode(t));
      },
      insertTextBefore(e, t, n) {
        let r = e.childNodes[e.childNodes.indexOf(n) - 1];
        r && ve.isTextNode(r) ? r.value += t : ve.insertBefore(e, ve.createTextNode(t), n);
      },
      adoptAttributes(e, t) {
        let n = new Set(e.attrs.map(r => r.name));
        for (let r = 0; r < t.length; r++) n.has(t[r].name) || e.attrs.push(t[r]);
      },
      getFirstChild(e) {
        return e.childNodes[0];
      },
      getChildNodes(e) {
        return e.childNodes;
      },
      getParentNode(e) {
        return e.parentNode;
      },
      getAttrList(e) {
        return e.attrs;
      },
      getTagName(e) {
        return e.tagName;
      },
      getNamespaceURI(e) {
        return e.namespaceURI;
      },
      getTextNodeContent(e) {
        return e.value;
      },
      getCommentNodeContent(e) {
        return e.data;
      },
      getDocumentTypeNodeName(e) {
        return e.name;
      },
      getDocumentTypeNodePublicId(e) {
        return e.publicId;
      },
      getDocumentTypeNodeSystemId(e) {
        return e.systemId;
      },
      isTextNode(e) {
        return e.nodeName === "#text";
      },
      isCommentNode(e) {
        return e.nodeName === "#comment";
      },
      isDocumentTypeNode(e) {
        return e.nodeName === "#documentType";
      },
      isElementNode(e) {
        return Object.prototype.hasOwnProperty.call(e, "tagName");
      },
      setNodeSourceCodeLocation(e, t) {
        e.sourceCodeLocation = t;
      },
      getNodeSourceCodeLocation(e) {
        return e.sourceCodeLocation;
      },
      updateNodeSourceCodeLocation(e, t) {
        e.sourceCodeLocation = L(L({}, e.sourceCodeLocation), t);
      }
    },
    du = "html",
    Qo = "about:legacy-compat",
    Yo = "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd",
    Eu = ["+//silmaril//dtd html pro v0r11 19970101//", "-//as//dtd html 3.0 aswedit + extensions//", "-//advasoft ltd//dtd html 3.0 aswedit + extensions//", "-//ietf//dtd html 2.0 level 1//", "-//ietf//dtd html 2.0 level 2//", "-//ietf//dtd html 2.0 strict level 1//", "-//ietf//dtd html 2.0 strict level 2//", "-//ietf//dtd html 2.0 strict//", "-//ietf//dtd html 2.0//", "-//ietf//dtd html 2.1e//", "-//ietf//dtd html 3.0//", "-//ietf//dtd html 3.2 final//", "-//ietf//dtd html 3.2//", "-//ietf//dtd html 3//", "-//ietf//dtd html level 0//", "-//ietf//dtd html level 1//", "-//ietf//dtd html level 2//", "-//ietf//dtd html level 3//", "-//ietf//dtd html strict level 0//", "-//ietf//dtd html strict level 1//", "-//ietf//dtd html strict level 2//", "-//ietf//dtd html strict level 3//", "-//ietf//dtd html strict//", "-//ietf//dtd html//", "-//metrius//dtd metrius presentational//", "-//microsoft//dtd internet explorer 2.0 html strict//", "-//microsoft//dtd internet explorer 2.0 html//", "-//microsoft//dtd internet explorer 2.0 tables//", "-//microsoft//dtd internet explorer 3.0 html strict//", "-//microsoft//dtd internet explorer 3.0 html//", "-//microsoft//dtd internet explorer 3.0 tables//", "-//netscape comm. corp.//dtd html//", "-//netscape comm. corp.//dtd strict html//", "-//o'reilly and associates//dtd html 2.0//", "-//o'reilly and associates//dtd html extended 1.0//", "-//o'reilly and associates//dtd html extended relaxed 1.0//", "-//sq//dtd html 2.0 hotmetal + extensions//", "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//", "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//", "-//spyglass//dtd html 2.0 extended//", "-//sun microsystems corp.//dtd hotjava html//", "-//sun microsystems corp.//dtd hotjava strict html//", "-//w3c//dtd html 3 1995-03-24//", "-//w3c//dtd html 3.2 draft//", "-//w3c//dtd html 3.2 final//", "-//w3c//dtd html 3.2//", "-//w3c//dtd html 3.2s draft//", "-//w3c//dtd html 4.0 frameset//", "-//w3c//dtd html 4.0 transitional//", "-//w3c//dtd html experimental 19960712//", "-//w3c//dtd html experimental 970421//", "-//w3c//dtd w3 html//", "-//w3o//dtd w3 html 3.0//", "-//webtechs//dtd mozilla html 2.0//", "-//webtechs//dtd mozilla html//"],
    Xo = [...Eu, "-//w3c//dtd html 4.01 frameset//", "-//w3c//dtd html 4.01 transitional//"],
    Jo = new Set(["-//w3o//dtd w3 html strict 3.0//en//", "-/w3c/dtd html 4.0 transitional/en", "html"]),
    Cu = ["-//w3c//dtd xhtml 1.0 frameset//", "-//w3c//dtd xhtml 1.0 transitional//"],
    qo = [...Cu, "-//w3c//dtd html 4.01 frameset//", "-//w3c//dtd html 4.01 transitional//"];
  function pu(e, t) {
    return t.some(n => e.startsWith(n));
  }
  function Vo(e) {
    return e.name === du && e.publicId === null && (e.systemId === null || e.systemId === Qo);
  }
  function jo(e) {
    if (e.name !== du) return de.QUIRKS;
    let t = e.systemId;
    if (t && t.toLowerCase() === Yo) return de.QUIRKS;
    let n = e.publicId;
    if (n !== null) {
      if (n = n.toLowerCase(), Jo.has(n)) return de.QUIRKS;
      let r = t === null ? Xo : Eu;
      if (pu(n, r)) return de.QUIRKS;
      if (r = t === null ? Cu : qo, pu(n, r)) return de.LIMITED_QUIRKS;
    }
    return de.NO_QUIRKS;
  }
  var Zo = {};
  le(Zo, {
    SVG_TAG_NAMES_ADJUSTMENT_MAP: () => xu,
    adjustTokenMathMLAttrs: () => h0,
    adjustTokenSVGAttrs: () => d0,
    adjustTokenSVGTagName: () => mu,
    adjustTokenXMLAttrs: () => an,
    causesExit: () => Bu,
    isIntegrationPoint: () => Du
  });
  var fu = {
      TEXT_HTML: "text/html",
      APPLICATION_XML: "application/xhtml+xml"
    },
    zo = "definitionurl",
    $o = "definitionURL",
    eA = new Map(["attributeName", "attributeType", "baseFrequency", "baseProfile", "calcMode", "clipPathUnits", "diffuseConstant", "edgeMode", "filterUnits", "glyphRef", "gradientTransform", "gradientUnits", "kernelMatrix", "kernelUnitLength", "keyPoints", "keySplines", "keyTimes", "lengthAdjust", "limitingConeAngle", "markerHeight", "markerUnits", "markerWidth", "maskContentUnits", "maskUnits", "numOctaves", "pathLength", "patternContentUnits", "patternTransform", "patternUnits", "pointsAtX", "pointsAtY", "pointsAtZ", "preserveAlpha", "preserveAspectRatio", "primitiveUnits", "refX", "refY", "repeatCount", "repeatDur", "requiredExtensions", "requiredFeatures", "specularConstant", "specularExponent", "spreadMethod", "startOffset", "stdDeviation", "stitchTiles", "surfaceScale", "systemLanguage", "tableValues", "targetX", "targetY", "textLength", "viewBox", "viewTarget", "xChannelSelector", "yChannelSelector", "zoomAndPan"].map(e => [e.toLowerCase(), e])),
    tA = new Map([["xlink:actuate", {
      prefix: "xlink",
      name: "actuate",
      namespace: p.XLINK
    }], ["xlink:arcrole", {
      prefix: "xlink",
      name: "arcrole",
      namespace: p.XLINK
    }], ["xlink:href", {
      prefix: "xlink",
      name: "href",
      namespace: p.XLINK
    }], ["xlink:role", {
      prefix: "xlink",
      name: "role",
      namespace: p.XLINK
    }], ["xlink:show", {
      prefix: "xlink",
      name: "show",
      namespace: p.XLINK
    }], ["xlink:title", {
      prefix: "xlink",
      name: "title",
      namespace: p.XLINK
    }], ["xlink:type", {
      prefix: "xlink",
      name: "type",
      namespace: p.XLINK
    }], ["xml:lang", {
      prefix: "xml",
      name: "lang",
      namespace: p.XML
    }], ["xml:space", {
      prefix: "xml",
      name: "space",
      namespace: p.XML
    }], ["xmlns", {
      prefix: "",
      name: "xmlns",
      namespace: p.XMLNS
    }], ["xmlns:xlink", {
      prefix: "xmlns",
      name: "xlink",
      namespace: p.XMLNS
    }]]),
    xu = new Map(["altGlyph", "altGlyphDef", "altGlyphItem", "animateColor", "animateMotion", "animateTransform", "clipPath", "feBlend", "feColorMatrix", "feComponentTransfer", "feComposite", "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap", "feDistantLight", "feFlood", "feFuncA", "feFuncB", "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge", "feMergeNode", "feMorphology", "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight", "feTile", "feTurbulence", "foreignObject", "glyphRef", "linearGradient", "radialGradient", "textPath"].map(e => [e.toLowerCase(), e])),
    nA = new Set([u.B, u.BIG, u.BLOCKQUOTE, u.BODY, u.BR, u.CENTER, u.CODE, u.DD, u.DIV, u.DL, u.DT, u.EM, u.EMBED, u.H1, u.H2, u.H3, u.H4, u.H5, u.H6, u.HEAD, u.HR, u.I, u.IMG, u.LI, u.LISTING, u.MENU, u.META, u.NOBR, u.OL, u.P, u.PRE, u.RUBY, u.S, u.SMALL, u.SPAN, u.STRONG, u.STRIKE, u.SUB, u.SUP, u.TABLE, u.TT, u.U, u.UL, u.VAR]);
  function Bu(e) {
    let t = e.tagID;
    return t === u.FONT && e.attrs.some(({
      name: n
    }) => n === We.COLOR || n === We.SIZE || n === We.FACE) || nA.has(t);
  }
  function h0(e) {
    for (let t = 0; t < e.attrs.length; t++) if (e.attrs[t].name === zo) {
      e.attrs[t].name = $o;
      break;
    }
  }
  function d0(e) {
    for (let t = 0; t < e.attrs.length; t++) {
      let n = eA.get(e.attrs[t].name);
      n != null && (e.attrs[t].name = n);
    }
  }
  function an(e) {
    for (let t = 0; t < e.attrs.length; t++) {
      let n = tA.get(e.attrs[t].name);
      n && (e.attrs[t].prefix = n.prefix, e.attrs[t].name = n.name, e.attrs[t].namespace = n.namespace);
    }
  }
  function mu(e) {
    let t = xu.get(e.tagName);
    t != null && (e.tagName = t, e.tagID = _t(e.tagName));
  }
  function rA(e, t) {
    return t === p.MATHML && (e === u.MI || e === u.MO || e === u.MN || e === u.MS || e === u.MTEXT);
  }
  function uA(e, t, n) {
    if (t === p.MATHML && e === u.ANNOTATION_XML) {
      for (let r = 0; r < n.length; r++) if (n[r].name === We.ENCODING) {
        let a = n[r].value.toLowerCase();
        return a === fu.TEXT_HTML || a === fu.APPLICATION_XML;
      }
    }
    return t === p.SVG && (e === u.FOREIGN_OBJECT || e === u.DESC || e === u.TITLE);
  }
  function Du(e, t, n, r) {
    return (!r || r === p.HTML) && uA(e, t, n) || (!r || r === p.MATHML) && rA(e, t);
  }
  var aA = "hidden",
    iA = 8,
    sA = 3,
    h;
  (function (e) {
    e[e.INITIAL = 0] = "INITIAL", e[e.BEFORE_HTML = 1] = "BEFORE_HTML", e[e.BEFORE_HEAD = 2] = "BEFORE_HEAD", e[e.IN_HEAD = 3] = "IN_HEAD", e[e.IN_HEAD_NO_SCRIPT = 4] = "IN_HEAD_NO_SCRIPT", e[e.AFTER_HEAD = 5] = "AFTER_HEAD", e[e.IN_BODY = 6] = "IN_BODY", e[e.TEXT = 7] = "TEXT", e[e.IN_TABLE = 8] = "IN_TABLE", e[e.IN_TABLE_TEXT = 9] = "IN_TABLE_TEXT", e[e.IN_CAPTION = 10] = "IN_CAPTION", e[e.IN_COLUMN_GROUP = 11] = "IN_COLUMN_GROUP", e[e.IN_TABLE_BODY = 12] = "IN_TABLE_BODY", e[e.IN_ROW = 13] = "IN_ROW", e[e.IN_CELL = 14] = "IN_CELL", e[e.IN_SELECT = 15] = "IN_SELECT", e[e.IN_SELECT_IN_TABLE = 16] = "IN_SELECT_IN_TABLE", e[e.IN_TEMPLATE = 17] = "IN_TEMPLATE", e[e.AFTER_BODY = 18] = "AFTER_BODY", e[e.IN_FRAMESET = 19] = "IN_FRAMESET", e[e.AFTER_FRAMESET = 20] = "AFTER_FRAMESET", e[e.AFTER_AFTER_BODY = 21] = "AFTER_AFTER_BODY", e[e.AFTER_AFTER_FRAMESET = 22] = "AFTER_AFTER_FRAMESET";
  })(h || (h = {}));
  var oA = {
      startLine: -1,
      startCol: -1,
      startOffset: -1,
      endLine: -1,
      endCol: -1,
      endOffset: -1
    },
    gu = new Set([u.TABLE, u.TBODY, u.TFOOT, u.THEAD, u.TR]),
    Tu = {
      scriptingEnabled: !0,
      sourceCodeLocationInfo: !1,
      treeAdapter: ve,
      onParseError: null
    },
    Fu = class {
      constructor(e, t, n = null, r = null) {
        this.fragmentContext = n, this.scriptHandler = r, this.currentToken = null, this.stopped = !1, this.insertionMode = h.INITIAL, this.originalInsertionMode = h.INITIAL, this.headElement = null, this.formElement = null, this.currentNotInHTML = !1, this.tmplInsertionModeStack = [], this.pendingCharacterTokens = [], this.hasNonWhitespacePendingCharacterToken = !1, this.framesetOk = !0, this.skipNextNewLine = !1, this.fosterParentingEnabled = !1, this.options = L(L({}, Tu), e), this.treeAdapter = this.options.treeAdapter, this.onParseError = this.options.onParseError, this.onParseError && (this.options.sourceCodeLocationInfo = !0), this.document = t != null ? t : this.treeAdapter.createDocument(), this.tokenizer = new Oo(this.options, this), this.activeFormattingElements = new Ko(this.treeAdapter), this.fragmentContextID = n ? _t(this.treeAdapter.getTagName(n)) : u.UNKNOWN, this._setContextModes(n != null ? n : this.document, this.fragmentContextID), this.openElements = new Go(this.document, this.treeAdapter, this);
      }
      static parse(e, t) {
        let n = new this(t);
        return n.tokenizer.write(e, !0), n.document;
      }
      static getFragmentParser(e, t) {
        let n = L(L({}, Tu), t);
        e != null || (e = n.treeAdapter.createElement(d.TEMPLATE, p.HTML, []));
        let r = n.treeAdapter.createElement("documentmock", p.HTML, []),
          a = new this(n, r, e);
        return a.fragmentContextID === u.TEMPLATE && a.tmplInsertionModeStack.unshift(h.IN_TEMPLATE), a._initTokenizerForFragmentParsing(), a._insertFakeRootElement(), a._resetInsertionMode(), a._findFormInFragmentContext(), a;
      }
      getFragment() {
        let e = this.treeAdapter.getFirstChild(this.document),
          t = this.treeAdapter.createDocumentFragment();
        return this._adoptNodes(e, t), t;
      }
      _err(e, t, n) {
        var r;
        if (!this.onParseError) return;
        let a = (r = e.location) !== null && r !== void 0 ? r : oA,
          i = {
            code: t,
            startLine: a.startLine,
            startCol: a.startCol,
            startOffset: a.startOffset,
            endLine: n ? a.startLine : a.endLine,
            endCol: n ? a.startCol : a.endCol,
            endOffset: n ? a.startOffset : a.endOffset
          };
        this.onParseError(i);
      }
      onItemPush(e, t, n) {
        var r, a;
        (a = (r = this.treeAdapter).onItemPush) === null || a === void 0 || a.call(r, e), n && this.openElements.stackTop > 0 && this._setContextModes(e, t);
      }
      onItemPop(e, t) {
        var n, r;
        if (this.options.sourceCodeLocationInfo && this._setEndLocation(e, this.currentToken), (r = (n = this.treeAdapter).onItemPop) === null || r === void 0 || r.call(n, e, this.openElements.current), t) {
          var a;
          let i, s;
          this.openElements.stackTop === 0 && this.fragmentContext ? (i = this.fragmentContext, s = this.fragmentContextID) : (a = this.openElements, i = a.current, s = a.currentTagId), this._setContextModes(i, s);
        }
      }
      _setContextModes(e, t) {
        let n = e === this.document || e && this.treeAdapter.getNamespaceURI(e) === p.HTML;
        this.currentNotInHTML = !n, this.tokenizer.inForeignNode = !n && e !== void 0 && t !== void 0 && !this._isIntegrationPoint(t, e);
      }
      _switchToTextParsing(e, t) {
        this._insertElement(e, p.HTML), this.tokenizer.state = t, this.originalInsertionMode = this.insertionMode, this.insertionMode = h.TEXT;
      }
      switchToPlaintextParsing() {
        this.insertionMode = h.TEXT, this.originalInsertionMode = h.IN_BODY, this.tokenizer.state = Ae.PLAINTEXT;
      }
      _getAdjustedCurrentElement() {
        return this.openElements.stackTop === 0 && this.fragmentContext ? this.fragmentContext : this.openElements.current;
      }
      _findFormInFragmentContext() {
        let e = this.fragmentContext;
        for (; e;) {
          if (this.treeAdapter.getTagName(e) === d.FORM) {
            this.formElement = e;
            break;
          }
          e = this.treeAdapter.getParentNode(e);
        }
      }
      _initTokenizerForFragmentParsing() {
        if (!(!this.fragmentContext || this.treeAdapter.getNamespaceURI(this.fragmentContext) !== p.HTML)) switch (this.fragmentContextID) {
          case u.TITLE:
          case u.TEXTAREA:
            {
              this.tokenizer.state = Ae.RCDATA;
              break;
            }
          case u.STYLE:
          case u.XMP:
          case u.IFRAME:
          case u.NOEMBED:
          case u.NOFRAMES:
          case u.NOSCRIPT:
            {
              this.tokenizer.state = Ae.RAWTEXT;
              break;
            }
          case u.SCRIPT:
            {
              this.tokenizer.state = Ae.SCRIPT_DATA;
              break;
            }
          case u.PLAINTEXT:
            {
              this.tokenizer.state = Ae.PLAINTEXT;
              break;
            }
          default:
        }
      }
      _setDocumentType(e) {
        let t = e.name || "",
          n = e.publicId || "",
          r = e.systemId || "";
        if (this.treeAdapter.setDocumentType(this.document, t, n, r), e.location) {
          let a = this.treeAdapter.getChildNodes(this.document).find(i => this.treeAdapter.isDocumentTypeNode(i));
          a && this.treeAdapter.setNodeSourceCodeLocation(a, e.location);
        }
      }
      _attachElementToTree(e, t) {
        if (this.options.sourceCodeLocationInfo) {
          let n = t && Ye(L({}, t), {
            startTag: t
          });
          this.treeAdapter.setNodeSourceCodeLocation(e, n);
        }
        if (this._shouldFosterParentOnInsertion()) this._fosterParentElement(e);else {
          let n = this.openElements.currentTmplContentOrNode;
          this.treeAdapter.appendChild(n != null ? n : this.document, e);
        }
      }
      _appendElement(e, t) {
        let n = this.treeAdapter.createElement(e.tagName, t, e.attrs);
        this._attachElementToTree(n, e.location);
      }
      _insertElement(e, t) {
        let n = this.treeAdapter.createElement(e.tagName, t, e.attrs);
        this._attachElementToTree(n, e.location), this.openElements.push(n, e.tagID);
      }
      _insertFakeElement(e, t) {
        let n = this.treeAdapter.createElement(e, p.HTML, []);
        this._attachElementToTree(n, null), this.openElements.push(n, t);
      }
      _insertTemplate(e) {
        let t = this.treeAdapter.createElement(e.tagName, p.HTML, e.attrs),
          n = this.treeAdapter.createDocumentFragment();
        this.treeAdapter.setTemplateContent(t, n), this._attachElementToTree(t, e.location), this.openElements.push(t, e.tagID), this.options.sourceCodeLocationInfo && this.treeAdapter.setNodeSourceCodeLocation(n, null);
      }
      _insertFakeRootElement() {
        let e = this.treeAdapter.createElement(d.HTML, p.HTML, []);
        this.options.sourceCodeLocationInfo && this.treeAdapter.setNodeSourceCodeLocation(e, null), this.treeAdapter.appendChild(this.openElements.current, e), this.openElements.push(e, u.HTML);
      }
      _appendCommentNode(e, t) {
        let n = this.treeAdapter.createCommentNode(e.data);
        this.treeAdapter.appendChild(t, n), this.options.sourceCodeLocationInfo && this.treeAdapter.setNodeSourceCodeLocation(n, e.location);
      }
      _insertCharacters(e) {
        var t;
        let n, r;
        if (this._shouldFosterParentOnInsertion() ? (t = this._findFosterParentingLocation(), n = t.parent, r = t.beforeElement, r ? this.treeAdapter.insertTextBefore(n, e.chars, r) : this.treeAdapter.insertText(n, e.chars)) : (n = this.openElements.currentTmplContentOrNode, this.treeAdapter.insertText(n, e.chars)), !e.location) return;
        let a = this.treeAdapter.getChildNodes(n),
          i = r ? a.lastIndexOf(r) : a.length,
          s = a[i - 1];
        if (this.treeAdapter.getNodeSourceCodeLocation(s)) {
          let l = e.location,
            c = l.endLine,
            E = l.endCol,
            f = l.endOffset;
          this.treeAdapter.updateNodeSourceCodeLocation(s, {
            endLine: c,
            endCol: E,
            endOffset: f
          });
        } else this.options.sourceCodeLocationInfo && this.treeAdapter.setNodeSourceCodeLocation(s, e.location);
      }
      _adoptNodes(e, t) {
        for (let n = this.treeAdapter.getFirstChild(e); n; n = this.treeAdapter.getFirstChild(e)) this.treeAdapter.detachNode(n), this.treeAdapter.appendChild(t, n);
      }
      _setEndLocation(e, t) {
        if (this.treeAdapter.getNodeSourceCodeLocation(e) && t.location) {
          let n = t.location,
            r = this.treeAdapter.getTagName(e),
            a = t.type === v.END_TAG && r === t.tagName ? {
              endTag: L({}, n),
              endLine: n.endLine,
              endCol: n.endCol,
              endOffset: n.endOffset
            } : {
              endLine: n.startLine,
              endCol: n.startCol,
              endOffset: n.startOffset
            };
          this.treeAdapter.updateNodeSourceCodeLocation(e, a);
        }
      }
      shouldProcessStartTagTokenInForeignContent(e) {
        var t;
        if (!this.currentNotInHTML) return !1;
        let n, r;
        return this.openElements.stackTop === 0 && this.fragmentContext ? (n = this.fragmentContext, r = this.fragmentContextID) : (t = this.openElements, n = t.current, r = t.currentTagId), e.tagID === u.SVG && this.treeAdapter.getTagName(n) === d.ANNOTATION_XML && this.treeAdapter.getNamespaceURI(n) === p.MATHML ? !1 : this.tokenizer.inForeignNode || (e.tagID === u.MGLYPH || e.tagID === u.MALIGNMARK) && r !== void 0 && !this._isIntegrationPoint(r, n, p.HTML);
      }
      _processToken(e) {
        switch (e.type) {
          case v.CHARACTER:
            {
              this.onCharacter(e);
              break;
            }
          case v.NULL_CHARACTER:
            {
              this.onNullCharacter(e);
              break;
            }
          case v.COMMENT:
            {
              this.onComment(e);
              break;
            }
          case v.DOCTYPE:
            {
              this.onDoctype(e);
              break;
            }
          case v.START_TAG:
            {
              this._processStartTag(e);
              break;
            }
          case v.END_TAG:
            {
              this.onEndTag(e);
              break;
            }
          case v.EOF:
            {
              this.onEof(e);
              break;
            }
          case v.WHITESPACE_CHARACTER:
            {
              this.onWhitespaceCharacter(e);
              break;
            }
        }
      }
      _isIntegrationPoint(e, t, n) {
        let r = this.treeAdapter.getNamespaceURI(t),
          a = this.treeAdapter.getAttrList(t);
        return Du(e, r, a, n);
      }
      _reconstructActiveFormattingElements() {
        let e = this.activeFormattingElements.entries.length;
        if (e) {
          let t = this.activeFormattingElements.entries.findIndex(r => r.type === ge.Marker || this.openElements.contains(r.element)),
            n = t === -1 ? e - 1 : t - 1;
          for (let r = n; r >= 0; r--) {
            let a = this.activeFormattingElements.entries[r];
            this._insertElement(a.token, this.treeAdapter.getNamespaceURI(a.element)), a.element = this.openElements.current;
          }
        }
      }
      _closeTableCell() {
        this.openElements.generateImpliedEndTags(), this.openElements.popUntilTableCellPopped(), this.activeFormattingElements.clearToLastMarker(), this.insertionMode = h.IN_ROW;
      }
      _closePElement() {
        this.openElements.generateImpliedEndTagsWithExclusion(u.P), this.openElements.popUntilTagNamePopped(u.P);
      }
      _resetInsertionMode() {
        for (let e = this.openElements.stackTop; e >= 0; e--) switch (e === 0 && this.fragmentContext ? this.fragmentContextID : this.openElements.tagIDs[e]) {
          case u.TR:
            {
              this.insertionMode = h.IN_ROW;
              return;
            }
          case u.TBODY:
          case u.THEAD:
          case u.TFOOT:
            {
              this.insertionMode = h.IN_TABLE_BODY;
              return;
            }
          case u.CAPTION:
            {
              this.insertionMode = h.IN_CAPTION;
              return;
            }
          case u.COLGROUP:
            {
              this.insertionMode = h.IN_COLUMN_GROUP;
              return;
            }
          case u.TABLE:
            {
              this.insertionMode = h.IN_TABLE;
              return;
            }
          case u.BODY:
            {
              this.insertionMode = h.IN_BODY;
              return;
            }
          case u.FRAMESET:
            {
              this.insertionMode = h.IN_FRAMESET;
              return;
            }
          case u.SELECT:
            {
              this._resetInsertionModeForSelect(e);
              return;
            }
          case u.TEMPLATE:
            {
              this.insertionMode = this.tmplInsertionModeStack[0];
              return;
            }
          case u.HTML:
            {
              this.insertionMode = this.headElement ? h.AFTER_HEAD : h.BEFORE_HEAD;
              return;
            }
          case u.TD:
          case u.TH:
            {
              if (e > 0) {
                this.insertionMode = h.IN_CELL;
                return;
              }
              break;
            }
          case u.HEAD:
            {
              if (e > 0) {
                this.insertionMode = h.IN_HEAD;
                return;
              }
              break;
            }
        }
        this.insertionMode = h.IN_BODY;
      }
      _resetInsertionModeForSelect(e) {
        if (e > 0) for (let t = e - 1; t > 0; t--) {
          let n = this.openElements.tagIDs[t];
          if (n === u.TEMPLATE) break;
          if (n === u.TABLE) {
            this.insertionMode = h.IN_SELECT_IN_TABLE;
            return;
          }
        }
        this.insertionMode = h.IN_SELECT;
      }
      _isElementCausesFosterParenting(e) {
        return gu.has(e);
      }
      _shouldFosterParentOnInsertion() {
        return this.fosterParentingEnabled && this.openElements.currentTagId !== void 0 && this._isElementCausesFosterParenting(this.openElements.currentTagId);
      }
      _findFosterParentingLocation() {
        for (let e = this.openElements.stackTop; e >= 0; e--) {
          let t = this.openElements.items[e];
          switch (this.openElements.tagIDs[e]) {
            case u.TEMPLATE:
              {
                if (this.treeAdapter.getNamespaceURI(t) === p.HTML) return {
                  parent: this.treeAdapter.getTemplateContent(t),
                  beforeElement: null
                };
                break;
              }
            case u.TABLE:
              {
                let n = this.treeAdapter.getParentNode(t);
                return n ? {
                  parent: n,
                  beforeElement: t
                } : {
                  parent: this.openElements.items[e - 1],
                  beforeElement: null
                };
              }
            default:
          }
        }
        return {
          parent: this.openElements.items[0],
          beforeElement: null
        };
      }
      _fosterParentElement(e) {
        let t = this._findFosterParentingLocation();
        t.beforeElement ? this.treeAdapter.insertBefore(t.parent, e, t.beforeElement) : this.treeAdapter.appendChild(t.parent, e);
      }
      _isSpecialElement(e, t) {
        let n = this.treeAdapter.getNamespaceURI(e);
        return ru[n].has(t);
      }
      onCharacter(e) {
        if (this.skipNextNewLine = !1, this.tokenizer.inForeignNode) {
          Hc(this, e);
          return;
        }
        switch (this.insertionMode) {
          case h.INITIAL:
            {
              bt(this, e);
              break;
            }
          case h.BEFORE_HTML:
            {
              St(this, e);
              break;
            }
          case h.BEFORE_HEAD:
            {
              It(this, e);
              break;
            }
          case h.IN_HEAD:
            {
              vt(this, e);
              break;
            }
          case h.IN_HEAD_NO_SCRIPT:
            {
              Nt(this, e);
              break;
            }
          case h.AFTER_HEAD:
            {
              kt(this, e);
              break;
            }
          case h.IN_BODY:
          case h.IN_CAPTION:
          case h.IN_CELL:
          case h.IN_TEMPLATE:
            {
              yu(this, e);
              break;
            }
          case h.TEXT:
          case h.IN_SELECT:
          case h.IN_SELECT_IN_TABLE:
            {
              this._insertCharacters(e);
              break;
            }
          case h.IN_TABLE:
          case h.IN_TABLE_BODY:
          case h.IN_ROW:
            {
              f0(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ru(this, e);
              break;
            }
          case h.IN_COLUMN_GROUP:
            {
              An(this, e);
              break;
            }
          case h.AFTER_BODY:
            {
              hn(this, e);
              break;
            }
          case h.AFTER_AFTER_BODY:
            {
              dn(this, e);
              break;
            }
          default:
        }
      }
      onNullCharacter(e) {
        if (this.skipNextNewLine = !1, this.tokenizer.inForeignNode) {
          Mc(this, e);
          return;
        }
        switch (this.insertionMode) {
          case h.INITIAL:
            {
              bt(this, e);
              break;
            }
          case h.BEFORE_HTML:
            {
              St(this, e);
              break;
            }
          case h.BEFORE_HEAD:
            {
              It(this, e);
              break;
            }
          case h.IN_HEAD:
            {
              vt(this, e);
              break;
            }
          case h.IN_HEAD_NO_SCRIPT:
            {
              Nt(this, e);
              break;
            }
          case h.AFTER_HEAD:
            {
              kt(this, e);
              break;
            }
          case h.TEXT:
            {
              this._insertCharacters(e);
              break;
            }
          case h.IN_TABLE:
          case h.IN_TABLE_BODY:
          case h.IN_ROW:
            {
              f0(this, e);
              break;
            }
          case h.IN_COLUMN_GROUP:
            {
              An(this, e);
              break;
            }
          case h.AFTER_BODY:
            {
              hn(this, e);
              break;
            }
          case h.AFTER_AFTER_BODY:
            {
              dn(this, e);
              break;
            }
          default:
        }
      }
      onComment(e) {
        if (this.skipNextNewLine = !1, this.currentNotInHTML) {
          C0(this, e);
          return;
        }
        switch (this.insertionMode) {
          case h.INITIAL:
          case h.BEFORE_HTML:
          case h.BEFORE_HEAD:
          case h.IN_HEAD:
          case h.IN_HEAD_NO_SCRIPT:
          case h.AFTER_HEAD:
          case h.IN_BODY:
          case h.IN_TABLE:
          case h.IN_CAPTION:
          case h.IN_COLUMN_GROUP:
          case h.IN_TABLE_BODY:
          case h.IN_ROW:
          case h.IN_CELL:
          case h.IN_SELECT:
          case h.IN_SELECT_IN_TABLE:
          case h.IN_TEMPLATE:
          case h.IN_FRAMESET:
          case h.AFTER_FRAMESET:
            {
              C0(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ot(this, e);
              break;
            }
          case h.AFTER_BODY:
            {
              CA(this, e);
              break;
            }
          case h.AFTER_AFTER_BODY:
          case h.AFTER_AFTER_FRAMESET:
            {
              pA(this, e);
              break;
            }
          default:
        }
      }
      onDoctype(e) {
        switch (this.skipNextNewLine = !1, this.insertionMode) {
          case h.INITIAL:
            {
              fA(this, e);
              break;
            }
          case h.BEFORE_HEAD:
          case h.IN_HEAD:
          case h.IN_HEAD_NO_SCRIPT:
          case h.AFTER_HEAD:
            {
              this._err(e, C.misplacedDoctype);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ot(this, e);
              break;
            }
          default:
        }
      }
      onStartTag(e) {
        this.skipNextNewLine = !1, this.currentToken = e, this._processStartTag(e), e.selfClosing && !e.ackSelfClosing && this._err(e, C.nonVoidHtmlElementStartTagWithTrailingSolidus);
      }
      _processStartTag(e) {
        this.shouldProcessStartTagTokenInForeignContent(e) ? Pc(this, e) : this._startTagOutsideForeignContent(e);
      }
      _startTagOutsideForeignContent(e) {
        switch (this.insertionMode) {
          case h.INITIAL:
            {
              bt(this, e);
              break;
            }
          case h.BEFORE_HTML:
            {
              xA(this, e);
              break;
            }
          case h.BEFORE_HEAD:
            {
              mA(this, e);
              break;
            }
          case h.IN_HEAD:
            {
              xe(this, e);
              break;
            }
          case h.IN_HEAD_NO_SCRIPT:
            {
              TA(this, e);
              break;
            }
          case h.AFTER_HEAD:
            {
              _A(this, e);
              break;
            }
          case h.IN_BODY:
            {
              ne(this, e);
              break;
            }
          case h.IN_TABLE:
            {
              ct(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ot(this, e);
              break;
            }
          case h.IN_CAPTION:
            {
              Dc(this, e);
              break;
            }
          case h.IN_COLUMN_GROUP:
            {
              x0(this, e);
              break;
            }
          case h.IN_TABLE_BODY:
            {
              cn(this, e);
              break;
            }
          case h.IN_ROW:
            {
              ln(this, e);
              break;
            }
          case h.IN_CELL:
            {
              Fc(this, e);
              break;
            }
          case h.IN_SELECT:
            {
              Mu(this, e);
              break;
            }
          case h.IN_SELECT_IN_TABLE:
            {
              yc(this, e);
              break;
            }
          case h.IN_TEMPLATE:
            {
              Sc(this, e);
              break;
            }
          case h.AFTER_BODY:
            {
              vc(this, e);
              break;
            }
          case h.IN_FRAMESET:
            {
              Nc(this, e);
              break;
            }
          case h.AFTER_FRAMESET:
            {
              wc(this, e);
              break;
            }
          case h.AFTER_AFTER_BODY:
            {
              Oc(this, e);
              break;
            }
          case h.AFTER_AFTER_FRAMESET:
            {
              Lc(this, e);
              break;
            }
          default:
        }
      }
      onEndTag(e) {
        this.skipNextNewLine = !1, this.currentToken = e, this.currentNotInHTML ? Uc(this, e) : this._endTagOutsideForeignContent(e);
      }
      _endTagOutsideForeignContent(e) {
        switch (this.insertionMode) {
          case h.INITIAL:
            {
              bt(this, e);
              break;
            }
          case h.BEFORE_HTML:
            {
              BA(this, e);
              break;
            }
          case h.BEFORE_HEAD:
            {
              DA(this, e);
              break;
            }
          case h.IN_HEAD:
            {
              gA(this, e);
              break;
            }
          case h.IN_HEAD_NO_SCRIPT:
            {
              FA(this, e);
              break;
            }
          case h.AFTER_HEAD:
            {
              yA(this, e);
              break;
            }
          case h.IN_BODY:
            {
              on(this, e);
              break;
            }
          case h.TEXT:
            {
              lc(this, e);
              break;
            }
          case h.IN_TABLE:
            {
              wt(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ot(this, e);
              break;
            }
          case h.IN_CAPTION:
            {
              gc(this, e);
              break;
            }
          case h.IN_COLUMN_GROUP:
            {
              Tc(this, e);
              break;
            }
          case h.IN_TABLE_BODY:
            {
              B0(this, e);
              break;
            }
          case h.IN_ROW:
            {
              Lu(this, e);
              break;
            }
          case h.IN_CELL:
            {
              _c(this, e);
              break;
            }
          case h.IN_SELECT:
            {
              Hu(this, e);
              break;
            }
          case h.IN_SELECT_IN_TABLE:
            {
              bc(this, e);
              break;
            }
          case h.IN_TEMPLATE:
            {
              Ic(this, e);
              break;
            }
          case h.AFTER_BODY:
            {
              Uu(this, e);
              break;
            }
          case h.IN_FRAMESET:
            {
              kc(this, e);
              break;
            }
          case h.AFTER_FRAMESET:
            {
              Rc(this, e);
              break;
            }
          case h.AFTER_AFTER_BODY:
            {
              dn(this, e);
              break;
            }
          default:
        }
      }
      onEof(e) {
        switch (this.insertionMode) {
          case h.INITIAL:
            {
              bt(this, e);
              break;
            }
          case h.BEFORE_HTML:
            {
              St(this, e);
              break;
            }
          case h.BEFORE_HEAD:
            {
              It(this, e);
              break;
            }
          case h.IN_HEAD:
            {
              vt(this, e);
              break;
            }
          case h.IN_HEAD_NO_SCRIPT:
            {
              Nt(this, e);
              break;
            }
          case h.AFTER_HEAD:
            {
              kt(this, e);
              break;
            }
          case h.IN_BODY:
          case h.IN_TABLE:
          case h.IN_CAPTION:
          case h.IN_COLUMN_GROUP:
          case h.IN_TABLE_BODY:
          case h.IN_ROW:
          case h.IN_CELL:
          case h.IN_SELECT:
          case h.IN_SELECT_IN_TABLE:
            {
              ku(this, e);
              break;
            }
          case h.TEXT:
            {
              hc(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              Ot(this, e);
              break;
            }
          case h.IN_TEMPLATE:
            {
              Pu(this, e);
              break;
            }
          case h.AFTER_BODY:
          case h.IN_FRAMESET:
          case h.AFTER_FRAMESET:
          case h.AFTER_AFTER_BODY:
          case h.AFTER_AFTER_FRAMESET:
            {
              p0(this, e);
              break;
            }
          default:
        }
      }
      onWhitespaceCharacter(e) {
        if (this.skipNextNewLine && (this.skipNextNewLine = !1, e.chars.charCodeAt(0) === o.LINE_FEED)) {
          if (e.chars.length === 1) return;
          e.chars = e.chars.substr(1);
        }
        if (this.tokenizer.inForeignNode) {
          this._insertCharacters(e);
          return;
        }
        switch (this.insertionMode) {
          case h.IN_HEAD:
          case h.IN_HEAD_NO_SCRIPT:
          case h.AFTER_HEAD:
          case h.TEXT:
          case h.IN_COLUMN_GROUP:
          case h.IN_SELECT:
          case h.IN_SELECT_IN_TABLE:
          case h.IN_FRAMESET:
          case h.AFTER_FRAMESET:
            {
              this._insertCharacters(e);
              break;
            }
          case h.IN_BODY:
          case h.IN_CAPTION:
          case h.IN_CELL:
          case h.IN_TEMPLATE:
          case h.AFTER_BODY:
          case h.AFTER_AFTER_BODY:
          case h.AFTER_AFTER_FRAMESET:
            {
              _u(this, e);
              break;
            }
          case h.IN_TABLE:
          case h.IN_TABLE_BODY:
          case h.IN_ROW:
            {
              f0(this, e);
              break;
            }
          case h.IN_TABLE_TEXT:
            {
              wu(this, e);
              break;
            }
          default:
        }
      }
    };
  function AA(e, t) {
    let n = e.activeFormattingElements.getElementEntryInScopeWithTagName(t.tagName);
    return n ? e.openElements.contains(n.element) ? e.openElements.hasInScope(t.tagID) || (n = null) : (e.activeFormattingElements.removeEntry(n), n = null) : Nu(e, t), n;
  }
  function cA(e, t) {
    let n = null,
      r = e.openElements.stackTop;
    for (; r >= 0; r--) {
      let a = e.openElements.items[r];
      if (a === t.element) break;
      e._isSpecialElement(a, e.openElements.tagIDs[r]) && (n = a);
    }
    return n || (e.openElements.shortenToLength(Math.max(r, 0)), e.activeFormattingElements.removeEntry(t)), n;
  }
  function lA(e, t, n) {
    let r = t,
      a = e.openElements.getCommonAncestor(t);
    for (let i = 0, s = a; s !== n; i++, s = a) {
      a = e.openElements.getCommonAncestor(s);
      let l = e.activeFormattingElements.getElementEntry(s),
        c = l && i >= sA;
      !l || c ? (c && e.activeFormattingElements.removeEntry(l), e.openElements.remove(s)) : (s = hA(e, l), r === t && (e.activeFormattingElements.bookmark = l), e.treeAdapter.detachNode(r), e.treeAdapter.appendChild(s, r), r = s);
    }
    return r;
  }
  function hA(e, t) {
    let n = e.treeAdapter.getNamespaceURI(t.element),
      r = e.treeAdapter.createElement(t.token.tagName, n, t.token.attrs);
    return e.openElements.replace(t.element, r), t.element = r, r;
  }
  function dA(e, t, n) {
    let r = e.treeAdapter.getTagName(t),
      a = _t(r);
    if (e._isElementCausesFosterParenting(a)) e._fosterParentElement(n);else {
      let i = e.treeAdapter.getNamespaceURI(t);
      a === u.TEMPLATE && i === p.HTML && (t = e.treeAdapter.getTemplateContent(t)), e.treeAdapter.appendChild(t, n);
    }
  }
  function EA(e, t, n) {
    let r = e.treeAdapter.getNamespaceURI(n.element),
      a = n.token,
      i = e.treeAdapter.createElement(a.tagName, r, a.attrs);
    e._adoptNodes(t, i), e.treeAdapter.appendChild(t, i), e.activeFormattingElements.insertElementAfterBookmark(i, a), e.activeFormattingElements.removeEntry(n), e.openElements.remove(n.element), e.openElements.insertAfter(t, i, a.tagID);
  }
  function E0(e, t) {
    for (let n = 0; n < iA; n++) {
      let r = AA(e, t);
      if (!r) break;
      let a = cA(e, r);
      if (!a) break;
      e.activeFormattingElements.bookmark = r;
      let i = lA(e, a, r.element),
        s = e.openElements.getCommonAncestor(r.element);
      e.treeAdapter.detachNode(i), s && dA(e, s, i), EA(e, a, r);
    }
  }
  function C0(e, t) {
    e._appendCommentNode(t, e.openElements.currentTmplContentOrNode);
  }
  function CA(e, t) {
    e._appendCommentNode(t, e.openElements.items[0]);
  }
  function pA(e, t) {
    e._appendCommentNode(t, e.document);
  }
  function p0(e, t) {
    if (e.stopped = !0, t.location) {
      let n = e.fragmentContext ? 0 : 2;
      for (let r = e.openElements.stackTop; r >= n; r--) e._setEndLocation(e.openElements.items[r], t);
      if (!e.fragmentContext && e.openElements.stackTop >= 0) {
        let r = e.openElements.items[0],
          a = e.treeAdapter.getNodeSourceCodeLocation(r);
        if (a && !a.endTag && (e._setEndLocation(r, t), e.openElements.stackTop >= 1)) {
          let i = e.openElements.items[1],
            s = e.treeAdapter.getNodeSourceCodeLocation(i);
          s && !s.endTag && e._setEndLocation(i, t);
        }
      }
    }
  }
  function fA(e, t) {
    e._setDocumentType(t);
    let n = t.forceQuirks ? de.QUIRKS : jo(t);
    Vo(t) || e._err(t, C.nonConformingDoctype), e.treeAdapter.setDocumentMode(e.document, n), e.insertionMode = h.BEFORE_HTML;
  }
  function bt(e, t) {
    e._err(t, C.missingDoctype, !0), e.treeAdapter.setDocumentMode(e.document, de.QUIRKS), e.insertionMode = h.BEFORE_HTML, e._processToken(t);
  }
  function xA(e, t) {
    t.tagID === u.HTML ? (e._insertElement(t, p.HTML), e.insertionMode = h.BEFORE_HEAD) : St(e, t);
  }
  function BA(e, t) {
    let n = t.tagID;
    (n === u.HTML || n === u.HEAD || n === u.BODY || n === u.BR) && St(e, t);
  }
  function St(e, t) {
    e._insertFakeRootElement(), e.insertionMode = h.BEFORE_HEAD, e._processToken(t);
  }
  function mA(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.HEAD:
        {
          e._insertElement(t, p.HTML), e.headElement = e.openElements.current, e.insertionMode = h.IN_HEAD;
          break;
        }
      default:
        It(e, t);
    }
  }
  function DA(e, t) {
    let n = t.tagID;
    n === u.HEAD || n === u.BODY || n === u.HTML || n === u.BR ? It(e, t) : e._err(t, C.endTagWithoutMatchingOpenElement);
  }
  function It(e, t) {
    e._insertFakeElement(d.HEAD, u.HEAD), e.headElement = e.openElements.current, e.insertionMode = h.IN_HEAD, e._processToken(t);
  }
  function xe(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.BASE:
      case u.BASEFONT:
      case u.BGSOUND:
      case u.LINK:
      case u.META:
        {
          e._appendElement(t, p.HTML), t.ackSelfClosing = !0;
          break;
        }
      case u.TITLE:
        {
          e._switchToTextParsing(t, Ae.RCDATA);
          break;
        }
      case u.NOSCRIPT:
        {
          e.options.scriptingEnabled ? e._switchToTextParsing(t, Ae.RAWTEXT) : (e._insertElement(t, p.HTML), e.insertionMode = h.IN_HEAD_NO_SCRIPT);
          break;
        }
      case u.NOFRAMES:
      case u.STYLE:
        {
          e._switchToTextParsing(t, Ae.RAWTEXT);
          break;
        }
      case u.SCRIPT:
        {
          e._switchToTextParsing(t, Ae.SCRIPT_DATA);
          break;
        }
      case u.TEMPLATE:
        {
          e._insertTemplate(t), e.activeFormattingElements.insertMarker(), e.framesetOk = !1, e.insertionMode = h.IN_TEMPLATE, e.tmplInsertionModeStack.unshift(h.IN_TEMPLATE);
          break;
        }
      case u.HEAD:
        {
          e._err(t, C.misplacedStartTagForHeadElement);
          break;
        }
      default:
        vt(e, t);
    }
  }
  function gA(e, t) {
    switch (t.tagID) {
      case u.HEAD:
        {
          e.openElements.pop(), e.insertionMode = h.AFTER_HEAD;
          break;
        }
      case u.BODY:
      case u.BR:
      case u.HTML:
        {
          vt(e, t);
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      default:
        e._err(t, C.endTagWithoutMatchingOpenElement);
    }
  }
  function tt(e, t) {
    e.openElements.tmplCount > 0 ? (e.openElements.generateImpliedEndTagsThoroughly(), e.openElements.currentTagId !== u.TEMPLATE && e._err(t, C.closingOfElementWithOpenChildElements), e.openElements.popUntilTagNamePopped(u.TEMPLATE), e.activeFormattingElements.clearToLastMarker(), e.tmplInsertionModeStack.shift(), e._resetInsertionMode()) : e._err(t, C.endTagWithoutMatchingOpenElement);
  }
  function vt(e, t) {
    e.openElements.pop(), e.insertionMode = h.AFTER_HEAD, e._processToken(t);
  }
  function TA(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.BASEFONT:
      case u.BGSOUND:
      case u.HEAD:
      case u.LINK:
      case u.META:
      case u.NOFRAMES:
      case u.STYLE:
        {
          xe(e, t);
          break;
        }
      case u.NOSCRIPT:
        {
          e._err(t, C.nestedNoscriptInHead);
          break;
        }
      default:
        Nt(e, t);
    }
  }
  function FA(e, t) {
    switch (t.tagID) {
      case u.NOSCRIPT:
        {
          e.openElements.pop(), e.insertionMode = h.IN_HEAD;
          break;
        }
      case u.BR:
        {
          Nt(e, t);
          break;
        }
      default:
        e._err(t, C.endTagWithoutMatchingOpenElement);
    }
  }
  function Nt(e, t) {
    let n = t.type === v.EOF ? C.openElementsLeftAfterEof : C.disallowedContentInNoscriptInHead;
    e._err(t, n), e.openElements.pop(), e.insertionMode = h.IN_HEAD, e._processToken(t);
  }
  function _A(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.BODY:
        {
          e._insertElement(t, p.HTML), e.framesetOk = !1, e.insertionMode = h.IN_BODY;
          break;
        }
      case u.FRAMESET:
        {
          e._insertElement(t, p.HTML), e.insertionMode = h.IN_FRAMESET;
          break;
        }
      case u.BASE:
      case u.BASEFONT:
      case u.BGSOUND:
      case u.LINK:
      case u.META:
      case u.NOFRAMES:
      case u.SCRIPT:
      case u.STYLE:
      case u.TEMPLATE:
      case u.TITLE:
        {
          e._err(t, C.abandonedHeadElementChild), e.openElements.push(e.headElement, u.HEAD), xe(e, t), e.openElements.remove(e.headElement);
          break;
        }
      case u.HEAD:
        {
          e._err(t, C.misplacedStartTagForHeadElement);
          break;
        }
      default:
        kt(e, t);
    }
  }
  function yA(e, t) {
    switch (t.tagID) {
      case u.BODY:
      case u.HTML:
      case u.BR:
        {
          kt(e, t);
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      default:
        e._err(t, C.endTagWithoutMatchingOpenElement);
    }
  }
  function kt(e, t) {
    e._insertFakeElement(d.BODY, u.BODY), e.insertionMode = h.IN_BODY, sn(e, t);
  }
  function sn(e, t) {
    switch (t.type) {
      case v.CHARACTER:
        {
          yu(e, t);
          break;
        }
      case v.WHITESPACE_CHARACTER:
        {
          _u(e, t);
          break;
        }
      case v.COMMENT:
        {
          C0(e, t);
          break;
        }
      case v.START_TAG:
        {
          ne(e, t);
          break;
        }
      case v.END_TAG:
        {
          on(e, t);
          break;
        }
      case v.EOF:
        {
          ku(e, t);
          break;
        }
      default:
    }
  }
  function _u(e, t) {
    e._reconstructActiveFormattingElements(), e._insertCharacters(t);
  }
  function yu(e, t) {
    e._reconstructActiveFormattingElements(), e._insertCharacters(t), e.framesetOk = !1;
  }
  function bA(e, t) {
    e.openElements.tmplCount === 0 && e.treeAdapter.adoptAttributes(e.openElements.items[0], t.attrs);
  }
  function SA(e, t) {
    let n = e.openElements.tryPeekProperlyNestedBodyElement();
    n && e.openElements.tmplCount === 0 && (e.framesetOk = !1, e.treeAdapter.adoptAttributes(n, t.attrs));
  }
  function IA(e, t) {
    let n = e.openElements.tryPeekProperlyNestedBodyElement();
    e.framesetOk && n && (e.treeAdapter.detachNode(n), e.openElements.popAllUpToHtmlElement(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_FRAMESET);
  }
  function vA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML);
  }
  function NA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e.openElements.currentTagId !== void 0 && nn.has(e.openElements.currentTagId) && e.openElements.pop(), e._insertElement(t, p.HTML);
  }
  function kA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML), e.skipNextNewLine = !0, e.framesetOk = !1;
  }
  function wA(e, t) {
    let n = e.openElements.tmplCount > 0;
    (!e.formElement || n) && (e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML), n || (e.formElement = e.openElements.current));
  }
  function RA(e, t) {
    e.framesetOk = !1;
    let n = t.tagID;
    for (let r = e.openElements.stackTop; r >= 0; r--) {
      let a = e.openElements.tagIDs[r];
      if (n === u.LI && a === u.LI || (n === u.DD || n === u.DT) && (a === u.DD || a === u.DT)) {
        e.openElements.generateImpliedEndTagsWithExclusion(a), e.openElements.popUntilTagNamePopped(a);
        break;
      }
      if (a !== u.ADDRESS && a !== u.DIV && a !== u.P && e._isSpecialElement(e.openElements.items[r], a)) break;
    }
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML);
  }
  function OA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML), e.tokenizer.state = Ae.PLAINTEXT;
  }
  function LA(e, t) {
    e.openElements.hasInScope(u.BUTTON) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(u.BUTTON)), e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML), e.framesetOk = !1;
  }
  function MA(e, t) {
    let n = e.activeFormattingElements.getElementEntryInScopeWithTagName(d.A);
    n && (E0(e, t), e.openElements.remove(n.element), e.activeFormattingElements.removeEntry(n)), e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML), e.activeFormattingElements.pushElement(e.openElements.current, t);
  }
  function HA(e, t) {
    e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML), e.activeFormattingElements.pushElement(e.openElements.current, t);
  }
  function PA(e, t) {
    e._reconstructActiveFormattingElements(), e.openElements.hasInScope(u.NOBR) && (E0(e, t), e._reconstructActiveFormattingElements()), e._insertElement(t, p.HTML), e.activeFormattingElements.pushElement(e.openElements.current, t);
  }
  function UA(e, t) {
    e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML), e.activeFormattingElements.insertMarker(), e.framesetOk = !1;
  }
  function WA(e, t) {
    e.treeAdapter.getDocumentMode(e.document) !== de.QUIRKS && e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._insertElement(t, p.HTML), e.framesetOk = !1, e.insertionMode = h.IN_TABLE;
  }
  function bu(e, t) {
    e._reconstructActiveFormattingElements(), e._appendElement(t, p.HTML), e.framesetOk = !1, t.ackSelfClosing = !0;
  }
  function Su(e) {
    let t = A0(e, We.TYPE);
    return t != null && t.toLowerCase() === aA;
  }
  function GA(e, t) {
    e._reconstructActiveFormattingElements(), e._appendElement(t, p.HTML), Su(t) || (e.framesetOk = !1), t.ackSelfClosing = !0;
  }
  function KA(e, t) {
    e._appendElement(t, p.HTML), t.ackSelfClosing = !0;
  }
  function QA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._appendElement(t, p.HTML), e.framesetOk = !1, t.ackSelfClosing = !0;
  }
  function YA(e, t) {
    t.tagName = d.IMG, t.tagID = u.IMG, bu(e, t);
  }
  function XA(e, t) {
    e._insertElement(t, p.HTML), e.skipNextNewLine = !0, e.tokenizer.state = Ae.RCDATA, e.originalInsertionMode = e.insertionMode, e.framesetOk = !1, e.insertionMode = h.TEXT;
  }
  function JA(e, t) {
    e.openElements.hasInButtonScope(u.P) && e._closePElement(), e._reconstructActiveFormattingElements(), e.framesetOk = !1, e._switchToTextParsing(t, Ae.RAWTEXT);
  }
  function qA(e, t) {
    e.framesetOk = !1, e._switchToTextParsing(t, Ae.RAWTEXT);
  }
  function Iu(e, t) {
    e._switchToTextParsing(t, Ae.RAWTEXT);
  }
  function VA(e, t) {
    e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML), e.framesetOk = !1, e.insertionMode = e.insertionMode === h.IN_TABLE || e.insertionMode === h.IN_CAPTION || e.insertionMode === h.IN_TABLE_BODY || e.insertionMode === h.IN_ROW || e.insertionMode === h.IN_CELL ? h.IN_SELECT_IN_TABLE : h.IN_SELECT;
  }
  function jA(e, t) {
    e.openElements.currentTagId === u.OPTION && e.openElements.pop(), e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML);
  }
  function ZA(e, t) {
    e.openElements.hasInScope(u.RUBY) && e.openElements.generateImpliedEndTags(), e._insertElement(t, p.HTML);
  }
  function zA(e, t) {
    e.openElements.hasInScope(u.RUBY) && e.openElements.generateImpliedEndTagsWithExclusion(u.RTC), e._insertElement(t, p.HTML);
  }
  function $A(e, t) {
    e._reconstructActiveFormattingElements(), h0(t), an(t), t.selfClosing ? e._appendElement(t, p.MATHML) : e._insertElement(t, p.MATHML), t.ackSelfClosing = !0;
  }
  function ec(e, t) {
    e._reconstructActiveFormattingElements(), d0(t), an(t), t.selfClosing ? e._appendElement(t, p.SVG) : e._insertElement(t, p.SVG), t.ackSelfClosing = !0;
  }
  function vu(e, t) {
    e._reconstructActiveFormattingElements(), e._insertElement(t, p.HTML);
  }
  function ne(e, t) {
    switch (t.tagID) {
      case u.I:
      case u.S:
      case u.B:
      case u.U:
      case u.EM:
      case u.TT:
      case u.BIG:
      case u.CODE:
      case u.FONT:
      case u.SMALL:
      case u.STRIKE:
      case u.STRONG:
        {
          HA(e, t);
          break;
        }
      case u.A:
        {
          MA(e, t);
          break;
        }
      case u.H1:
      case u.H2:
      case u.H3:
      case u.H4:
      case u.H5:
      case u.H6:
        {
          NA(e, t);
          break;
        }
      case u.P:
      case u.DL:
      case u.OL:
      case u.UL:
      case u.DIV:
      case u.DIR:
      case u.NAV:
      case u.MAIN:
      case u.MENU:
      case u.ASIDE:
      case u.CENTER:
      case u.FIGURE:
      case u.FOOTER:
      case u.HEADER:
      case u.HGROUP:
      case u.DIALOG:
      case u.DETAILS:
      case u.ADDRESS:
      case u.ARTICLE:
      case u.SEARCH:
      case u.SECTION:
      case u.SUMMARY:
      case u.FIELDSET:
      case u.BLOCKQUOTE:
      case u.FIGCAPTION:
        {
          vA(e, t);
          break;
        }
      case u.LI:
      case u.DD:
      case u.DT:
        {
          RA(e, t);
          break;
        }
      case u.BR:
      case u.IMG:
      case u.WBR:
      case u.AREA:
      case u.EMBED:
      case u.KEYGEN:
        {
          bu(e, t);
          break;
        }
      case u.HR:
        {
          QA(e, t);
          break;
        }
      case u.RB:
      case u.RTC:
        {
          ZA(e, t);
          break;
        }
      case u.RT:
      case u.RP:
        {
          zA(e, t);
          break;
        }
      case u.PRE:
      case u.LISTING:
        {
          kA(e, t);
          break;
        }
      case u.XMP:
        {
          JA(e, t);
          break;
        }
      case u.SVG:
        {
          ec(e, t);
          break;
        }
      case u.HTML:
        {
          bA(e, t);
          break;
        }
      case u.BASE:
      case u.LINK:
      case u.META:
      case u.STYLE:
      case u.TITLE:
      case u.SCRIPT:
      case u.BGSOUND:
      case u.BASEFONT:
      case u.TEMPLATE:
        {
          xe(e, t);
          break;
        }
      case u.BODY:
        {
          SA(e, t);
          break;
        }
      case u.FORM:
        {
          wA(e, t);
          break;
        }
      case u.NOBR:
        {
          PA(e, t);
          break;
        }
      case u.MATH:
        {
          $A(e, t);
          break;
        }
      case u.TABLE:
        {
          WA(e, t);
          break;
        }
      case u.INPUT:
        {
          GA(e, t);
          break;
        }
      case u.PARAM:
      case u.TRACK:
      case u.SOURCE:
        {
          KA(e, t);
          break;
        }
      case u.IMAGE:
        {
          YA(e, t);
          break;
        }
      case u.BUTTON:
        {
          LA(e, t);
          break;
        }
      case u.APPLET:
      case u.OBJECT:
      case u.MARQUEE:
        {
          UA(e, t);
          break;
        }
      case u.IFRAME:
        {
          qA(e, t);
          break;
        }
      case u.SELECT:
        {
          VA(e, t);
          break;
        }
      case u.OPTION:
      case u.OPTGROUP:
        {
          jA(e, t);
          break;
        }
      case u.NOEMBED:
      case u.NOFRAMES:
        {
          Iu(e, t);
          break;
        }
      case u.FRAMESET:
        {
          IA(e, t);
          break;
        }
      case u.TEXTAREA:
        {
          XA(e, t);
          break;
        }
      case u.NOSCRIPT:
        {
          e.options.scriptingEnabled ? Iu(e, t) : vu(e, t);
          break;
        }
      case u.PLAINTEXT:
        {
          OA(e, t);
          break;
        }
      case u.COL:
      case u.TH:
      case u.TD:
      case u.TR:
      case u.HEAD:
      case u.FRAME:
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
      case u.CAPTION:
      case u.COLGROUP:
        break;
      default:
        vu(e, t);
    }
  }
  function tc(e, t) {
    if (e.openElements.hasInScope(u.BODY) && (e.insertionMode = h.AFTER_BODY, e.options.sourceCodeLocationInfo)) {
      let n = e.openElements.tryPeekProperlyNestedBodyElement();
      n && e._setEndLocation(n, t);
    }
  }
  function nc(e, t) {
    e.openElements.hasInScope(u.BODY) && (e.insertionMode = h.AFTER_BODY, Uu(e, t));
  }
  function rc(e, t) {
    let n = t.tagID;
    e.openElements.hasInScope(n) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(n));
  }
  function uc(e) {
    let t = e.openElements.tmplCount > 0,
      n = e.formElement;
    t || (e.formElement = null), (n || t) && e.openElements.hasInScope(u.FORM) && (e.openElements.generateImpliedEndTags(), t ? e.openElements.popUntilTagNamePopped(u.FORM) : n && e.openElements.remove(n));
  }
  function ac(e) {
    e.openElements.hasInButtonScope(u.P) || e._insertFakeElement(d.P, u.P), e._closePElement();
  }
  function ic(e) {
    e.openElements.hasInListItemScope(u.LI) && (e.openElements.generateImpliedEndTagsWithExclusion(u.LI), e.openElements.popUntilTagNamePopped(u.LI));
  }
  function sc(e, t) {
    let n = t.tagID;
    e.openElements.hasInScope(n) && (e.openElements.generateImpliedEndTagsWithExclusion(n), e.openElements.popUntilTagNamePopped(n));
  }
  function oc(e) {
    e.openElements.hasNumberedHeaderInScope() && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilNumberedHeaderPopped());
  }
  function Ac(e, t) {
    let n = t.tagID;
    e.openElements.hasInScope(n) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(n), e.activeFormattingElements.clearToLastMarker());
  }
  function cc(e) {
    e._reconstructActiveFormattingElements(), e._insertFakeElement(d.BR, u.BR), e.openElements.pop(), e.framesetOk = !1;
  }
  function Nu(e, t) {
    let n = t.tagName,
      r = t.tagID;
    for (let a = e.openElements.stackTop; a > 0; a--) {
      let i = e.openElements.items[a],
        s = e.openElements.tagIDs[a];
      if (r === s && (r !== u.UNKNOWN || e.treeAdapter.getTagName(i) === n)) {
        e.openElements.generateImpliedEndTagsWithExclusion(r), e.openElements.stackTop >= a && e.openElements.shortenToLength(a);
        break;
      }
      if (e._isSpecialElement(i, s)) break;
    }
  }
  function on(e, t) {
    switch (t.tagID) {
      case u.A:
      case u.B:
      case u.I:
      case u.S:
      case u.U:
      case u.EM:
      case u.TT:
      case u.BIG:
      case u.CODE:
      case u.FONT:
      case u.NOBR:
      case u.SMALL:
      case u.STRIKE:
      case u.STRONG:
        {
          E0(e, t);
          break;
        }
      case u.P:
        {
          ac(e);
          break;
        }
      case u.DL:
      case u.UL:
      case u.OL:
      case u.DIR:
      case u.DIV:
      case u.NAV:
      case u.PRE:
      case u.MAIN:
      case u.MENU:
      case u.ASIDE:
      case u.BUTTON:
      case u.CENTER:
      case u.FIGURE:
      case u.FOOTER:
      case u.HEADER:
      case u.HGROUP:
      case u.DIALOG:
      case u.ADDRESS:
      case u.ARTICLE:
      case u.DETAILS:
      case u.SEARCH:
      case u.SECTION:
      case u.SUMMARY:
      case u.LISTING:
      case u.FIELDSET:
      case u.BLOCKQUOTE:
      case u.FIGCAPTION:
        {
          rc(e, t);
          break;
        }
      case u.LI:
        {
          ic(e);
          break;
        }
      case u.DD:
      case u.DT:
        {
          sc(e, t);
          break;
        }
      case u.H1:
      case u.H2:
      case u.H3:
      case u.H4:
      case u.H5:
      case u.H6:
        {
          oc(e);
          break;
        }
      case u.BR:
        {
          cc(e);
          break;
        }
      case u.BODY:
        {
          tc(e, t);
          break;
        }
      case u.HTML:
        {
          nc(e, t);
          break;
        }
      case u.FORM:
        {
          uc(e);
          break;
        }
      case u.APPLET:
      case u.OBJECT:
      case u.MARQUEE:
        {
          Ac(e, t);
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      default:
        Nu(e, t);
    }
  }
  function ku(e, t) {
    e.tmplInsertionModeStack.length > 0 ? Pu(e, t) : p0(e, t);
  }
  function lc(e, t) {
    var n;
    t.tagID === u.SCRIPT && ((n = e.scriptHandler) === null || n === void 0 || n.call(e, e.openElements.current)), e.openElements.pop(), e.insertionMode = e.originalInsertionMode;
  }
  function hc(e, t) {
    e._err(t, C.eofInElementThatCanContainOnlyText), e.openElements.pop(), e.insertionMode = e.originalInsertionMode, e.onEof(t);
  }
  function f0(e, t) {
    if (e.openElements.currentTagId !== void 0 && gu.has(e.openElements.currentTagId)) switch (e.pendingCharacterTokens.length = 0, e.hasNonWhitespacePendingCharacterToken = !1, e.originalInsertionMode = e.insertionMode, e.insertionMode = h.IN_TABLE_TEXT, t.type) {
      case v.CHARACTER:
        {
          Ru(e, t);
          break;
        }
      case v.WHITESPACE_CHARACTER:
        {
          wu(e, t);
          break;
        }
    } else Rt(e, t);
  }
  function dc(e, t) {
    e.openElements.clearBackToTableContext(), e.activeFormattingElements.insertMarker(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_CAPTION;
  }
  function Ec(e, t) {
    e.openElements.clearBackToTableContext(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_COLUMN_GROUP;
  }
  function Cc(e, t) {
    e.openElements.clearBackToTableContext(), e._insertFakeElement(d.COLGROUP, u.COLGROUP), e.insertionMode = h.IN_COLUMN_GROUP, x0(e, t);
  }
  function pc(e, t) {
    e.openElements.clearBackToTableContext(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_TABLE_BODY;
  }
  function fc(e, t) {
    e.openElements.clearBackToTableContext(), e._insertFakeElement(d.TBODY, u.TBODY), e.insertionMode = h.IN_TABLE_BODY, cn(e, t);
  }
  function xc(e, t) {
    e.openElements.hasInTableScope(u.TABLE) && (e.openElements.popUntilTagNamePopped(u.TABLE), e._resetInsertionMode(), e._processStartTag(t));
  }
  function Bc(e, t) {
    Su(t) ? e._appendElement(t, p.HTML) : Rt(e, t), t.ackSelfClosing = !0;
  }
  function mc(e, t) {
    !e.formElement && e.openElements.tmplCount === 0 && (e._insertElement(t, p.HTML), e.formElement = e.openElements.current, e.openElements.pop());
  }
  function ct(e, t) {
    switch (t.tagID) {
      case u.TD:
      case u.TH:
      case u.TR:
        {
          fc(e, t);
          break;
        }
      case u.STYLE:
      case u.SCRIPT:
      case u.TEMPLATE:
        {
          xe(e, t);
          break;
        }
      case u.COL:
        {
          Cc(e, t);
          break;
        }
      case u.FORM:
        {
          mc(e, t);
          break;
        }
      case u.TABLE:
        {
          xc(e, t);
          break;
        }
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
        {
          pc(e, t);
          break;
        }
      case u.INPUT:
        {
          Bc(e, t);
          break;
        }
      case u.CAPTION:
        {
          dc(e, t);
          break;
        }
      case u.COLGROUP:
        {
          Ec(e, t);
          break;
        }
      default:
        Rt(e, t);
    }
  }
  function wt(e, t) {
    switch (t.tagID) {
      case u.TABLE:
        {
          e.openElements.hasInTableScope(u.TABLE) && (e.openElements.popUntilTagNamePopped(u.TABLE), e._resetInsertionMode());
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      case u.BODY:
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.HTML:
      case u.TBODY:
      case u.TD:
      case u.TFOOT:
      case u.TH:
      case u.THEAD:
      case u.TR:
        break;
      default:
        Rt(e, t);
    }
  }
  function Rt(e, t) {
    let n = e.fosterParentingEnabled;
    e.fosterParentingEnabled = !0, sn(e, t), e.fosterParentingEnabled = n;
  }
  function wu(e, t) {
    e.pendingCharacterTokens.push(t);
  }
  function Ru(e, t) {
    e.pendingCharacterTokens.push(t), e.hasNonWhitespacePendingCharacterToken = !0;
  }
  function Ot(e, t) {
    let n = 0;
    if (e.hasNonWhitespacePendingCharacterToken) for (; n < e.pendingCharacterTokens.length; n++) Rt(e, e.pendingCharacterTokens[n]);else for (; n < e.pendingCharacterTokens.length; n++) e._insertCharacters(e.pendingCharacterTokens[n]);
    e.insertionMode = e.originalInsertionMode, e._processToken(t);
  }
  var Ou = new Set([u.CAPTION, u.COL, u.COLGROUP, u.TBODY, u.TD, u.TFOOT, u.TH, u.THEAD, u.TR]);
  function Dc(e, t) {
    let n = t.tagID;
    Ou.has(n) ? e.openElements.hasInTableScope(u.CAPTION) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(u.CAPTION), e.activeFormattingElements.clearToLastMarker(), e.insertionMode = h.IN_TABLE, ct(e, t)) : ne(e, t);
  }
  function gc(e, t) {
    let n = t.tagID;
    switch (n) {
      case u.CAPTION:
      case u.TABLE:
        {
          e.openElements.hasInTableScope(u.CAPTION) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(u.CAPTION), e.activeFormattingElements.clearToLastMarker(), e.insertionMode = h.IN_TABLE, n === u.TABLE && wt(e, t));
          break;
        }
      case u.BODY:
      case u.COL:
      case u.COLGROUP:
      case u.HTML:
      case u.TBODY:
      case u.TD:
      case u.TFOOT:
      case u.TH:
      case u.THEAD:
      case u.TR:
        break;
      default:
        on(e, t);
    }
  }
  function x0(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.COL:
        {
          e._appendElement(t, p.HTML), t.ackSelfClosing = !0;
          break;
        }
      case u.TEMPLATE:
        {
          xe(e, t);
          break;
        }
      default:
        An(e, t);
    }
  }
  function Tc(e, t) {
    switch (t.tagID) {
      case u.COLGROUP:
        {
          e.openElements.currentTagId === u.COLGROUP && (e.openElements.pop(), e.insertionMode = h.IN_TABLE);
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      case u.COL:
        break;
      default:
        An(e, t);
    }
  }
  function An(e, t) {
    e.openElements.currentTagId === u.COLGROUP && (e.openElements.pop(), e.insertionMode = h.IN_TABLE, e._processToken(t));
  }
  function cn(e, t) {
    switch (t.tagID) {
      case u.TR:
        {
          e.openElements.clearBackToTableBodyContext(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_ROW;
          break;
        }
      case u.TH:
      case u.TD:
        {
          e.openElements.clearBackToTableBodyContext(), e._insertFakeElement(d.TR, u.TR), e.insertionMode = h.IN_ROW, ln(e, t);
          break;
        }
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
        {
          e.openElements.hasTableBodyContextInTableScope() && (e.openElements.clearBackToTableBodyContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE, ct(e, t));
          break;
        }
      default:
        ct(e, t);
    }
  }
  function B0(e, t) {
    let n = t.tagID;
    switch (t.tagID) {
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
        {
          e.openElements.hasInTableScope(n) && (e.openElements.clearBackToTableBodyContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE);
          break;
        }
      case u.TABLE:
        {
          e.openElements.hasTableBodyContextInTableScope() && (e.openElements.clearBackToTableBodyContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE, wt(e, t));
          break;
        }
      case u.BODY:
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.HTML:
      case u.TD:
      case u.TH:
      case u.TR:
        break;
      default:
        wt(e, t);
    }
  }
  function ln(e, t) {
    switch (t.tagID) {
      case u.TH:
      case u.TD:
        {
          e.openElements.clearBackToTableRowContext(), e._insertElement(t, p.HTML), e.insertionMode = h.IN_CELL, e.activeFormattingElements.insertMarker();
          break;
        }
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
      case u.TR:
        {
          e.openElements.hasInTableScope(u.TR) && (e.openElements.clearBackToTableRowContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE_BODY, cn(e, t));
          break;
        }
      default:
        ct(e, t);
    }
  }
  function Lu(e, t) {
    switch (t.tagID) {
      case u.TR:
        {
          e.openElements.hasInTableScope(u.TR) && (e.openElements.clearBackToTableRowContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE_BODY);
          break;
        }
      case u.TABLE:
        {
          e.openElements.hasInTableScope(u.TR) && (e.openElements.clearBackToTableRowContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE_BODY, B0(e, t));
          break;
        }
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
        {
          (e.openElements.hasInTableScope(t.tagID) || e.openElements.hasInTableScope(u.TR)) && (e.openElements.clearBackToTableRowContext(), e.openElements.pop(), e.insertionMode = h.IN_TABLE_BODY, B0(e, t));
          break;
        }
      case u.BODY:
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.HTML:
      case u.TD:
      case u.TH:
        break;
      default:
        wt(e, t);
    }
  }
  function Fc(e, t) {
    let n = t.tagID;
    Ou.has(n) ? (e.openElements.hasInTableScope(u.TD) || e.openElements.hasInTableScope(u.TH)) && (e._closeTableCell(), ln(e, t)) : ne(e, t);
  }
  function _c(e, t) {
    let n = t.tagID;
    switch (n) {
      case u.TD:
      case u.TH:
        {
          e.openElements.hasInTableScope(n) && (e.openElements.generateImpliedEndTags(), e.openElements.popUntilTagNamePopped(n), e.activeFormattingElements.clearToLastMarker(), e.insertionMode = h.IN_ROW);
          break;
        }
      case u.TABLE:
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
      case u.TR:
        {
          e.openElements.hasInTableScope(n) && (e._closeTableCell(), Lu(e, t));
          break;
        }
      case u.BODY:
      case u.CAPTION:
      case u.COL:
      case u.COLGROUP:
      case u.HTML:
        break;
      default:
        on(e, t);
    }
  }
  function Mu(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.OPTION:
        {
          e.openElements.currentTagId === u.OPTION && e.openElements.pop(), e._insertElement(t, p.HTML);
          break;
        }
      case u.OPTGROUP:
        {
          e.openElements.currentTagId === u.OPTION && e.openElements.pop(), e.openElements.currentTagId === u.OPTGROUP && e.openElements.pop(), e._insertElement(t, p.HTML);
          break;
        }
      case u.HR:
        {
          e.openElements.currentTagId === u.OPTION && e.openElements.pop(), e.openElements.currentTagId === u.OPTGROUP && e.openElements.pop(), e._appendElement(t, p.HTML), t.ackSelfClosing = !0;
          break;
        }
      case u.INPUT:
      case u.KEYGEN:
      case u.TEXTAREA:
      case u.SELECT:
        {
          e.openElements.hasInSelectScope(u.SELECT) && (e.openElements.popUntilTagNamePopped(u.SELECT), e._resetInsertionMode(), t.tagID !== u.SELECT && e._processStartTag(t));
          break;
        }
      case u.SCRIPT:
      case u.TEMPLATE:
        {
          xe(e, t);
          break;
        }
      default:
    }
  }
  function Hu(e, t) {
    switch (t.tagID) {
      case u.OPTGROUP:
        {
          e.openElements.stackTop > 0 && e.openElements.currentTagId === u.OPTION && e.openElements.tagIDs[e.openElements.stackTop - 1] === u.OPTGROUP && e.openElements.pop(), e.openElements.currentTagId === u.OPTGROUP && e.openElements.pop();
          break;
        }
      case u.OPTION:
        {
          e.openElements.currentTagId === u.OPTION && e.openElements.pop();
          break;
        }
      case u.SELECT:
        {
          e.openElements.hasInSelectScope(u.SELECT) && (e.openElements.popUntilTagNamePopped(u.SELECT), e._resetInsertionMode());
          break;
        }
      case u.TEMPLATE:
        {
          tt(e, t);
          break;
        }
      default:
    }
  }
  function yc(e, t) {
    let n = t.tagID;
    n === u.CAPTION || n === u.TABLE || n === u.TBODY || n === u.TFOOT || n === u.THEAD || n === u.TR || n === u.TD || n === u.TH ? (e.openElements.popUntilTagNamePopped(u.SELECT), e._resetInsertionMode(), e._processStartTag(t)) : Mu(e, t);
  }
  function bc(e, t) {
    let n = t.tagID;
    n === u.CAPTION || n === u.TABLE || n === u.TBODY || n === u.TFOOT || n === u.THEAD || n === u.TR || n === u.TD || n === u.TH ? e.openElements.hasInTableScope(n) && (e.openElements.popUntilTagNamePopped(u.SELECT), e._resetInsertionMode(), e.onEndTag(t)) : Hu(e, t);
  }
  function Sc(e, t) {
    switch (t.tagID) {
      case u.BASE:
      case u.BASEFONT:
      case u.BGSOUND:
      case u.LINK:
      case u.META:
      case u.NOFRAMES:
      case u.SCRIPT:
      case u.STYLE:
      case u.TEMPLATE:
      case u.TITLE:
        {
          xe(e, t);
          break;
        }
      case u.CAPTION:
      case u.COLGROUP:
      case u.TBODY:
      case u.TFOOT:
      case u.THEAD:
        {
          e.tmplInsertionModeStack[0] = h.IN_TABLE, e.insertionMode = h.IN_TABLE, ct(e, t);
          break;
        }
      case u.COL:
        {
          e.tmplInsertionModeStack[0] = h.IN_COLUMN_GROUP, e.insertionMode = h.IN_COLUMN_GROUP, x0(e, t);
          break;
        }
      case u.TR:
        {
          e.tmplInsertionModeStack[0] = h.IN_TABLE_BODY, e.insertionMode = h.IN_TABLE_BODY, cn(e, t);
          break;
        }
      case u.TD:
      case u.TH:
        {
          e.tmplInsertionModeStack[0] = h.IN_ROW, e.insertionMode = h.IN_ROW, ln(e, t);
          break;
        }
      default:
        e.tmplInsertionModeStack[0] = h.IN_BODY, e.insertionMode = h.IN_BODY, ne(e, t);
    }
  }
  function Ic(e, t) {
    t.tagID === u.TEMPLATE && tt(e, t);
  }
  function Pu(e, t) {
    e.openElements.tmplCount > 0 ? (e.openElements.popUntilTagNamePopped(u.TEMPLATE), e.activeFormattingElements.clearToLastMarker(), e.tmplInsertionModeStack.shift(), e._resetInsertionMode(), e.onEof(t)) : p0(e, t);
  }
  function vc(e, t) {
    t.tagID === u.HTML ? ne(e, t) : hn(e, t);
  }
  function Uu(e, t) {
    var n;
    if (t.tagID === u.HTML) {
      if (e.fragmentContext || (e.insertionMode = h.AFTER_AFTER_BODY), e.options.sourceCodeLocationInfo && e.openElements.tagIDs[0] === u.HTML) {
        e._setEndLocation(e.openElements.items[0], t);
        let r = e.openElements.items[1];
        r && !(!((n = e.treeAdapter.getNodeSourceCodeLocation(r)) === null || n === void 0) && n.endTag) && e._setEndLocation(r, t);
      }
    } else hn(e, t);
  }
  function hn(e, t) {
    e.insertionMode = h.IN_BODY, sn(e, t);
  }
  function Nc(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.FRAMESET:
        {
          e._insertElement(t, p.HTML);
          break;
        }
      case u.FRAME:
        {
          e._appendElement(t, p.HTML), t.ackSelfClosing = !0;
          break;
        }
      case u.NOFRAMES:
        {
          xe(e, t);
          break;
        }
      default:
    }
  }
  function kc(e, t) {
    t.tagID === u.FRAMESET && !e.openElements.isRootHtmlElementCurrent() && (e.openElements.pop(), !e.fragmentContext && e.openElements.currentTagId !== u.FRAMESET && (e.insertionMode = h.AFTER_FRAMESET));
  }
  function wc(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.NOFRAMES:
        {
          xe(e, t);
          break;
        }
      default:
    }
  }
  function Rc(e, t) {
    t.tagID === u.HTML && (e.insertionMode = h.AFTER_AFTER_FRAMESET);
  }
  function Oc(e, t) {
    t.tagID === u.HTML ? ne(e, t) : dn(e, t);
  }
  function dn(e, t) {
    e.insertionMode = h.IN_BODY, sn(e, t);
  }
  function Lc(e, t) {
    switch (t.tagID) {
      case u.HTML:
        {
          ne(e, t);
          break;
        }
      case u.NOFRAMES:
        {
          xe(e, t);
          break;
        }
      default:
    }
  }
  function Mc(e, t) {
    t.chars = H, e._insertCharacters(t);
  }
  function Hc(e, t) {
    e._insertCharacters(t), e.framesetOk = !1;
  }
  function Wu(e) {
    for (; e.treeAdapter.getNamespaceURI(e.openElements.current) !== p.HTML && e.openElements.currentTagId !== void 0 && !e._isIntegrationPoint(e.openElements.currentTagId, e.openElements.current);) e.openElements.pop();
  }
  function Pc(e, t) {
    if (Bu(t)) Wu(e), e._startTagOutsideForeignContent(t);else {
      let n = e._getAdjustedCurrentElement(),
        r = e.treeAdapter.getNamespaceURI(n);
      r === p.MATHML ? h0(t) : r === p.SVG && (mu(t), d0(t)), an(t), t.selfClosing ? e._appendElement(t, r) : e._insertElement(t, r), t.ackSelfClosing = !0;
    }
  }
  function Uc(e, t) {
    if (t.tagID === u.P || t.tagID === u.BR) {
      Wu(e), e._endTagOutsideForeignContent(t);
      return;
    }
    for (let n = e.openElements.stackTop; n > 0; n--) {
      let r = e.openElements.items[n];
      if (e.treeAdapter.getNamespaceURI(r) === p.HTML) {
        e._endTagOutsideForeignContent(t);
        break;
      }
      let a = e.treeAdapter.getTagName(r);
      if (a.toLowerCase() === t.tagName) {
        t.tagName = a, e.openElements.shortenToLength(n);
        break;
      }
    }
  }
  var m1 = String.prototype.codePointAt == null ? (e, t) => (e.charCodeAt(t) & 64512) === 55296 ? (e.charCodeAt(t) - 55296) * 1024 + e.charCodeAt(t + 1) - 56320 + 65536 : e.charCodeAt(t) : (e, t) => e.codePointAt(t);
  function Gu(e, t) {
    return function (n) {
      let r,
        a = 0,
        i = "";
      for (; r = e.exec(n);) a !== r.index && (i += n.substring(a, r.index)), i += t.get(r[0].charCodeAt(0)), a = r.index + 1;
      return i + n.substring(a);
    };
  }
  var Wc = Gu(/["&\u00A0]/g, new Map([[34, "&quot;"], [38, "&amp;"], [160, "&nbsp;"]])),
    Gc = Gu(/[&<>\u00A0]/g, new Map([[38, "&amp;"], [60, "&lt;"], [62, "&gt;"], [160, "&nbsp;"]])),
    Kc = new Set([d.AREA, d.BASE, d.BASEFONT, d.BGSOUND, d.BR, d.COL, d.EMBED, d.FRAME, d.HR, d.IMG, d.INPUT, d.KEYGEN, d.LINK, d.META, d.PARAM, d.SOURCE, d.TRACK, d.WBR]);
  function Qc(e, t) {
    return t.treeAdapter.isElementNode(e) && t.treeAdapter.getNamespaceURI(e) === p.HTML && Kc.has(t.treeAdapter.getTagName(e));
  }
  var Yc = {
    treeAdapter: ve,
    scriptingEnabled: !0
  };
  function Xc(e, t) {
    let n = L(L({}, Yc), t);
    return Ku(e, n);
  }
  function Jc(e, t) {
    let n = "",
      r = t.treeAdapter.isElementNode(e) && t.treeAdapter.getTagName(e) === d.TEMPLATE && t.treeAdapter.getNamespaceURI(e) === p.HTML ? t.treeAdapter.getTemplateContent(e) : e,
      a = t.treeAdapter.getChildNodes(r);
    if (a) for (let i of a) n += Ku(i, t);
    return n;
  }
  function Ku(e, t) {
    return t.treeAdapter.isElementNode(e) ? qc(e, t) : t.treeAdapter.isTextNode(e) ? jc(e, t) : t.treeAdapter.isCommentNode(e) ? Zc(e, t) : t.treeAdapter.isDocumentTypeNode(e) ? zc(e, t) : "";
  }
  function qc(e, t) {
    let n = t.treeAdapter.getTagName(e);
    return "<".concat(n).concat(Vc(e, t), ">").concat(Qc(e, t) ? "" : "".concat(Jc(e, t), "</").concat(n, ">"));
  }
  function Vc(e, {
    treeAdapter: t
  }) {
    let n = "";
    for (let r of t.getAttrList(e)) {
      if (n += " ", r.namespace) switch (r.namespace) {
        case p.XML:
          {
            n += "xml:".concat(r.name);
            break;
          }
        case p.XMLNS:
          {
            r.name !== "xmlns" && (n += "xmlns:"), n += r.name;
            break;
          }
        case p.XLINK:
          {
            n += "xlink:".concat(r.name);
            break;
          }
        default:
          n += "".concat(r.prefix, ":").concat(r.name);
      } else n += r.name;
      n += '="'.concat(Wc(r.value), '"');
    }
    return n;
  }
  function jc(e, t) {
    let n = t.treeAdapter,
      r = n.getTextNodeContent(e),
      a = n.getParentNode(e),
      i = a && n.isElementNode(a) && n.getTagName(a);
    return i && n.getNamespaceURI(a) === p.HTML && uu(i, t.scriptingEnabled) ? r : Gc(r);
  }
  function Zc(e, {
    treeAdapter: t
  }) {
    return "<!--".concat(t.getCommentNodeContent(e), "-->");
  }
  function zc(e, {
    treeAdapter: t
  }) {
    return "<!DOCTYPE ".concat(t.getDocumentTypeNodeName(e), ">");
  }
  function $c(e, t) {
    return Fu.parse(e, t);
  }
  function e1(e, t, n) {
    typeof e == "string" && (n = t, t = e, e = null);
    let r = Fu.getFragmentParser(e, n);
    return r.tokenizer.write(t, !0), r.getFragment();
  }
  function Qu(e) {
    let t = e.includes('"') ? "'" : '"';
    return t + e + t;
  }
  function t1(e, t, n) {
    let r = "!DOCTYPE ";
    return e && (r += e), t ? r += " PUBLIC ".concat(Qu(t)) : n && (r += " SYSTEM"), n && (r += " ".concat(Qu(n))), r;
  }
  var Ke = {
    isCommentNode: Wt,
    isElementNode: y,
    isTextNode: Be,
    createDocument() {
      let e = new Ve([]);
      return e["x-mode"] = nu.DOCUMENT_MODE.NO_QUIRKS, e;
    },
    createDocumentFragment() {
      return new Ve([]);
    },
    createElement(e, t, n) {
      let r = Object.create(null),
        a = Object.create(null),
        i = Object.create(null);
      for (let l = 0; l < n.length; l++) {
        let c = n[l].name;
        r[c] = n[l].value, a[c] = n[l].namespace, i[c] = n[l].prefix;
      }
      let s = new Dn(e, r, []);
      return s.namespace = t, s["x-attribsNamespace"] = a, s["x-attribsPrefix"] = i, s;
    },
    createCommentNode(e) {
      return new xn(e);
    },
    createTextNode(e) {
      return new dt(e);
    },
    appendChild(e, t) {
      let n = e.children[e.children.length - 1];
      n && (n.next = t, t.prev = n), e.children.push(t), t.parent = e;
    },
    insertBefore(e, t, n) {
      let r = e.children.indexOf(n),
        a = n.prev;
      a && (a.next = t, t.prev = a), n.prev = t, t.next = n, e.children.splice(r, 0, t), t.parent = e;
    },
    setTemplateContent(e, t) {
      Ke.appendChild(e, t);
    },
    getTemplateContent(e) {
      return e.children[0];
    },
    setDocumentType(e, t, n, r) {
      let a = t1(t, n, r),
        i = e.children.find(s => gn(s) && s.name === "!doctype");
      i ? i.data = a != null ? a : null : (i = new Bn("!doctype", a), Ke.appendChild(e, i)), i["x-name"] = t, i["x-publicId"] = n, i["x-systemId"] = r;
    },
    setDocumentMode(e, t) {
      e["x-mode"] = t;
    },
    getDocumentMode(e) {
      return e["x-mode"];
    },
    detachNode(e) {
      if (e.parent) {
        let t = e.parent.children.indexOf(e),
          n = e.prev,
          r = e.next;
        e.prev = null, e.next = null, n && (n.next = r), r && (r.prev = n), e.parent.children.splice(t, 1), e.parent = null;
      }
    },
    insertText(e, t) {
      let n = e.children[e.children.length - 1];
      n && Be(n) ? n.data += t : Ke.appendChild(e, Ke.createTextNode(t));
    },
    insertTextBefore(e, t, n) {
      let r = e.children[e.children.indexOf(n) - 1];
      r && Be(r) ? r.data += t : Ke.insertBefore(e, Ke.createTextNode(t), n);
    },
    adoptAttributes(e, t) {
      for (let n = 0; n < t.length; n++) {
        let r = t[n].name;
        e.attribs[r] === void 0 && (e.attribs[r] = t[n].value, e["x-attribsNamespace"][r] = t[n].namespace, e["x-attribsPrefix"][r] = t[n].prefix);
      }
    },
    getFirstChild(e) {
      return e.children[0];
    },
    getChildNodes(e) {
      return e.children;
    },
    getParentNode(e) {
      return e.parent;
    },
    getAttrList(e) {
      return e.attributes;
    },
    getTagName(e) {
      return e.name;
    },
    getNamespaceURI(e) {
      return e.namespace;
    },
    getTextNodeContent(e) {
      return e.data;
    },
    getCommentNodeContent(e) {
      return e.data;
    },
    getDocumentTypeNodeName(e) {
      var t;
      return (t = e["x-name"]) !== null && t !== void 0 ? t : "";
    },
    getDocumentTypeNodePublicId(e) {
      var t;
      return (t = e["x-publicId"]) !== null && t !== void 0 ? t : "";
    },
    getDocumentTypeNodeSystemId(e) {
      var t;
      return (t = e["x-systemId"]) !== null && t !== void 0 ? t : "";
    },
    isDocumentTypeNode(e) {
      return gn(e) && e.name === "!doctype";
    },
    setNodeSourceCodeLocation(e, t) {
      t && (e.startIndex = t.startOffset, e.endIndex = t.endOffset), e.sourceCodeLocation = t;
    },
    getNodeSourceCodeLocation(e) {
      return e.sourceCodeLocation;
    },
    updateNodeSourceCodeLocation(e, t) {
      t.endOffset != null && (e.endIndex = t.endOffset), e.sourceCodeLocation = L(L({}, e.sourceCodeLocation), t);
    }
  };
  function n1(e, t, n, r) {
    var a;
    return (a = t.treeAdapter) !== null && a !== void 0 || (t.treeAdapter = Ke), t.scriptingEnabled !== !1 && (t.scriptingEnabled = !0), n ? $c(e, t) : e1(r, e, t);
  }
  var r1 = {
    treeAdapter: Ke
  };
  function u1(e) {
    let t = "length" in e ? e : [e];
    for (let r = 0; r < t.length; r += 1) {
      let a = t[r];
      Oe(a) && Array.prototype.splice.call(t, r, 1, ...a.children);
    }
    let n = "";
    for (let r = 0; r < t.length; r += 1) {
      let a = t[r];
      n += Xc(a, r1);
    }
    return n;
  }
  var a1 = Ms((e, t, n, r) => t._useHtmlParser2 ? pi(e, t) : n1(e, t, n, r)),
    i1 = ho(a1, (e, t) => t._useHtmlParser2 ? Y0(e, t) : u1(e)),
    m0 = typeof self < "u" ? self : typeof globalThis < "u" ? globalThis : void 0,
    Yu = T0,
    ie = null;
  function Te(e) {
    m0.postMessage(e);
  }
  function En(e, t) {
    var n;
    try {
      return e.getString(t);
    } catch (r) {
      try {
        return String((n = e.dump(t)) != null ? n : "");
      } catch (a) {
        return "";
      }
    }
  }
  function s1(e, t = 4, n = 1e4) {
    var r,
      a = new Map(),
      i = new Map(),
      s = 0,
      l = 0,
      c = Yu.load || ((r = Yu.default) == null ? void 0 : r.load);
    if (typeof c != "function") throw new Error("Cheerio load is unavailable");
    function E(k) {
      if (a.size >= t) throw new Error("Plugin DOM document quota exceeded");
      var g = "d" + ++s;
      return a.set(g, c(String(k || ""), {
        decodeEntities: !1
      })), g;
    }
    function f(k, g) {
      if (!g) return "";
      if (i.size >= n) throw new Error("Plugin DOM element quota exceeded");
      var I = "e" + ++l;
      return i.set(I, {
        documentId: k,
        node: g
      }), I;
    }
    function m(k) {
      return i.get(String(k || ""));
    }
    function D(k, g) {
      return (g || []).map(I => f(k, I)).filter(Boolean);
    }
    function T(k, g) {
      return (g || []).map(I => m(I)).filter(I => I && I.documentId === k).map(I => I.node);
    }
    function b(k, g) {
      var I = a.get(k);
      if (!I) return "";
      var te = ee => String(ee != null ? ee : "").replace(/\s+/g, " ").trim();
      return T(k, g).map(ee => te(I(ee).text())).join(" ");
    }
    function Y(k, g) {
      var I = a.get(k);
      if (!I) return "";
      var te = m(g);
      return g ? te && I(te.node).html() || "" : I.html() || "";
    }
    function re(k, g) {
      switch (k) {
        case "load":
          return E(g[0]);
        case "select":
          try {
            var I = a.get(g[0]);
            return I ? JSON.stringify(D(g[0], I(g[1] || "").toArray())) : "[]";
          } catch (D0) {
            return "[]";
          }
        case "find":
          try {
            var te = a.get(g[0]),
              ee = m(g[1]);
            return te ? JSON.stringify(D(g[0], ee ? te(ee.node).find(g[2] || "").toArray() : [])) : "[]";
          } catch (D0) {
            return "[]";
          }
        case "text":
          return b(g[0], String(g[1] || "").split(",").filter(Boolean));
        case "html":
          return Y(g[0], g[1]);
        case "innerHtml":
          {
            var q = a.get(g[0]),
              nt = m(g[1]);
            return q && nt && q(nt.node).html() || "";
          }
        case "attr":
          {
            var Cn = a.get(g[0]),
              Lt = m(g[1]);
            if (!Cn || !Lt) return "__UNDEFINED__";
            var Ne = Cn(Lt.node).attr(g[2]);
            return Ne == null || Ne === "" ? "__UNDEFINED__" : String(Ne);
          }
        case "next":
          {
            var Mt = m(g[1]),
              lt = a.get(g[0]);
            if (!lt || !Mt) return "__NONE__";
            var ke = lt(Mt.node).next().get(0);
            return ke && f(g[0], ke) || "__NONE__";
          }
        case "prev":
          {
            var S = m(g[1]),
              W = a.get(g[0]);
            if (!W || !S) return "__NONE__";
            var M = W(S.node).prev().get(0);
            return M && f(g[0], M) || "__NONE__";
          }
        case "parent":
          {
            var G = m(g[1]),
              ce = a.get(g[0]);
            if (!ce || !G) return "__NONE__";
            var Ce = ce(G.node).parent().get(0);
            return Ce && f(g[0], Ce) || "__NONE__";
          }
        case "children":
          try {
            var pn = a.get(g[0]),
              pe = m(g[1]);
            return !pn || !pe ? "[]" : JSON.stringify(D(g[0], pn(pe.node).children().toArray()));
          } catch (D0) {
            return "[]";
          }
        case "filter":
          try {
            return JSON.stringify(D(g[0], a.get(g[0])(T(g[0], String(g[1] || "").split(",").filter(Boolean))).filter(g[2] || "").toArray()));
          } catch (D0) {
            return "[]";
          }
        case "eq":
          {
            var Fe = String(g[1] || "").split(",").filter(Boolean),
              Ht = Number(g[2]);
            return Ht >= 0 && Ht < Fe.length ? Fe[Ht] : "";
          }
        default:
          return "";
      }
    }
    return function (k) {
      var g = ["load", "select", "find", "text", "html", "innerHtml", "attr", "next", "prev", "parent", "children", "filter", "eq"];
      g.forEach(I => {
        var te = k.newFunction("__cheerio_" + I, (...ee) => {
          var q;
          return k.newString(String((q = re(I, ee.map(nt => En(k, nt)))) != null ? q : ""));
        });
        te.consume(ee => k.setProp(k.global, "__cheerio_" + I, ee));
      }), e.disposeCheerio = function () {
        a.clear(), i.clear();
      };
    };
  }
  function o1() {
    return "\n    (function() {\n    // Capture the host bridge in this trusted closure. The names installed by\n    // the host are removed immediately after this polyfill is evaluated, so a\n    // plugin can use the small public compatibility surface but cannot call\n    // private bridge functions directly.\n    var __nuvioNativeFetch = __native_fetch;\n    var __nuvioNativeCancel = __native_cancel;\n    var __nuvioNativeLog = __native_log;\n    var __nuvioFetchCounter = 0;\n    var __nuvioParseUrl = __parse_url;\n    var __nuvioCheerioLoad = __cheerio_load;\n    var __nuvioCheerioSelect = __cheerio_select;\n    var __nuvioCheerioFind = __cheerio_find;\n    var __nuvioCheerioText = __cheerio_text;\n    var __nuvioCheerioHtml = __cheerio_html;\n    var __nuvioCheerioInnerHtml = __cheerio_innerHtml;\n    var __nuvioCheerioAttr = __cheerio_attr;\n    var __nuvioCheerioNext = __cheerio_next;\n    var __nuvioCheerioPrev = __cheerio_prev;\n    var __nuvioCheerioParent = __cheerio_parent;\n    var __nuvioCheerioChildren = __cheerio_children;\n    var __nuvioCheerioFilter = __cheerio_filter;\n    var __nuvioCheerioEq = __cheerio_eq;\n\n    globalThis.global = globalThis;\n    globalThis.window = globalThis;\n    globalThis.self = globalThis;\n    function __nuvioFormatLogValue(value) {\n      try {\n        if (value && value.stack) return String(value.stack);\n      } catch (_) {}\n      if (value === undefined) return 'undefined';\n      if (value === null) return 'null';\n      if (typeof value === 'string') return value;\n      try {\n        var json = JSON.stringify(value);\n        return json === undefined ? String(value) : json;\n      } catch (_) {\n        return String(value);\n      }\n    }\n    function __nuvioForwardPluginLog(level, args) {\n      var values = [];\n      for (var i = 0; i < args.length; i++) values.push(__nuvioFormatLogValue(args[i]));\n      try { __nuvioNativeLog(level, values.join(' ')); } catch (_) {}\n    }\n    // Keep provider info/debug logs quiet, but forward warnings and errors to\n    // the host. The host writes them through console.warn/error so they are\n    // visible in Inspector and in Settings > Console debug.\n    globalThis.console = {\n      log: function() {},\n      info: function() {},\n      warn: function() { __nuvioForwardPluginLog('warn', arguments); },\n      error: function() { __nuvioForwardPluginLog('error', arguments); },\n      debug: function() {}\n    };\n    globalThis.SCRAPER_ID = __get_scraper_id();\n    globalThis.SCRAPER_SETTINGS = JSON.parse(__get_scraper_settings());\n    globalThis.TMDB_API_KEY = __get_tmdb_api_key();\n\n    function __nuvioEncodeBytes(bytes) {\n      var chunks = [];\n      for (var i = 0; i < bytes.length; i += 8192) {\n        chunks.push(String.fromCharCode.apply(null, bytes.subarray(i, i + 8192)));\n      }\n      return btoa(chunks.join(''));\n    }\n    function __nuvioDecodeBytes(base64) {\n      var binary = atob(base64);\n      var bytes = new Uint8Array(binary.length);\n      for (var i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);\n      return bytes.buffer;\n    }\n    function __nuvioNormalizeFetchBody(body) {\n      if (body === undefined || body === null) return { kind: 'none', value: '' };\n      if (typeof body === 'string') return { kind: 'text', value: body };\n      var bytes = null;\n      if (typeof ArrayBuffer !== 'undefined' && body instanceof ArrayBuffer) {\n        bytes = new Uint8Array(body);\n      } else if (typeof ArrayBuffer !== 'undefined' &&\n                 typeof ArrayBuffer.isView === 'function' && ArrayBuffer.isView(body)) {\n        bytes = new Uint8Array(body.buffer, body.byteOffset, body.byteLength);\n      }\n      if (bytes !== null) return { kind: 'base64', value: __nuvioEncodeBytes(bytes) };\n      return { kind: 'text', value: String(body) };\n    }\n    var fetch = function(url, options) {\n      options = options || {};\n      var method = String(options.method || 'GET').toUpperCase();\n      var headers = options.headers || {};\n      var body = __nuvioNormalizeFetchBody(options.body);\n      var signal = options.signal;\n      if (signal && signal.aborted) { var before = new Error('The operation was aborted.'); before.name = 'AbortError'; return Promise.reject(before); }\n      if (!headers['User-Agent']) headers['User-Agent'] = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36';\n      var abortToken = signal ? 'fetch-' + (++__nuvioFetchCounter) : '';\n      var abortListener = abortToken ? function() { try { __nuvioNativeCancel(abortToken); } catch (_) {} } : null;\n      var cleanup = function() { if (signal && abortListener) signal.removeEventListener('abort', abortListener); };\n      if (signal && abortListener) signal.addEventListener('abort', abortListener);\n      var request = { url: String(url && url.href || url || ''), method: method, headers: headers, bodyKind: body.kind, body: body.kind === 'text' ? body.value : '', responseEncoding: 'base64' };\n      if (body.kind === 'base64') request.bodyBase64 = body.value;\n      return __nuvioNativeFetch(JSON.stringify(request), abortToken).then(function(raw) {\n        cleanup();\n        var payload = JSON.parse(raw);\n        if (signal && signal.aborted) { var after = new Error('The operation was aborted.'); after.name = 'AbortError'; return Promise.reject(after); }\n        return {\n          ok: payload.ok,\n          status: payload.status,\n          statusText: payload.statusText,\n          url: payload.url,\n          headers: {\n            get: function(name) {\n              return payload.headers && payload.headers[String(name || '').toLowerCase()] || null;\n            }\n          },\n          text: function() { return Promise.resolve(payload.body); },\n          arrayBuffer: function() {\n            var encoded = typeof payload.bodyBase64 === 'string' ? payload.bodyBase64 :\n              btoa(unescape(encodeURIComponent(payload.body || '')));\n            return Promise.resolve(__nuvioDecodeBytes(encoded));\n          },\n          json: function() {\n            try {\n              if (payload.body === null || payload.body === undefined || payload.body === '') return Promise.resolve(null);\n              return Promise.resolve(JSON.parse(payload.body));\n            } catch (_) {\n              console.error('fetch.json parse error:', _ && _.message ? _.message : _);\n              return Promise.resolve(null);\n            }\n          }\n        };\n      }, function(error) {\n        cleanup();\n        throw error;\n      });\n    };\n    globalThis.fetch = fetch;\n    if (typeof AbortSignal === 'undefined') {\n      globalThis.AbortSignal = function() { this.aborted = false; this.reason = undefined; this._listeners = []; };\n      AbortSignal.prototype.addEventListener = function(type, fn) { if (type === 'abort' && typeof fn === 'function') this._listeners.push(fn); };\n      AbortSignal.prototype.removeEventListener = function(type, fn) { if (type === 'abort') this._listeners = this._listeners.filter(function(entry) { return entry !== fn; }); };\n      AbortSignal.prototype.dispatchEvent = function(event) { if (!event || event.type !== 'abort') return true; this._listeners.slice().forEach(function(fn) { try { fn.call(this, event); } catch (_) {} }, this); return true; };\n    }\n    if (typeof AbortController === 'undefined') {\n      globalThis.AbortController = function() { this.signal = new AbortSignal(); };\n      AbortController.prototype.abort = function(reason) { if (this.signal.aborted) return; this.signal.aborted = true; this.signal.reason = reason; this.signal.dispatchEvent({ type: 'abort' }); };\n    }\n    if (typeof atob === 'undefined') globalThis.atob = function(input) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var str = String(input).replace(/=+$/, ''); if (str.length % 4 === 1) throw new Error('InvalidCharacterError'); var output = ''; var bc = 0; var bs; var buffer; var idx = 0; while ((buffer = str.charAt(idx++))) { buffer = chars.indexOf(buffer); if (buffer === -1) continue; bs = bc % 4 ? bs * 64 + buffer : buffer; if (bc++ % 4) output += String.fromCharCode(255 & (bs >> ((-2 * bc) & 6))); } return output; };\n    if (typeof btoa === 'undefined') globalThis.btoa = function(input) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var str = String(input); var output = ''; for (var block, charCode, idx = 0, map = chars; str.charAt(idx | 0) || (map = '=', idx % 1); output += map.charAt(63 & (block >> (8 - (idx % 1) * 8)))) { charCode = str.charCodeAt(idx += 3 / 4); if (charCode > 0xFF) throw new Error('InvalidCharacterError'); block = (block << 8) | charCode; } return output; };\n    var URL = function(url, base) {\n      var urlString = String(url && url.href || url || '');\n      var fullUrl = urlString;\n      if (base && !new RegExp('^https?://', 'i').test(urlString)) {\n        var baseString = typeof base === 'string' ? base : base.href;\n        if (urlString.charAt(0) === '/') {\n          var originMatch = baseString.match(new RegExp('^(https?://[^/]+)'));\n          fullUrl = originMatch ? originMatch[1] + urlString : urlString;\n        } else {\n          fullUrl = baseString.replace(new RegExp('/[^/]*$'), '/') + urlString;\n        }\n      }\n      var parsed = JSON.parse(__nuvioParseUrl(fullUrl));\n      this.href = fullUrl; this.protocol = parsed.protocol; this.host = parsed.host; this.hostname = parsed.hostname; this.port = parsed.port; this.pathname = parsed.pathname; this.search = parsed.search; this.hash = parsed.hash;\n      this.origin = parsed.protocol + '//' + parsed.host;\n      this.searchParams = new URLSearchParams(parsed.search);\n    };\n    globalThis.URL = URL;\n    URL.prototype.toString = function() { return this.href; };\n    var URLSearchParams = function(init) {\n      this._params = {};\n      var self = this;\n      if (init && typeof init === 'object' && !Array.isArray(init)) {\n        Object.keys(init).forEach(function(key) { self._params[key] = String(init[key]); });\n      } else if (typeof init === 'string') {\n        init.replace(/^\\?/, '').split('&').forEach(function(pair) {\n          var parts = pair.split('=');\n          if (parts[0]) self._params[decodeURIComponent(parts[0])] = decodeURIComponent(parts[1] || '');\n        });\n      }\n    };\n    globalThis.URLSearchParams = URLSearchParams;\n    URLSearchParams.prototype.toString = function() { var self = this; return Object.keys(this._params).map(function(key) { return encodeURIComponent(key) + '=' + encodeURIComponent(self._params[key]); }).join('&'); };\n    URLSearchParams.prototype.get = function(key) { return this._params.hasOwnProperty(key) ? this._params[key] : null; };\n    URLSearchParams.prototype.set = function(key, value) { this._params[key] = String(value); };\n    URLSearchParams.prototype.append = function(key, value) { this._params[key] = String(value); };\n    URLSearchParams.prototype.has = function(key) { return this._params.hasOwnProperty(key); };\n    URLSearchParams.prototype.delete = function(key) { delete this._params[key]; };\n    URLSearchParams.prototype.keys = function() { return Object.keys(this._params); };\n    URLSearchParams.prototype.values = function() { var self = this; return Object.keys(this._params).map(function(key) { return self._params[key]; }); };\n    URLSearchParams.prototype.entries = function() { var self = this; return Object.keys(this._params).map(function(key) { return [key, self._params[key]]; }); };\n    URLSearchParams.prototype.forEach = function(callback) { var self = this; Object.keys(this._params).forEach(function(key) { callback(self._params[key], key, self); }); };\n    URLSearchParams.prototype.getAll = function(key) { return this._params.hasOwnProperty(key) ? [this._params[key]] : []; };\n    URLSearchParams.prototype.sort = function() { var sorted = {}; var self = this; Object.keys(this._params).sort().forEach(function(key) { sorted[key] = self._params[key]; }); this._params = sorted; };\n\n    function __nuvioWrap(docId, ids) {\n      ids = ids || [];\n      var wrapper = { _docId: docId, _elementIds: ids, length: ids.length };\n      wrapper.each = function(callback) { for (var i = 0; i < ids.length; i++) { var item = __nuvioWrap(docId, [ids[i]]); callback.call(item, i, item); } return wrapper; };\n      wrapper.find = function(selector) { var allIds = []; for (var i = 0; i < ids.length; i++) { var found = JSON.parse(__nuvioCheerioFind(docId, ids[i], selector)); allIds = allIds.concat(found); } return __nuvioWrap(docId, allIds); };\n      wrapper.text = function() { if (ids.length === 0) return ''; return __nuvioCheerioText(docId, ids.join(',')); };\n      wrapper.html = function() { if (ids.length === 0) return ''; return __nuvioCheerioInnerHtml(docId, ids[0]); };\n      wrapper.attr = function(name) { if (ids.length === 0) return undefined; var value = __nuvioCheerioAttr(docId, ids[0], name); return value === '__UNDEFINED__' ? undefined : value; };\n      wrapper.first = function() { return __nuvioWrap(docId, ids.length > 0 ? [ids[0]] : []); };\n      wrapper.last = function() { return __nuvioWrap(docId, ids.length > 0 ? [ids[ids.length - 1]] : []); };\n      wrapper.next = function() { var nextIds = []; for (var i = 0; i < ids.length; i++) { var nextId = __nuvioCheerioNext(docId, ids[i]); if (nextId && nextId !== '__NONE__') nextIds.push(nextId); } return __nuvioWrap(docId, nextIds); };\n      wrapper.prev = function() { var prevIds = []; for (var i = 0; i < ids.length; i++) { var prevId = __nuvioCheerioPrev(docId, ids[i]); if (prevId && prevId !== '__NONE__') prevIds.push(prevId); } return __nuvioWrap(docId, prevIds); };\n      wrapper.eq = function(index) { if (index >= 0 && index < ids.length) return __nuvioWrap(docId, [ids[index]]); return __nuvioWrap(docId, []); };\n      wrapper.get = function(index) { if (typeof index === 'number') { if (index >= 0 && index < ids.length) return __nuvioWrap(docId, [ids[index]]); return undefined; } return ids.map(function(id) { return __nuvioWrap(docId, [id]); }); };\n      wrapper.map = function(callback) { var values = []; for (var i = 0; i < ids.length; i++) { var item = __nuvioWrap(docId, [ids[i]]); var value = callback.call(item, i, item); if (value !== undefined && value !== null) values.push(value); } return { length: values.length, get: function(index) { return typeof index === 'number' ? values[index] : values; }, toArray: function() { return values; } }; };\n      wrapper.filter = function(selectorOrCallback) { if (typeof selectorOrCallback !== 'function') return wrapper; var filtered = []; for (var i = 0; i < ids.length; i++) { var item = __nuvioWrap(docId, [ids[i]]); if (selectorOrCallback.call(item, i, item)) filtered.push(ids[i]); } return __nuvioWrap(docId, filtered); };\n      wrapper.children = function(selector) { return wrapper.find(selector || '*'); };\n      wrapper.parent = function() { return __nuvioWrap(docId, []); };\n      wrapper.toArray = function() { return ids.map(function(id) { return __nuvioWrap(docId, [id]); }); };\n      return wrapper;\n    }\n    function __nuvioLoad(html) {\n      var docId = __nuvioCheerioLoad(String(html || ''));\n      var $ = function(selector, context) {\n        if (selector && selector._elementIds) return selector;\n        if (context && context._elementIds && context._elementIds.length > 0) {\n          var contextIds = [];\n          for (var i = 0; i < context._elementIds.length; i++) {\n            JSON.parse(__nuvioCheerioFind(docId, context._elementIds[i], selector)).forEach(function(id) { contextIds.push(id); });\n          }\n          return __nuvioWrap(docId, contextIds);\n        }\n        return __nuvioWrap(docId, JSON.parse(__nuvioCheerioSelect(docId, String(selector || ''))));\n      };\n      $.html = function(element) { return element && element._elementIds && element._elementIds.length > 0 ? __nuvioCheerioHtml(docId, element._elementIds[0]) : __nuvioCheerioHtml(docId, ''); };\n      return $;\n    }\n    var cheerio = { load: __nuvioLoad };\n    var require = function(name) { if (name === 'cheerio' || name === 'cheerio-without-node-native' || name === 'react-native-cheerio') return cheerio; if (name === 'crypto-js') return globalThis.CryptoJS; throw new Error('Module not allowed: ' + name); };\n    // Android exposes these as top-level compatibility bindings. Publish only\n    // the lowercase module/function names that providers can use there; do not\n    // add browser-only constructor globals such as Headers, Response or\n    // Cheerio.\n    globalThis.cheerio = cheerio;\n    globalThis.require = require;\n    if (!Array.prototype.flat) {\n      Array.prototype.flat = function(depth) {\n        depth = depth === undefined ? 1 : Math.floor(depth);\n        if (depth < 1) return Array.prototype.slice.call(this);\n        return (function flatten(arr, currentDepth) {\n          return currentDepth > 0 ? arr.reduce(function(acc, value) {\n            return acc.concat(Array.isArray(value) ? flatten(value, currentDepth - 1) : value);\n          }, []) : arr.slice();\n        })(this, depth);\n      };\n    }\n    if (!Array.prototype.flatMap) {\n      Array.prototype.flatMap = function(callback, thisArg) {\n        return this.map(callback, thisArg).flat();\n      };\n    }\n    if (!Object.entries) {\n      Object.entries = function(object) {\n        var result = [];\n        for (var key in object) if (Object.prototype.hasOwnProperty.call(object, key)) result.push([key, object[key]]);\n        return result;\n      };\n    }\n    if (!Object.fromEntries) {\n      Object.fromEntries = function(entries) {\n        var result = {};\n        for (var i = 0; i < entries.length; i++) result[entries[i][0]] = entries[i][1];\n        return result;\n      };\n    }\n    if (!String.prototype.replaceAll) {\n      String.prototype.replaceAll = function(search, replacement) {\n        if (search instanceof RegExp) {\n          if (!search.global) throw new TypeError('replaceAll must be called with a global RegExp');\n          return this.replace(search, replacement);\n        }\n        return this.split(search).join(replacement);\n      };\n    }\n    // Keep the standard JavaScript dynamic-code APIs available inside this\n    // isolated QuickJS context. Android providers use eval/Function for\n    // deobfuscation; isolation is provided by the dedicated context and the\n    // bridge cleanup above, not by rejecting valid JavaScript syntax.\n    })();\n  ";
  }
  function A1(e, t, n, r) {
    return g0(this, null, function* () {
      var a;
      let i = !1,
        s = null;
      e.resolvePromise(n).then(c => {
        i = !0, s = c;
      }, c => {
        i = !0, s = c;
      });
      let l = Date.now();
      for (; !i && Date.now() - l < Number(r || 6e4);) {
        let c = t.executePendingJobs();
        if (c != null && c.error) {
          let E = e.unwrapResult(c),
            f = String(((a = e.dump(E)) == null ? void 0 : a.message) || "QuickJS pending job failed");
          throw E.dispose(), new Error(f);
        }
        yield new Promise(E => setTimeout(E, 0));
      }
      if (!i) throw new Error("Plugin promise timed out");
      return s;
    });
  }
  function c1(e) {
    return g0(this, null, function* () {
      var t, n;
      importScripts("../libs/quickjs-emscripten.global.js");
      var r = m0.QJS;
      if (!r || typeof r.getQuickJS != "function") throw new Error("QuickJS WASM asset unavailable");
      var a = e || {},
        i = a.quota || {},
        s = {
          pending: new Map(),
          abortTokens: new Map(),
          requestCounter: 0,
          disposeCheerio: null,
          context: null,
          settleFetch: null,
          rejectPending: null
        };
      ie = s;
      var l = yield r.getQuickJS(),
        c = l.newContext({
          intrinsics: {
            BaseObjects: !0,
            Date: !0,
            Eval: !0,
            StringNormalize: !0,
            RegExp: !0,
            JSON: !0,
            Proxy: !0,
            MapSet: !0,
            TypedArrays: !0,
            Promise: !0,
            BigInt: !0
          }
        }),
        E = c.runtime;
      s.context = c, (t = E.setMemoryLimit) == null || t.call(E, Number(i.memoryLimitBytes || 32 * 1024 * 1024));
      var f = Number(a.deadline) || Date.now() + Number(a.timeoutMs || 6e4);
      (n = E.setInterruptHandler) == null || n.call(E, () => Date.now() > f);
      function m(S, W) {
        var M = c.newFunction(S, (...G) => {
          var ce;
          return c.newString(String((ce = W(...G.map(Ce => En(c, Ce)))) != null ? ce : ""));
        });
        M.consume(G => c.setProp(c.global, S, G));
      }
      m("__get_scraper_id", () => String(a.scraperId || "")), m("__get_scraper_settings", () => JSON.stringify(a.settings || {})), m("__get_tmdb_api_key", () => String(a.tmdbApiKey || "")), m("__native_log", (S, W) => (Te({
        type: "pluginLog",
        level: String(S || "").toLowerCase() === "error" ? "error" : "warn",
        message: String(W || "").slice(0, 4e3)
      }), "")), m("__native_cancel", S => {
        var W = s.abortTokens.get(String(S || ""));
        return W ? (Te({
          type: "cancel",
          requestId: W
        }), !0) : !1;
      }), m("__parse_url", (S, W) => {
        try {
          var M = new URL(String(S || ""), String(W || "") || void 0);
          return JSON.stringify({
            href: M.href,
            protocol: M.protocol,
            host: M.host,
            hostname: M.hostname,
            port: M.port,
            pathname: M.pathname,
            search: M.search,
            hash: M.hash
          });
        } catch (G) {
          return JSON.stringify({
            href: String(S || ""),
            protocol: "",
            host: "",
            hostname: "",
            port: "",
            pathname: "/",
            search: "",
            hash: ""
          });
        }
      }), s1(s, Math.max(1, Number(i.maxDocuments || 4)), Math.max(100, Number(i.maxDomElements || 1e4)))(c);
      function D(S, W, M) {
        var G = s.pending.get(S);
        if (G) {
          s.pending.delete(S), G.abortToken && s.abortTokens.delete(G.abortToken);
          try {
            if (M) {
              var ce = c.newError(M);
              try {
                G.deferred.reject(ce);
              } finally {
                ce.dispose();
              }
            } else {
              var Ce = c.newString(JSON.stringify(W || {}));
              try {
                G.deferred.resolve(Ce);
              } finally {
                Ce.dispose();
              }
            }
          } finally {
            G.deferred.dispose();
          }
        }
      }
      s.settleFetch = D, s.rejectPending = function (S) {
        Array.from(s.pending.keys()).forEach(W => D(W, null, S));
      }, s.cleanup = function () {
        var S, W, M, G, ce, Ce;
        if (!s.cleaned) {
          s.cleaned = !0, (S = s.rejectPending) == null || S.call(s, new Error("Plugin execution ended"));
          for (let pe = 0; pe < 64; pe += 1) {
            let Fe;
            try {
              Fe = E.executePendingJobs();
            } catch (Ht) {
              break;
            }
            if (Fe != null && Fe.error) {
              try {
                c.unwrapResult(Fe).dispose();
              } catch (Ht) {}
              break;
            }
            if (!Number((Fe == null ? void 0 : Fe.value) || 0)) break;
          }
          s.pending.clear();
          var pn = Array.from(((M = (W = E.contextMap) == null ? void 0 : W.values) == null ? void 0 : M.call(W)) || []);
          pn.forEach(pe => {
            if (!(!pe || pe === s.context || !pe.alive)) try {
              pe.dispose();
            } catch (Fe) {}
          });
          try {
            (ce = (G = s.context) == null ? void 0 : G.dispose) == null || ce.call(G);
          } catch (pe) {}
          try {
            (Ce = s.disposeCheerio) == null || Ce.call(s);
          } catch (pe) {}
          s.context = null, s.disposeCheerio = null;
        }
      };
      var T = c.newFunction("__native_fetch", (...S) => {
        if (s.cleaned) throw new Error("Plugin execution ended");
        var W = JSON.parse(En(c, S[0]) || "{}"),
          M = String(a.executionId || "execution") + "-" + ++s.requestCounter,
          G = En(c, S[1]);
        W.requestId = M;
        var ce = c.newPromise();
        s.pending.set(M, {
          deferred: ce,
          abortToken: G
        }), G && s.abortTokens.set(G, M);
        try {
          Te({
            type: "fetch",
            requestId: M,
            payload: W
          });
        } catch (Ce) {
          D(M, null, Ce);
        }
        return ce.handle;
      });
      T.consume(S => c.setProp(c.global, "__native_fetch", S));
      var b = o1(),
        Y = c.evalCode(b, "nuvio-plugin-polyfill.js");
      c.unwrapResult(Y).dispose();
      var re = ";(function (root, factory) {\n	if (typeof exports === \"object\") {\n		// CommonJS\n		module.exports = exports = factory();\n	}\n	else if (typeof define === \"function\" && define.amd) {\n		// AMD\n		define([], factory);\n	}\n	else {\n		// Global (browser)\n		root.CryptoJS = factory();\n	}\n}(this, function () {\n\n	/*globals window, global, require*/\n\n	/**\n	 * CryptoJS core components.\n	 */\n	var CryptoJS = CryptoJS || (function (Math, undefined) {\n\n	    var crypto;\n\n	    // Native crypto from window (Browser)\n	    if (typeof window !== 'undefined' && window.crypto) {\n	        crypto = window.crypto;\n	    }\n\n	    // Native crypto in web worker (Browser)\n	    if (typeof self !== 'undefined' && self.crypto) {\n	        crypto = self.crypto;\n	    }\n\n	    // Native crypto from worker\n	    if (typeof globalThis !== 'undefined' && globalThis.crypto) {\n	        crypto = globalThis.crypto;\n	    }\n\n	    // Native (experimental IE 11) crypto from window (Browser)\n	    if (!crypto && typeof window !== 'undefined' && window.msCrypto) {\n	        crypto = window.msCrypto;\n	    }\n\n	    // Native crypto from global (NodeJS)\n	    if (!crypto && typeof global !== 'undefined' && global.crypto) {\n	        crypto = global.crypto;\n	    }\n\n	    // Native crypto import via require (NodeJS)\n	    if (!crypto && typeof require === 'function') {\n	        try {\n	            crypto = require('crypto');\n	        } catch (err) {}\n	    }\n\n	    /*\n	     * Cryptographically secure pseudorandom number generator\n	     *\n	     * As Math.random() is cryptographically not safe to use\n	     */\n	    var cryptoSecureRandomInt = function () {\n	        if (crypto) {\n	            // Use getRandomValues method (Browser)\n	            if (typeof crypto.getRandomValues === 'function') {\n	                try {\n	                    return crypto.getRandomValues(new Uint32Array(1))[0];\n	                } catch (err) {}\n	            }\n\n	            // Use randomBytes method (NodeJS)\n	            if (typeof crypto.randomBytes === 'function') {\n	                try {\n	                    return crypto.randomBytes(4).readInt32LE();\n	                } catch (err) {}\n	            }\n	        }\n\n	        throw new Error('Native crypto module could not be used to get secure random number.');\n	    };\n\n	    /*\n	     * Local polyfill of Object.create\n\n	     */\n	    var create = Object.create || (function () {\n	        function F() {}\n\n	        return function (obj) {\n	            var subtype;\n\n	            F.prototype = obj;\n\n	            subtype = new F();\n\n	            F.prototype = null;\n\n	            return subtype;\n	        };\n	    }());\n\n	    /**\n	     * CryptoJS namespace.\n	     */\n	    var C = {};\n\n	    /**\n	     * Library namespace.\n	     */\n	    var C_lib = C.lib = {};\n\n	    /**\n	     * Base object for prototypal inheritance.\n	     */\n	    var Base = C_lib.Base = (function () {\n\n\n	        return {\n	            /**\n	             * Creates a new object that inherits from this object.\n	             *\n	             * @param {Object} overrides Properties to copy into the new object.\n	             *\n	             * @return {Object} The new object.\n	             *\n	             * @static\n	             *\n	             * @example\n	             *\n	             *     var MyType = CryptoJS.lib.Base.extend({\n	             *         field: 'value',\n	             *\n	             *         method: function () {\n	             *         }\n	             *     });\n	             */\n	            extend: function (overrides) {\n	                // Spawn\n	                var subtype = create(this);\n\n	                // Augment\n	                if (overrides) {\n	                    subtype.mixIn(overrides);\n	                }\n\n	                // Create default initializer\n	                if (!subtype.hasOwnProperty('init') || this.init === subtype.init) {\n	                    subtype.init = function () {\n	                        subtype.$super.init.apply(this, arguments);\n	                    };\n	                }\n\n	                // Initializer's prototype is the subtype object\n	                subtype.init.prototype = subtype;\n\n	                // Reference supertype\n	                subtype.$super = this;\n\n	                return subtype;\n	            },\n\n	            /**\n	             * Extends this object and runs the init method.\n	             * Arguments to create() will be passed to init().\n	             *\n	             * @return {Object} The new object.\n	             *\n	             * @static\n	             *\n	             * @example\n	             *\n	             *     var instance = MyType.create();\n	             */\n	            create: function () {\n	                var instance = this.extend();\n	                instance.init.apply(instance, arguments);\n\n	                return instance;\n	            },\n\n	            /**\n	             * Initializes a newly created object.\n	             * Override this method to add some logic when your objects are created.\n	             *\n	             * @example\n	             *\n	             *     var MyType = CryptoJS.lib.Base.extend({\n	             *         init: function () {\n	             *             // ...\n	             *         }\n	             *     });\n	             */\n	            init: function () {\n	            },\n\n	            /**\n	             * Copies properties into this object.\n	             *\n	             * @param {Object} properties The properties to mix in.\n	             *\n	             * @example\n	             *\n	             *     MyType.mixIn({\n	             *         field: 'value'\n	             *     });\n	             */\n	            mixIn: function (properties) {\n	                for (var propertyName in properties) {\n	                    if (properties.hasOwnProperty(propertyName)) {\n	                        this[propertyName] = properties[propertyName];\n	                    }\n	                }\n\n	                // IE won't copy toString using the loop above\n	                if (properties.hasOwnProperty('toString')) {\n	                    this.toString = properties.toString;\n	                }\n	            },\n\n	            /**\n	             * Creates a copy of this object.\n	             *\n	             * @return {Object} The clone.\n	             *\n	             * @example\n	             *\n	             *     var clone = instance.clone();\n	             */\n	            clone: function () {\n	                return this.init.prototype.extend(this);\n	            }\n	        };\n	    }());\n\n	    /**\n	     * An array of 32-bit words.\n	     *\n	     * @property {Array} words The array of 32-bit words.\n	     * @property {number} sigBytes The number of significant bytes in this word array.\n	     */\n	    var WordArray = C_lib.WordArray = Base.extend({\n	        /**\n	         * Initializes a newly created word array.\n	         *\n	         * @param {Array} words (Optional) An array of 32-bit words.\n	         * @param {number} sigBytes (Optional) The number of significant bytes in the words.\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.lib.WordArray.create();\n	         *     var wordArray = CryptoJS.lib.WordArray.create([0x00010203, 0x04050607]);\n	         *     var wordArray = CryptoJS.lib.WordArray.create([0x00010203, 0x04050607], 6);\n	         */\n	        init: function (words, sigBytes) {\n	            words = this.words = words || [];\n\n	            if (sigBytes != undefined) {\n	                this.sigBytes = sigBytes;\n	            } else {\n	                this.sigBytes = words.length * 4;\n	            }\n	        },\n\n	        /**\n	         * Converts this word array to a string.\n	         *\n	         * @param {Encoder} encoder (Optional) The encoding strategy to use. Default: CryptoJS.enc.Hex\n	         *\n	         * @return {string} The stringified word array.\n	         *\n	         * @example\n	         *\n	         *     var string = wordArray + '';\n	         *     var string = wordArray.toString();\n	         *     var string = wordArray.toString(CryptoJS.enc.Utf8);\n	         */\n	        toString: function (encoder) {\n	            return (encoder || Hex).stringify(this);\n	        },\n\n	        /**\n	         * Concatenates a word array to this word array.\n	         *\n	         * @param {WordArray} wordArray The word array to append.\n	         *\n	         * @return {WordArray} This word array.\n	         *\n	         * @example\n	         *\n	         *     wordArray1.concat(wordArray2);\n	         */\n	        concat: function (wordArray) {\n	            // Shortcuts\n	            var thisWords = this.words;\n	            var thatWords = wordArray.words;\n	            var thisSigBytes = this.sigBytes;\n	            var thatSigBytes = wordArray.sigBytes;\n\n	            // Clamp excess bits\n	            this.clamp();\n\n	            // Concat\n	            if (thisSigBytes % 4) {\n	                // Copy one byte at a time\n	                for (var i = 0; i < thatSigBytes; i++) {\n	                    var thatByte = (thatWords[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;\n	                    thisWords[(thisSigBytes + i) >>> 2] |= thatByte << (24 - ((thisSigBytes + i) % 4) * 8);\n	                }\n	            } else {\n	                // Copy one word at a time\n	                for (var j = 0; j < thatSigBytes; j += 4) {\n	                    thisWords[(thisSigBytes + j) >>> 2] = thatWords[j >>> 2];\n	                }\n	            }\n	            this.sigBytes += thatSigBytes;\n\n	            // Chainable\n	            return this;\n	        },\n\n	        /**\n	         * Removes insignificant bits.\n	         *\n	         * @example\n	         *\n	         *     wordArray.clamp();\n	         */\n	        clamp: function () {\n	            // Shortcuts\n	            var words = this.words;\n	            var sigBytes = this.sigBytes;\n\n	            // Clamp\n	            words[sigBytes >>> 2] &= 0xffffffff << (32 - (sigBytes % 4) * 8);\n	            words.length = Math.ceil(sigBytes / 4);\n	        },\n\n	        /**\n	         * Creates a copy of this word array.\n	         *\n	         * @return {WordArray} The clone.\n	         *\n	         * @example\n	         *\n	         *     var clone = wordArray.clone();\n	         */\n	        clone: function () {\n	            var clone = Base.clone.call(this);\n	            clone.words = this.words.slice(0);\n\n	            return clone;\n	        },\n\n	        /**\n	         * Creates a word array filled with random bytes.\n	         *\n	         * @param {number} nBytes The number of random bytes to generate.\n	         *\n	         * @return {WordArray} The random word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.lib.WordArray.random(16);\n	         */\n	        random: function (nBytes) {\n	            var words = [];\n\n	            for (var i = 0; i < nBytes; i += 4) {\n	                words.push(cryptoSecureRandomInt());\n	            }\n\n	            return new WordArray.init(words, nBytes);\n	        }\n	    });\n\n	    /**\n	     * Encoder namespace.\n	     */\n	    var C_enc = C.enc = {};\n\n	    /**\n	     * Hex encoding strategy.\n	     */\n	    var Hex = C_enc.Hex = {\n	        /**\n	         * Converts a word array to a hex string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The hex string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var hexString = CryptoJS.enc.Hex.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n\n	            // Convert\n	            var hexChars = [];\n	            for (var i = 0; i < sigBytes; i++) {\n	                var bite = (words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;\n	                hexChars.push((bite >>> 4).toString(16));\n	                hexChars.push((bite & 0x0f).toString(16));\n	            }\n\n	            return hexChars.join('');\n	        },\n\n	        /**\n	         * Converts a hex string to a word array.\n	         *\n	         * @param {string} hexStr The hex string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Hex.parse(hexString);\n	         */\n	        parse: function (hexStr) {\n	            // Shortcut\n	            var hexStrLength = hexStr.length;\n\n	            // Convert\n	            var words = [];\n	            for (var i = 0; i < hexStrLength; i += 2) {\n	                words[i >>> 3] |= parseInt(hexStr.substr(i, 2), 16) << (24 - (i % 8) * 4);\n	            }\n\n	            return new WordArray.init(words, hexStrLength / 2);\n	        }\n	    };\n\n	    /**\n	     * Latin1 encoding strategy.\n	     */\n	    var Latin1 = C_enc.Latin1 = {\n	        /**\n	         * Converts a word array to a Latin1 string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The Latin1 string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var latin1String = CryptoJS.enc.Latin1.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n\n	            // Convert\n	            var latin1Chars = [];\n	            for (var i = 0; i < sigBytes; i++) {\n	                var bite = (words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;\n	                latin1Chars.push(String.fromCharCode(bite));\n	            }\n\n	            return latin1Chars.join('');\n	        },\n\n	        /**\n	         * Converts a Latin1 string to a word array.\n	         *\n	         * @param {string} latin1Str The Latin1 string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Latin1.parse(latin1String);\n	         */\n	        parse: function (latin1Str) {\n	            // Shortcut\n	            var latin1StrLength = latin1Str.length;\n\n	            // Convert\n	            var words = [];\n	            for (var i = 0; i < latin1StrLength; i++) {\n	                words[i >>> 2] |= (latin1Str.charCodeAt(i) & 0xff) << (24 - (i % 4) * 8);\n	            }\n\n	            return new WordArray.init(words, latin1StrLength);\n	        }\n	    };\n\n	    /**\n	     * UTF-8 encoding strategy.\n	     */\n	    var Utf8 = C_enc.Utf8 = {\n	        /**\n	         * Converts a word array to a UTF-8 string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The UTF-8 string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var utf8String = CryptoJS.enc.Utf8.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            try {\n	                return decodeURIComponent(escape(Latin1.stringify(wordArray)));\n	            } catch (e) {\n	                throw new Error('Malformed UTF-8 data');\n	            }\n	        },\n\n	        /**\n	         * Converts a UTF-8 string to a word array.\n	         *\n	         * @param {string} utf8Str The UTF-8 string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Utf8.parse(utf8String);\n	         */\n	        parse: function (utf8Str) {\n	            return Latin1.parse(unescape(encodeURIComponent(utf8Str)));\n	        }\n	    };\n\n	    /**\n	     * Abstract buffered block algorithm template.\n	     *\n	     * The property blockSize must be implemented in a concrete subtype.\n	     *\n	     * @property {number} _minBufferSize The number of blocks that should be kept unprocessed in the buffer. Default: 0\n	     */\n	    var BufferedBlockAlgorithm = C_lib.BufferedBlockAlgorithm = Base.extend({\n	        /**\n	         * Resets this block algorithm's data buffer to its initial state.\n	         *\n	         * @example\n	         *\n	         *     bufferedBlockAlgorithm.reset();\n	         */\n	        reset: function () {\n	            // Initial values\n	            this._data = new WordArray.init();\n	            this._nDataBytes = 0;\n	        },\n\n	        /**\n	         * Adds new data to this block algorithm's buffer.\n	         *\n	         * @param {WordArray|string} data The data to append. Strings are converted to a WordArray using UTF-8.\n	         *\n	         * @example\n	         *\n	         *     bufferedBlockAlgorithm._append('data');\n	         *     bufferedBlockAlgorithm._append(wordArray);\n	         */\n	        _append: function (data) {\n	            // Convert string to WordArray, else assume WordArray already\n	            if (typeof data == 'string') {\n	                data = Utf8.parse(data);\n	            }\n\n	            // Append\n	            this._data.concat(data);\n	            this._nDataBytes += data.sigBytes;\n	        },\n\n	        /**\n	         * Processes available data blocks.\n	         *\n	         * This method invokes _doProcessBlock(offset), which must be implemented by a concrete subtype.\n	         *\n	         * @param {boolean} doFlush Whether all blocks and partial blocks should be processed.\n	         *\n	         * @return {WordArray} The processed data.\n	         *\n	         * @example\n	         *\n	         *     var processedData = bufferedBlockAlgorithm._process();\n	         *     var processedData = bufferedBlockAlgorithm._process(!!'flush');\n	         */\n	        _process: function (doFlush) {\n	            var processedWords;\n\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n	            var dataSigBytes = data.sigBytes;\n	            var blockSize = this.blockSize;\n	            var blockSizeBytes = blockSize * 4;\n\n	            // Count blocks ready\n	            var nBlocksReady = dataSigBytes / blockSizeBytes;\n	            if (doFlush) {\n	                // Round up to include partial blocks\n	                nBlocksReady = Math.ceil(nBlocksReady);\n	            } else {\n	                // Round down to include only full blocks,\n	                // less the number of blocks that must remain in the buffer\n	                nBlocksReady = Math.max((nBlocksReady | 0) - this._minBufferSize, 0);\n	            }\n\n	            // Count words ready\n	            var nWordsReady = nBlocksReady * blockSize;\n\n	            // Count bytes ready\n	            var nBytesReady = Math.min(nWordsReady * 4, dataSigBytes);\n\n	            // Process blocks\n	            if (nWordsReady) {\n	                for (var offset = 0; offset < nWordsReady; offset += blockSize) {\n	                    // Perform concrete-algorithm logic\n	                    this._doProcessBlock(dataWords, offset);\n	                }\n\n	                // Remove processed words\n	                processedWords = dataWords.splice(0, nWordsReady);\n	                data.sigBytes -= nBytesReady;\n	            }\n\n	            // Return processed words\n	            return new WordArray.init(processedWords, nBytesReady);\n	        },\n\n	        /**\n	         * Creates a copy of this object.\n	         *\n	         * @return {Object} The clone.\n	         *\n	         * @example\n	         *\n	         *     var clone = bufferedBlockAlgorithm.clone();\n	         */\n	        clone: function () {\n	            var clone = Base.clone.call(this);\n	            clone._data = this._data.clone();\n\n	            return clone;\n	        },\n\n	        _minBufferSize: 0\n	    });\n\n	    /**\n	     * Abstract hasher template.\n	     *\n	     * @property {number} blockSize The number of 32-bit words this hasher operates on. Default: 16 (512 bits)\n	     */\n	    var Hasher = C_lib.Hasher = BufferedBlockAlgorithm.extend({\n	        /**\n	         * Configuration options.\n	         */\n	        cfg: Base.extend(),\n\n	        /**\n	         * Initializes a newly created hasher.\n	         *\n	         * @param {Object} cfg (Optional) The configuration options to use for this hash computation.\n	         *\n	         * @example\n	         *\n	         *     var hasher = CryptoJS.algo.SHA256.create();\n	         */\n	        init: function (cfg) {\n	            // Apply config defaults\n	            this.cfg = this.cfg.extend(cfg);\n\n	            // Set initial values\n	            this.reset();\n	        },\n\n	        /**\n	         * Resets this hasher to its initial state.\n	         *\n	         * @example\n	         *\n	         *     hasher.reset();\n	         */\n	        reset: function () {\n	            // Reset data buffer\n	            BufferedBlockAlgorithm.reset.call(this);\n\n	            // Perform concrete-hasher logic\n	            this._doReset();\n	        },\n\n	        /**\n	         * Updates this hasher with a message.\n	         *\n	         * @param {WordArray|string} messageUpdate The message to append.\n	         *\n	         * @return {Hasher} This hasher.\n	         *\n	         * @example\n	         *\n	         *     hasher.update('message');\n	         *     hasher.update(wordArray);\n	         */\n	        update: function (messageUpdate) {\n	            // Append\n	            this._append(messageUpdate);\n\n	            // Update the hash\n	            this._process();\n\n	            // Chainable\n	            return this;\n	        },\n\n	        /**\n	         * Finalizes the hash computation.\n	         * Note that the finalize operation is effectively a destructive, read-once operation.\n	         *\n	         * @param {WordArray|string} messageUpdate (Optional) A final message update.\n	         *\n	         * @return {WordArray} The hash.\n	         *\n	         * @example\n	         *\n	         *     var hash = hasher.finalize();\n	         *     var hash = hasher.finalize('message');\n	         *     var hash = hasher.finalize(wordArray);\n	         */\n	        finalize: function (messageUpdate) {\n	            // Final message update\n	            if (messageUpdate) {\n	                this._append(messageUpdate);\n	            }\n\n	            // Perform concrete-hasher logic\n	            var hash = this._doFinalize();\n\n	            return hash;\n	        },\n\n	        blockSize: 512/32,\n\n	        /**\n	         * Creates a shortcut function to a hasher's object interface.\n	         *\n	         * @param {Hasher} hasher The hasher to create a helper for.\n	         *\n	         * @return {Function} The shortcut function.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var SHA256 = CryptoJS.lib.Hasher._createHelper(CryptoJS.algo.SHA256);\n	         */\n	        _createHelper: function (hasher) {\n	            return function (message, cfg) {\n	                return new hasher.init(cfg).finalize(message);\n	            };\n	        },\n\n	        /**\n	         * Creates a shortcut function to the HMAC's object interface.\n	         *\n	         * @param {Hasher} hasher The hasher to use in this HMAC helper.\n	         *\n	         * @return {Function} The shortcut function.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var HmacSHA256 = CryptoJS.lib.Hasher._createHmacHelper(CryptoJS.algo.SHA256);\n	         */\n	        _createHmacHelper: function (hasher) {\n	            return function (message, key) {\n	                return new C_algo.HMAC.init(hasher, key).finalize(message);\n	            };\n	        }\n	    });\n\n	    /**\n	     * Algorithm namespace.\n	     */\n	    var C_algo = C.algo = {};\n\n	    return C;\n	}(Math));\n\n\n	(function (undefined) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Base = C_lib.Base;\n	    var X32WordArray = C_lib.WordArray;\n\n	    /**\n	     * x64 namespace.\n	     */\n	    var C_x64 = C.x64 = {};\n\n	    /**\n	     * A 64-bit word.\n	     */\n	    var X64Word = C_x64.Word = Base.extend({\n	        /**\n	         * Initializes a newly created 64-bit word.\n	         *\n	         * @param {number} high The high 32 bits.\n	         * @param {number} low The low 32 bits.\n	         *\n	         * @example\n	         *\n	         *     var x64Word = CryptoJS.x64.Word.create(0x00010203, 0x04050607);\n	         */\n	        init: function (high, low) {\n	            this.high = high;\n	            this.low = low;\n	        }\n\n	        /**\n	         * Bitwise NOTs this word.\n	         *\n	         * @return {X64Word} A new x64-Word object after negating.\n	         *\n	         * @example\n	         *\n	         *     var negated = x64Word.not();\n	         */\n	        // not: function () {\n	            // var high = ~this.high;\n	            // var low = ~this.low;\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Bitwise ANDs this word with the passed word.\n	         *\n	         * @param {X64Word} word The x64-Word to AND with this word.\n	         *\n	         * @return {X64Word} A new x64-Word object after ANDing.\n	         *\n	         * @example\n	         *\n	         *     var anded = x64Word.and(anotherX64Word);\n	         */\n	        // and: function (word) {\n	            // var high = this.high & word.high;\n	            // var low = this.low & word.low;\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Bitwise ORs this word with the passed word.\n	         *\n	         * @param {X64Word} word The x64-Word to OR with this word.\n	         *\n	         * @return {X64Word} A new x64-Word object after ORing.\n	         *\n	         * @example\n	         *\n	         *     var ored = x64Word.or(anotherX64Word);\n	         */\n	        // or: function (word) {\n	            // var high = this.high | word.high;\n	            // var low = this.low | word.low;\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Bitwise XORs this word with the passed word.\n	         *\n	         * @param {X64Word} word The x64-Word to XOR with this word.\n	         *\n	         * @return {X64Word} A new x64-Word object after XORing.\n	         *\n	         * @example\n	         *\n	         *     var xored = x64Word.xor(anotherX64Word);\n	         */\n	        // xor: function (word) {\n	            // var high = this.high ^ word.high;\n	            // var low = this.low ^ word.low;\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Shifts this word n bits to the left.\n	         *\n	         * @param {number} n The number of bits to shift.\n	         *\n	         * @return {X64Word} A new x64-Word object after shifting.\n	         *\n	         * @example\n	         *\n	         *     var shifted = x64Word.shiftL(25);\n	         */\n	        // shiftL: function (n) {\n	            // if (n < 32) {\n	                // var high = (this.high << n) | (this.low >>> (32 - n));\n	                // var low = this.low << n;\n	            // } else {\n	                // var high = this.low << (n - 32);\n	                // var low = 0;\n	            // }\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Shifts this word n bits to the right.\n	         *\n	         * @param {number} n The number of bits to shift.\n	         *\n	         * @return {X64Word} A new x64-Word object after shifting.\n	         *\n	         * @example\n	         *\n	         *     var shifted = x64Word.shiftR(7);\n	         */\n	        // shiftR: function (n) {\n	            // if (n < 32) {\n	                // var low = (this.low >>> n) | (this.high << (32 - n));\n	                // var high = this.high >>> n;\n	            // } else {\n	                // var low = this.high >>> (n - 32);\n	                // var high = 0;\n	            // }\n\n	            // return X64Word.create(high, low);\n	        // },\n\n	        /**\n	         * Rotates this word n bits to the left.\n	         *\n	         * @param {number} n The number of bits to rotate.\n	         *\n	         * @return {X64Word} A new x64-Word object after rotating.\n	         *\n	         * @example\n	         *\n	         *     var rotated = x64Word.rotL(25);\n	         */\n	        // rotL: function (n) {\n	            // return this.shiftL(n).or(this.shiftR(64 - n));\n	        // },\n\n	        /**\n	         * Rotates this word n bits to the right.\n	         *\n	         * @param {number} n The number of bits to rotate.\n	         *\n	         * @return {X64Word} A new x64-Word object after rotating.\n	         *\n	         * @example\n	         *\n	         *     var rotated = x64Word.rotR(7);\n	         */\n	        // rotR: function (n) {\n	            // return this.shiftR(n).or(this.shiftL(64 - n));\n	        // },\n\n	        /**\n	         * Adds this word with the passed word.\n	         *\n	         * @param {X64Word} word The x64-Word to add with this word.\n	         *\n	         * @return {X64Word} A new x64-Word object after adding.\n	         *\n	         * @example\n	         *\n	         *     var added = x64Word.add(anotherX64Word);\n	         */\n	        // add: function (word) {\n	            // var low = (this.low + word.low) | 0;\n	            // var carry = (low >>> 0) < (this.low >>> 0) ? 1 : 0;\n	            // var high = (this.high + word.high + carry) | 0;\n\n	            // return X64Word.create(high, low);\n	        // }\n	    });\n\n	    /**\n	     * An array of 64-bit words.\n	     *\n	     * @property {Array} words The array of CryptoJS.x64.Word objects.\n	     * @property {number} sigBytes The number of significant bytes in this word array.\n	     */\n	    var X64WordArray = C_x64.WordArray = Base.extend({\n	        /**\n	         * Initializes a newly created word array.\n	         *\n	         * @param {Array} words (Optional) An array of CryptoJS.x64.Word objects.\n	         * @param {number} sigBytes (Optional) The number of significant bytes in the words.\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.x64.WordArray.create();\n	         *\n	         *     var wordArray = CryptoJS.x64.WordArray.create([\n	         *         CryptoJS.x64.Word.create(0x00010203, 0x04050607),\n	         *         CryptoJS.x64.Word.create(0x18191a1b, 0x1c1d1e1f)\n	         *     ]);\n	         *\n	         *     var wordArray = CryptoJS.x64.WordArray.create([\n	         *         CryptoJS.x64.Word.create(0x00010203, 0x04050607),\n	         *         CryptoJS.x64.Word.create(0x18191a1b, 0x1c1d1e1f)\n	         *     ], 10);\n	         */\n	        init: function (words, sigBytes) {\n	            words = this.words = words || [];\n\n	            if (sigBytes != undefined) {\n	                this.sigBytes = sigBytes;\n	            } else {\n	                this.sigBytes = words.length * 8;\n	            }\n	        },\n\n	        /**\n	         * Converts this 64-bit word array to a 32-bit word array.\n	         *\n	         * @return {CryptoJS.lib.WordArray} This word array's data as a 32-bit word array.\n	         *\n	         * @example\n	         *\n	         *     var x32WordArray = x64WordArray.toX32();\n	         */\n	        toX32: function () {\n	            // Shortcuts\n	            var x64Words = this.words;\n	            var x64WordsLength = x64Words.length;\n\n	            // Convert\n	            var x32Words = [];\n	            for (var i = 0; i < x64WordsLength; i++) {\n	                var x64Word = x64Words[i];\n	                x32Words.push(x64Word.high);\n	                x32Words.push(x64Word.low);\n	            }\n\n	            return X32WordArray.create(x32Words, this.sigBytes);\n	        },\n\n	        /**\n	         * Creates a copy of this word array.\n	         *\n	         * @return {X64WordArray} The clone.\n	         *\n	         * @example\n	         *\n	         *     var clone = x64WordArray.clone();\n	         */\n	        clone: function () {\n	            var clone = Base.clone.call(this);\n\n	            // Clone \"words\" array\n	            var words = clone.words = this.words.slice(0);\n\n	            // Clone each X64Word object\n	            var wordsLength = words.length;\n	            for (var i = 0; i < wordsLength; i++) {\n	                words[i] = words[i].clone();\n	            }\n\n	            return clone;\n	        }\n	    });\n	}());\n\n\n	(function () {\n	    // Check if typed arrays are supported\n	    if (typeof ArrayBuffer != 'function') {\n	        return;\n	    }\n\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n\n	    // Reference original init\n	    var superInit = WordArray.init;\n\n	    // Augment WordArray.init to handle typed arrays\n	    var subInit = WordArray.init = function (typedArray) {\n	        // Convert buffers to uint8\n	        if (typedArray instanceof ArrayBuffer) {\n	            typedArray = new Uint8Array(typedArray);\n	        }\n\n	        // Convert other array views to uint8\n	        if (\n	            typedArray instanceof Int8Array ||\n	            (typeof Uint8ClampedArray !== \"undefined\" && typedArray instanceof Uint8ClampedArray) ||\n	            typedArray instanceof Int16Array ||\n	            typedArray instanceof Uint16Array ||\n	            typedArray instanceof Int32Array ||\n	            typedArray instanceof Uint32Array ||\n	            typedArray instanceof Float32Array ||\n	            typedArray instanceof Float64Array\n	        ) {\n	            typedArray = new Uint8Array(typedArray.buffer, typedArray.byteOffset, typedArray.byteLength);\n	        }\n\n	        // Handle Uint8Array\n	        if (typedArray instanceof Uint8Array) {\n	            // Shortcut\n	            var typedArrayByteLength = typedArray.byteLength;\n\n	            // Extract bytes\n	            var words = [];\n	            for (var i = 0; i < typedArrayByteLength; i++) {\n	                words[i >>> 2] |= typedArray[i] << (24 - (i % 4) * 8);\n	            }\n\n	            // Initialize this word array\n	            superInit.call(this, words, typedArrayByteLength);\n	        } else {\n	            // Else call normal init\n	            superInit.apply(this, arguments);\n	        }\n	    };\n\n	    subInit.prototype = WordArray;\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var C_enc = C.enc;\n\n	    /**\n	     * UTF-16 BE encoding strategy.\n	     */\n	    var Utf16BE = C_enc.Utf16 = C_enc.Utf16BE = {\n	        /**\n	         * Converts a word array to a UTF-16 BE string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The UTF-16 BE string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var utf16String = CryptoJS.enc.Utf16.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n\n	            // Convert\n	            var utf16Chars = [];\n	            for (var i = 0; i < sigBytes; i += 2) {\n	                var codePoint = (words[i >>> 2] >>> (16 - (i % 4) * 8)) & 0xffff;\n	                utf16Chars.push(String.fromCharCode(codePoint));\n	            }\n\n	            return utf16Chars.join('');\n	        },\n\n	        /**\n	         * Converts a UTF-16 BE string to a word array.\n	         *\n	         * @param {string} utf16Str The UTF-16 BE string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Utf16.parse(utf16String);\n	         */\n	        parse: function (utf16Str) {\n	            // Shortcut\n	            var utf16StrLength = utf16Str.length;\n\n	            // Convert\n	            var words = [];\n	            for (var i = 0; i < utf16StrLength; i++) {\n	                words[i >>> 1] |= utf16Str.charCodeAt(i) << (16 - (i % 2) * 16);\n	            }\n\n	            return WordArray.create(words, utf16StrLength * 2);\n	        }\n	    };\n\n	    /**\n	     * UTF-16 LE encoding strategy.\n	     */\n	    C_enc.Utf16LE = {\n	        /**\n	         * Converts a word array to a UTF-16 LE string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The UTF-16 LE string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var utf16Str = CryptoJS.enc.Utf16LE.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n\n	            // Convert\n	            var utf16Chars = [];\n	            for (var i = 0; i < sigBytes; i += 2) {\n	                var codePoint = swapEndian((words[i >>> 2] >>> (16 - (i % 4) * 8)) & 0xffff);\n	                utf16Chars.push(String.fromCharCode(codePoint));\n	            }\n\n	            return utf16Chars.join('');\n	        },\n\n	        /**\n	         * Converts a UTF-16 LE string to a word array.\n	         *\n	         * @param {string} utf16Str The UTF-16 LE string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Utf16LE.parse(utf16Str);\n	         */\n	        parse: function (utf16Str) {\n	            // Shortcut\n	            var utf16StrLength = utf16Str.length;\n\n	            // Convert\n	            var words = [];\n	            for (var i = 0; i < utf16StrLength; i++) {\n	                words[i >>> 1] |= swapEndian(utf16Str.charCodeAt(i) << (16 - (i % 2) * 16));\n	            }\n\n	            return WordArray.create(words, utf16StrLength * 2);\n	        }\n	    };\n\n	    function swapEndian(word) {\n	        return ((word << 8) & 0xff00ff00) | ((word >>> 8) & 0x00ff00ff);\n	    }\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var C_enc = C.enc;\n\n	    /**\n	     * Base64 encoding strategy.\n	     */\n	    var Base64 = C_enc.Base64 = {\n	        /**\n	         * Converts a word array to a Base64 string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @return {string} The Base64 string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var base64String = CryptoJS.enc.Base64.stringify(wordArray);\n	         */\n	        stringify: function (wordArray) {\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n	            var map = this._map;\n\n	            // Clamp excess bits\n	            wordArray.clamp();\n\n	            // Convert\n	            var base64Chars = [];\n	            for (var i = 0; i < sigBytes; i += 3) {\n	                var byte1 = (words[i >>> 2]       >>> (24 - (i % 4) * 8))       & 0xff;\n	                var byte2 = (words[(i + 1) >>> 2] >>> (24 - ((i + 1) % 4) * 8)) & 0xff;\n	                var byte3 = (words[(i + 2) >>> 2] >>> (24 - ((i + 2) % 4) * 8)) & 0xff;\n\n	                var triplet = (byte1 << 16) | (byte2 << 8) | byte3;\n\n	                for (var j = 0; (j < 4) && (i + j * 0.75 < sigBytes); j++) {\n	                    base64Chars.push(map.charAt((triplet >>> (6 * (3 - j))) & 0x3f));\n	                }\n	            }\n\n	            // Add padding\n	            var paddingChar = map.charAt(64);\n	            if (paddingChar) {\n	                while (base64Chars.length % 4) {\n	                    base64Chars.push(paddingChar);\n	                }\n	            }\n\n	            return base64Chars.join('');\n	        },\n\n	        /**\n	         * Converts a Base64 string to a word array.\n	         *\n	         * @param {string} base64Str The Base64 string.\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Base64.parse(base64String);\n	         */\n	        parse: function (base64Str) {\n	            // Shortcuts\n	            var base64StrLength = base64Str.length;\n	            var map = this._map;\n	            var reverseMap = this._reverseMap;\n\n	            if (!reverseMap) {\n	                    reverseMap = this._reverseMap = [];\n	                    for (var j = 0; j < map.length; j++) {\n	                        reverseMap[map.charCodeAt(j)] = j;\n	                    }\n	            }\n\n	            // Ignore padding\n	            var paddingChar = map.charAt(64);\n	            if (paddingChar) {\n	                var paddingIndex = base64Str.indexOf(paddingChar);\n	                if (paddingIndex !== -1) {\n	                    base64StrLength = paddingIndex;\n	                }\n	            }\n\n	            // Convert\n	            return parseLoop(base64Str, base64StrLength, reverseMap);\n\n	        },\n\n	        _map: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='\n	    };\n\n	    function parseLoop(base64Str, base64StrLength, reverseMap) {\n	      var words = [];\n	      var nBytes = 0;\n	      for (var i = 0; i < base64StrLength; i++) {\n	          if (i % 4) {\n	              var bits1 = reverseMap[base64Str.charCodeAt(i - 1)] << ((i % 4) * 2);\n	              var bits2 = reverseMap[base64Str.charCodeAt(i)] >>> (6 - (i % 4) * 2);\n	              var bitsCombined = bits1 | bits2;\n	              words[nBytes >>> 2] |= bitsCombined << (24 - (nBytes % 4) * 8);\n	              nBytes++;\n	          }\n	      }\n	      return WordArray.create(words, nBytes);\n	    }\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var C_enc = C.enc;\n\n	    /**\n	     * Base64url encoding strategy.\n	     */\n	    var Base64url = C_enc.Base64url = {\n	        /**\n	         * Converts a word array to a Base64url string.\n	         *\n	         * @param {WordArray} wordArray The word array.\n	         *\n	         * @param {boolean} urlSafe Whether to use url safe\n	         *\n	         * @return {string} The Base64url string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var base64String = CryptoJS.enc.Base64url.stringify(wordArray);\n	         */\n	        stringify: function (wordArray, urlSafe) {\n	            if (urlSafe === undefined) {\n	                urlSafe = true\n	            }\n	            // Shortcuts\n	            var words = wordArray.words;\n	            var sigBytes = wordArray.sigBytes;\n	            var map = urlSafe ? this._safe_map : this._map;\n\n	            // Clamp excess bits\n	            wordArray.clamp();\n\n	            // Convert\n	            var base64Chars = [];\n	            for (var i = 0; i < sigBytes; i += 3) {\n	                var byte1 = (words[i >>> 2]       >>> (24 - (i % 4) * 8))       & 0xff;\n	                var byte2 = (words[(i + 1) >>> 2] >>> (24 - ((i + 1) % 4) * 8)) & 0xff;\n	                var byte3 = (words[(i + 2) >>> 2] >>> (24 - ((i + 2) % 4) * 8)) & 0xff;\n\n	                var triplet = (byte1 << 16) | (byte2 << 8) | byte3;\n\n	                for (var j = 0; (j < 4) && (i + j * 0.75 < sigBytes); j++) {\n	                    base64Chars.push(map.charAt((triplet >>> (6 * (3 - j))) & 0x3f));\n	                }\n	            }\n\n	            // Add padding\n	            var paddingChar = map.charAt(64);\n	            if (paddingChar) {\n	                while (base64Chars.length % 4) {\n	                    base64Chars.push(paddingChar);\n	                }\n	            }\n\n	            return base64Chars.join('');\n	        },\n\n	        /**\n	         * Converts a Base64url string to a word array.\n	         *\n	         * @param {string} base64Str The Base64url string.\n	         *\n	         * @param {boolean} urlSafe Whether to use url safe\n	         *\n	         * @return {WordArray} The word array.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var wordArray = CryptoJS.enc.Base64url.parse(base64String);\n	         */\n	        parse: function (base64Str, urlSafe) {\n	            if (urlSafe === undefined) {\n	                urlSafe = true\n	            }\n\n	            // Shortcuts\n	            var base64StrLength = base64Str.length;\n	            var map = urlSafe ? this._safe_map : this._map;\n	            var reverseMap = this._reverseMap;\n\n	            if (!reverseMap) {\n	                reverseMap = this._reverseMap = [];\n	                for (var j = 0; j < map.length; j++) {\n	                    reverseMap[map.charCodeAt(j)] = j;\n	                }\n	            }\n\n	            // Ignore padding\n	            var paddingChar = map.charAt(64);\n	            if (paddingChar) {\n	                var paddingIndex = base64Str.indexOf(paddingChar);\n	                if (paddingIndex !== -1) {\n	                    base64StrLength = paddingIndex;\n	                }\n	            }\n\n	            // Convert\n	            return parseLoop(base64Str, base64StrLength, reverseMap);\n\n	        },\n\n	        _map: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=',\n	        _safe_map: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_',\n	    };\n\n	    function parseLoop(base64Str, base64StrLength, reverseMap) {\n	        var words = [];\n	        var nBytes = 0;\n	        for (var i = 0; i < base64StrLength; i++) {\n	            if (i % 4) {\n	                var bits1 = reverseMap[base64Str.charCodeAt(i - 1)] << ((i % 4) * 2);\n	                var bits2 = reverseMap[base64Str.charCodeAt(i)] >>> (6 - (i % 4) * 2);\n	                var bitsCombined = bits1 | bits2;\n	                words[nBytes >>> 2] |= bitsCombined << (24 - (nBytes % 4) * 8);\n	                nBytes++;\n	            }\n	        }\n	        return WordArray.create(words, nBytes);\n	    }\n	}());\n\n\n	(function (Math) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var Hasher = C_lib.Hasher;\n	    var C_algo = C.algo;\n\n	    // Constants table\n	    var T = [];\n\n	    // Compute constants\n	    (function () {\n	        for (var i = 0; i < 64; i++) {\n	            T[i] = (Math.abs(Math.sin(i + 1)) * 0x100000000) | 0;\n	        }\n	    }());\n\n	    /**\n	     * MD5 hash algorithm.\n	     */\n	    var MD5 = C_algo.MD5 = Hasher.extend({\n	        _doReset: function () {\n	            this._hash = new WordArray.init([\n	                0x67452301, 0xefcdab89,\n	                0x98badcfe, 0x10325476\n	            ]);\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Swap endian\n	            for (var i = 0; i < 16; i++) {\n	                // Shortcuts\n	                var offset_i = offset + i;\n	                var M_offset_i = M[offset_i];\n\n	                M[offset_i] = (\n	                    (((M_offset_i << 8)  | (M_offset_i >>> 24)) & 0x00ff00ff) |\n	                    (((M_offset_i << 24) | (M_offset_i >>> 8))  & 0xff00ff00)\n	                );\n	            }\n\n	            // Shortcuts\n	            var H = this._hash.words;\n\n	            var M_offset_0  = M[offset + 0];\n	            var M_offset_1  = M[offset + 1];\n	            var M_offset_2  = M[offset + 2];\n	            var M_offset_3  = M[offset + 3];\n	            var M_offset_4  = M[offset + 4];\n	            var M_offset_5  = M[offset + 5];\n	            var M_offset_6  = M[offset + 6];\n	            var M_offset_7  = M[offset + 7];\n	            var M_offset_8  = M[offset + 8];\n	            var M_offset_9  = M[offset + 9];\n	            var M_offset_10 = M[offset + 10];\n	            var M_offset_11 = M[offset + 11];\n	            var M_offset_12 = M[offset + 12];\n	            var M_offset_13 = M[offset + 13];\n	            var M_offset_14 = M[offset + 14];\n	            var M_offset_15 = M[offset + 15];\n\n	            // Working variables\n	            var a = H[0];\n	            var b = H[1];\n	            var c = H[2];\n	            var d = H[3];\n\n	            // Computation\n	            a = FF(a, b, c, d, M_offset_0,  7,  T[0]);\n	            d = FF(d, a, b, c, M_offset_1,  12, T[1]);\n	            c = FF(c, d, a, b, M_offset_2,  17, T[2]);\n	            b = FF(b, c, d, a, M_offset_3,  22, T[3]);\n	            a = FF(a, b, c, d, M_offset_4,  7,  T[4]);\n	            d = FF(d, a, b, c, M_offset_5,  12, T[5]);\n	            c = FF(c, d, a, b, M_offset_6,  17, T[6]);\n	            b = FF(b, c, d, a, M_offset_7,  22, T[7]);\n	            a = FF(a, b, c, d, M_offset_8,  7,  T[8]);\n	            d = FF(d, a, b, c, M_offset_9,  12, T[9]);\n	            c = FF(c, d, a, b, M_offset_10, 17, T[10]);\n	            b = FF(b, c, d, a, M_offset_11, 22, T[11]);\n	            a = FF(a, b, c, d, M_offset_12, 7,  T[12]);\n	            d = FF(d, a, b, c, M_offset_13, 12, T[13]);\n	            c = FF(c, d, a, b, M_offset_14, 17, T[14]);\n	            b = FF(b, c, d, a, M_offset_15, 22, T[15]);\n\n	            a = GG(a, b, c, d, M_offset_1,  5,  T[16]);\n	            d = GG(d, a, b, c, M_offset_6,  9,  T[17]);\n	            c = GG(c, d, a, b, M_offset_11, 14, T[18]);\n	            b = GG(b, c, d, a, M_offset_0,  20, T[19]);\n	            a = GG(a, b, c, d, M_offset_5,  5,  T[20]);\n	            d = GG(d, a, b, c, M_offset_10, 9,  T[21]);\n	            c = GG(c, d, a, b, M_offset_15, 14, T[22]);\n	            b = GG(b, c, d, a, M_offset_4,  20, T[23]);\n	            a = GG(a, b, c, d, M_offset_9,  5,  T[24]);\n	            d = GG(d, a, b, c, M_offset_14, 9,  T[25]);\n	            c = GG(c, d, a, b, M_offset_3,  14, T[26]);\n	            b = GG(b, c, d, a, M_offset_8,  20, T[27]);\n	            a = GG(a, b, c, d, M_offset_13, 5,  T[28]);\n	            d = GG(d, a, b, c, M_offset_2,  9,  T[29]);\n	            c = GG(c, d, a, b, M_offset_7,  14, T[30]);\n	            b = GG(b, c, d, a, M_offset_12, 20, T[31]);\n\n	            a = HH(a, b, c, d, M_offset_5,  4,  T[32]);\n	            d = HH(d, a, b, c, M_offset_8,  11, T[33]);\n	            c = HH(c, d, a, b, M_offset_11, 16, T[34]);\n	            b = HH(b, c, d, a, M_offset_14, 23, T[35]);\n	            a = HH(a, b, c, d, M_offset_1,  4,  T[36]);\n	            d = HH(d, a, b, c, M_offset_4,  11, T[37]);\n	            c = HH(c, d, a, b, M_offset_7,  16, T[38]);\n	            b = HH(b, c, d, a, M_offset_10, 23, T[39]);\n	            a = HH(a, b, c, d, M_offset_13, 4,  T[40]);\n	            d = HH(d, a, b, c, M_offset_0,  11, T[41]);\n	            c = HH(c, d, a, b, M_offset_3,  16, T[42]);\n	            b = HH(b, c, d, a, M_offset_6,  23, T[43]);\n	            a = HH(a, b, c, d, M_offset_9,  4,  T[44]);\n	            d = HH(d, a, b, c, M_offset_12, 11, T[45]);\n	            c = HH(c, d, a, b, M_offset_15, 16, T[46]);\n	            b = HH(b, c, d, a, M_offset_2,  23, T[47]);\n\n	            a = II(a, b, c, d, M_offset_0,  6,  T[48]);\n	            d = II(d, a, b, c, M_offset_7,  10, T[49]);\n	            c = II(c, d, a, b, M_offset_14, 15, T[50]);\n	            b = II(b, c, d, a, M_offset_5,  21, T[51]);\n	            a = II(a, b, c, d, M_offset_12, 6,  T[52]);\n	            d = II(d, a, b, c, M_offset_3,  10, T[53]);\n	            c = II(c, d, a, b, M_offset_10, 15, T[54]);\n	            b = II(b, c, d, a, M_offset_1,  21, T[55]);\n	            a = II(a, b, c, d, M_offset_8,  6,  T[56]);\n	            d = II(d, a, b, c, M_offset_15, 10, T[57]);\n	            c = II(c, d, a, b, M_offset_6,  15, T[58]);\n	            b = II(b, c, d, a, M_offset_13, 21, T[59]);\n	            a = II(a, b, c, d, M_offset_4,  6,  T[60]);\n	            d = II(d, a, b, c, M_offset_11, 10, T[61]);\n	            c = II(c, d, a, b, M_offset_2,  15, T[62]);\n	            b = II(b, c, d, a, M_offset_9,  21, T[63]);\n\n	            // Intermediate hash value\n	            H[0] = (H[0] + a) | 0;\n	            H[1] = (H[1] + b) | 0;\n	            H[2] = (H[2] + c) | 0;\n	            H[3] = (H[3] + d) | 0;\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x80 << (24 - nBitsLeft % 32);\n\n	            var nBitsTotalH = Math.floor(nBitsTotal / 0x100000000);\n	            var nBitsTotalL = nBitsTotal;\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 15] = (\n	                (((nBitsTotalH << 8)  | (nBitsTotalH >>> 24)) & 0x00ff00ff) |\n	                (((nBitsTotalH << 24) | (nBitsTotalH >>> 8))  & 0xff00ff00)\n	            );\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 14] = (\n	                (((nBitsTotalL << 8)  | (nBitsTotalL >>> 24)) & 0x00ff00ff) |\n	                (((nBitsTotalL << 24) | (nBitsTotalL >>> 8))  & 0xff00ff00)\n	            );\n\n	            data.sigBytes = (dataWords.length + 1) * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Shortcuts\n	            var hash = this._hash;\n	            var H = hash.words;\n\n	            // Swap endian\n	            for (var i = 0; i < 4; i++) {\n	                // Shortcut\n	                var H_i = H[i];\n\n	                H[i] = (((H_i << 8)  | (H_i >>> 24)) & 0x00ff00ff) |\n	                       (((H_i << 24) | (H_i >>> 8))  & 0xff00ff00);\n	            }\n\n	            // Return final computed hash\n	            return hash;\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n	            clone._hash = this._hash.clone();\n\n	            return clone;\n	        }\n	    });\n\n	    function FF(a, b, c, d, x, s, t) {\n	        var n = a + ((b & c) | (~b & d)) + x + t;\n	        return ((n << s) | (n >>> (32 - s))) + b;\n	    }\n\n	    function GG(a, b, c, d, x, s, t) {\n	        var n = a + ((b & d) | (c & ~d)) + x + t;\n	        return ((n << s) | (n >>> (32 - s))) + b;\n	    }\n\n	    function HH(a, b, c, d, x, s, t) {\n	        var n = a + (b ^ c ^ d) + x + t;\n	        return ((n << s) | (n >>> (32 - s))) + b;\n	    }\n\n	    function II(a, b, c, d, x, s, t) {\n	        var n = a + (c ^ (b | ~d)) + x + t;\n	        return ((n << s) | (n >>> (32 - s))) + b;\n	    }\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.MD5('message');\n	     *     var hash = CryptoJS.MD5(wordArray);\n	     */\n	    C.MD5 = Hasher._createHelper(MD5);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacMD5(message, key);\n	     */\n	    C.HmacMD5 = Hasher._createHmacHelper(MD5);\n	}(Math));\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var Hasher = C_lib.Hasher;\n	    var C_algo = C.algo;\n\n	    // Reusable object\n	    var W = [];\n\n	    /**\n	     * SHA-1 hash algorithm.\n	     */\n	    var SHA1 = C_algo.SHA1 = Hasher.extend({\n	        _doReset: function () {\n	            this._hash = new WordArray.init([\n	                0x67452301, 0xefcdab89,\n	                0x98badcfe, 0x10325476,\n	                0xc3d2e1f0\n	            ]);\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcut\n	            var H = this._hash.words;\n\n	            // Working variables\n	            var a = H[0];\n	            var b = H[1];\n	            var c = H[2];\n	            var d = H[3];\n	            var e = H[4];\n\n	            // Computation\n	            for (var i = 0; i < 80; i++) {\n	                if (i < 16) {\n	                    W[i] = M[offset + i] | 0;\n	                } else {\n	                    var n = W[i - 3] ^ W[i - 8] ^ W[i - 14] ^ W[i - 16];\n	                    W[i] = (n << 1) | (n >>> 31);\n	                }\n\n	                var t = ((a << 5) | (a >>> 27)) + e + W[i];\n	                if (i < 20) {\n	                    t += ((b & c) | (~b & d)) + 0x5a827999;\n	                } else if (i < 40) {\n	                    t += (b ^ c ^ d) + 0x6ed9eba1;\n	                } else if (i < 60) {\n	                    t += ((b & c) | (b & d) | (c & d)) - 0x70e44324;\n	                } else /* if (i < 80) */ {\n	                    t += (b ^ c ^ d) - 0x359d3e2a;\n	                }\n\n	                e = d;\n	                d = c;\n	                c = (b << 30) | (b >>> 2);\n	                b = a;\n	                a = t;\n	            }\n\n	            // Intermediate hash value\n	            H[0] = (H[0] + a) | 0;\n	            H[1] = (H[1] + b) | 0;\n	            H[2] = (H[2] + c) | 0;\n	            H[3] = (H[3] + d) | 0;\n	            H[4] = (H[4] + e) | 0;\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x80 << (24 - nBitsLeft % 32);\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 14] = Math.floor(nBitsTotal / 0x100000000);\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 15] = nBitsTotal;\n	            data.sigBytes = dataWords.length * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Return final computed hash\n	            return this._hash;\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n	            clone._hash = this._hash.clone();\n\n	            return clone;\n	        }\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA1('message');\n	     *     var hash = CryptoJS.SHA1(wordArray);\n	     */\n	    C.SHA1 = Hasher._createHelper(SHA1);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA1(message, key);\n	     */\n	    C.HmacSHA1 = Hasher._createHmacHelper(SHA1);\n	}());\n\n\n	(function (Math) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var Hasher = C_lib.Hasher;\n	    var C_algo = C.algo;\n\n	    // Initialization and round constants tables\n	    var H = [];\n	    var K = [];\n\n	    // Compute constants\n	    (function () {\n	        function isPrime(n) {\n	            var sqrtN = Math.sqrt(n);\n	            for (var factor = 2; factor <= sqrtN; factor++) {\n	                if (!(n % factor)) {\n	                    return false;\n	                }\n	            }\n\n	            return true;\n	        }\n\n	        function getFractionalBits(n) {\n	            return ((n - (n | 0)) * 0x100000000) | 0;\n	        }\n\n	        var n = 2;\n	        var nPrime = 0;\n	        while (nPrime < 64) {\n	            if (isPrime(n)) {\n	                if (nPrime < 8) {\n	                    H[nPrime] = getFractionalBits(Math.pow(n, 1 / 2));\n	                }\n	                K[nPrime] = getFractionalBits(Math.pow(n, 1 / 3));\n\n	                nPrime++;\n	            }\n\n	            n++;\n	        }\n	    }());\n\n	    // Reusable object\n	    var W = [];\n\n	    /**\n	     * SHA-256 hash algorithm.\n	     */\n	    var SHA256 = C_algo.SHA256 = Hasher.extend({\n	        _doReset: function () {\n	            this._hash = new WordArray.init(H.slice(0));\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcut\n	            var H = this._hash.words;\n\n	            // Working variables\n	            var a = H[0];\n	            var b = H[1];\n	            var c = H[2];\n	            var d = H[3];\n	            var e = H[4];\n	            var f = H[5];\n	            var g = H[6];\n	            var h = H[7];\n\n	            // Computation\n	            for (var i = 0; i < 64; i++) {\n	                if (i < 16) {\n	                    W[i] = M[offset + i] | 0;\n	                } else {\n	                    var gamma0x = W[i - 15];\n	                    var gamma0  = ((gamma0x << 25) | (gamma0x >>> 7))  ^\n	                                  ((gamma0x << 14) | (gamma0x >>> 18)) ^\n	                                   (gamma0x >>> 3);\n\n	                    var gamma1x = W[i - 2];\n	                    var gamma1  = ((gamma1x << 15) | (gamma1x >>> 17)) ^\n	                                  ((gamma1x << 13) | (gamma1x >>> 19)) ^\n	                                   (gamma1x >>> 10);\n\n	                    W[i] = gamma0 + W[i - 7] + gamma1 + W[i - 16];\n	                }\n\n	                var ch  = (e & f) ^ (~e & g);\n	                var maj = (a & b) ^ (a & c) ^ (b & c);\n\n	                var sigma0 = ((a << 30) | (a >>> 2)) ^ ((a << 19) | (a >>> 13)) ^ ((a << 10) | (a >>> 22));\n	                var sigma1 = ((e << 26) | (e >>> 6)) ^ ((e << 21) | (e >>> 11)) ^ ((e << 7)  | (e >>> 25));\n\n	                var t1 = h + sigma1 + ch + K[i] + W[i];\n	                var t2 = sigma0 + maj;\n\n	                h = g;\n	                g = f;\n	                f = e;\n	                e = (d + t1) | 0;\n	                d = c;\n	                c = b;\n	                b = a;\n	                a = (t1 + t2) | 0;\n	            }\n\n	            // Intermediate hash value\n	            H[0] = (H[0] + a) | 0;\n	            H[1] = (H[1] + b) | 0;\n	            H[2] = (H[2] + c) | 0;\n	            H[3] = (H[3] + d) | 0;\n	            H[4] = (H[4] + e) | 0;\n	            H[5] = (H[5] + f) | 0;\n	            H[6] = (H[6] + g) | 0;\n	            H[7] = (H[7] + h) | 0;\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x80 << (24 - nBitsLeft % 32);\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 14] = Math.floor(nBitsTotal / 0x100000000);\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 15] = nBitsTotal;\n	            data.sigBytes = dataWords.length * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Return final computed hash\n	            return this._hash;\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n	            clone._hash = this._hash.clone();\n\n	            return clone;\n	        }\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA256('message');\n	     *     var hash = CryptoJS.SHA256(wordArray);\n	     */\n	    C.SHA256 = Hasher._createHelper(SHA256);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA256(message, key);\n	     */\n	    C.HmacSHA256 = Hasher._createHmacHelper(SHA256);\n	}(Math));\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var C_algo = C.algo;\n	    var SHA256 = C_algo.SHA256;\n\n	    /**\n	     * SHA-224 hash algorithm.\n	     */\n	    var SHA224 = C_algo.SHA224 = SHA256.extend({\n	        _doReset: function () {\n	            this._hash = new WordArray.init([\n	                0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939,\n	                0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4\n	            ]);\n	        },\n\n	        _doFinalize: function () {\n	            var hash = SHA256._doFinalize.call(this);\n\n	            hash.sigBytes -= 4;\n\n	            return hash;\n	        }\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA224('message');\n	     *     var hash = CryptoJS.SHA224(wordArray);\n	     */\n	    C.SHA224 = SHA256._createHelper(SHA224);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA224(message, key);\n	     */\n	    C.HmacSHA224 = SHA256._createHmacHelper(SHA224);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Hasher = C_lib.Hasher;\n	    var C_x64 = C.x64;\n	    var X64Word = C_x64.Word;\n	    var X64WordArray = C_x64.WordArray;\n	    var C_algo = C.algo;\n\n	    function X64Word_create() {\n	        return X64Word.create.apply(X64Word, arguments);\n	    }\n\n	    // Constants\n	    var K = [\n	        X64Word_create(0x428a2f98, 0xd728ae22), X64Word_create(0x71374491, 0x23ef65cd),\n	        X64Word_create(0xb5c0fbcf, 0xec4d3b2f), X64Word_create(0xe9b5dba5, 0x8189dbbc),\n	        X64Word_create(0x3956c25b, 0xf348b538), X64Word_create(0x59f111f1, 0xb605d019),\n	        X64Word_create(0x923f82a4, 0xaf194f9b), X64Word_create(0xab1c5ed5, 0xda6d8118),\n	        X64Word_create(0xd807aa98, 0xa3030242), X64Word_create(0x12835b01, 0x45706fbe),\n	        X64Word_create(0x243185be, 0x4ee4b28c), X64Word_create(0x550c7dc3, 0xd5ffb4e2),\n	        X64Word_create(0x72be5d74, 0xf27b896f), X64Word_create(0x80deb1fe, 0x3b1696b1),\n	        X64Word_create(0x9bdc06a7, 0x25c71235), X64Word_create(0xc19bf174, 0xcf692694),\n	        X64Word_create(0xe49b69c1, 0x9ef14ad2), X64Word_create(0xefbe4786, 0x384f25e3),\n	        X64Word_create(0x0fc19dc6, 0x8b8cd5b5), X64Word_create(0x240ca1cc, 0x77ac9c65),\n	        X64Word_create(0x2de92c6f, 0x592b0275), X64Word_create(0x4a7484aa, 0x6ea6e483),\n	        X64Word_create(0x5cb0a9dc, 0xbd41fbd4), X64Word_create(0x76f988da, 0x831153b5),\n	        X64Word_create(0x983e5152, 0xee66dfab), X64Word_create(0xa831c66d, 0x2db43210),\n	        X64Word_create(0xb00327c8, 0x98fb213f), X64Word_create(0xbf597fc7, 0xbeef0ee4),\n	        X64Word_create(0xc6e00bf3, 0x3da88fc2), X64Word_create(0xd5a79147, 0x930aa725),\n	        X64Word_create(0x06ca6351, 0xe003826f), X64Word_create(0x14292967, 0x0a0e6e70),\n	        X64Word_create(0x27b70a85, 0x46d22ffc), X64Word_create(0x2e1b2138, 0x5c26c926),\n	        X64Word_create(0x4d2c6dfc, 0x5ac42aed), X64Word_create(0x53380d13, 0x9d95b3df),\n	        X64Word_create(0x650a7354, 0x8baf63de), X64Word_create(0x766a0abb, 0x3c77b2a8),\n	        X64Word_create(0x81c2c92e, 0x47edaee6), X64Word_create(0x92722c85, 0x1482353b),\n	        X64Word_create(0xa2bfe8a1, 0x4cf10364), X64Word_create(0xa81a664b, 0xbc423001),\n	        X64Word_create(0xc24b8b70, 0xd0f89791), X64Word_create(0xc76c51a3, 0x0654be30),\n	        X64Word_create(0xd192e819, 0xd6ef5218), X64Word_create(0xd6990624, 0x5565a910),\n	        X64Word_create(0xf40e3585, 0x5771202a), X64Word_create(0x106aa070, 0x32bbd1b8),\n	        X64Word_create(0x19a4c116, 0xb8d2d0c8), X64Word_create(0x1e376c08, 0x5141ab53),\n	        X64Word_create(0x2748774c, 0xdf8eeb99), X64Word_create(0x34b0bcb5, 0xe19b48a8),\n	        X64Word_create(0x391c0cb3, 0xc5c95a63), X64Word_create(0x4ed8aa4a, 0xe3418acb),\n	        X64Word_create(0x5b9cca4f, 0x7763e373), X64Word_create(0x682e6ff3, 0xd6b2b8a3),\n	        X64Word_create(0x748f82ee, 0x5defb2fc), X64Word_create(0x78a5636f, 0x43172f60),\n	        X64Word_create(0x84c87814, 0xa1f0ab72), X64Word_create(0x8cc70208, 0x1a6439ec),\n	        X64Word_create(0x90befffa, 0x23631e28), X64Word_create(0xa4506ceb, 0xde82bde9),\n	        X64Word_create(0xbef9a3f7, 0xb2c67915), X64Word_create(0xc67178f2, 0xe372532b),\n	        X64Word_create(0xca273ece, 0xea26619c), X64Word_create(0xd186b8c7, 0x21c0c207),\n	        X64Word_create(0xeada7dd6, 0xcde0eb1e), X64Word_create(0xf57d4f7f, 0xee6ed178),\n	        X64Word_create(0x06f067aa, 0x72176fba), X64Word_create(0x0a637dc5, 0xa2c898a6),\n	        X64Word_create(0x113f9804, 0xbef90dae), X64Word_create(0x1b710b35, 0x131c471b),\n	        X64Word_create(0x28db77f5, 0x23047d84), X64Word_create(0x32caab7b, 0x40c72493),\n	        X64Word_create(0x3c9ebe0a, 0x15c9bebc), X64Word_create(0x431d67c4, 0x9c100d4c),\n	        X64Word_create(0x4cc5d4be, 0xcb3e42b6), X64Word_create(0x597f299c, 0xfc657e2a),\n	        X64Word_create(0x5fcb6fab, 0x3ad6faec), X64Word_create(0x6c44198c, 0x4a475817)\n	    ];\n\n	    // Reusable objects\n	    var W = [];\n	    (function () {\n	        for (var i = 0; i < 80; i++) {\n	            W[i] = X64Word_create();\n	        }\n	    }());\n\n	    /**\n	     * SHA-512 hash algorithm.\n	     */\n	    var SHA512 = C_algo.SHA512 = Hasher.extend({\n	        _doReset: function () {\n	            this._hash = new X64WordArray.init([\n	                new X64Word.init(0x6a09e667, 0xf3bcc908), new X64Word.init(0xbb67ae85, 0x84caa73b),\n	                new X64Word.init(0x3c6ef372, 0xfe94f82b), new X64Word.init(0xa54ff53a, 0x5f1d36f1),\n	                new X64Word.init(0x510e527f, 0xade682d1), new X64Word.init(0x9b05688c, 0x2b3e6c1f),\n	                new X64Word.init(0x1f83d9ab, 0xfb41bd6b), new X64Word.init(0x5be0cd19, 0x137e2179)\n	            ]);\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcuts\n	            var H = this._hash.words;\n\n	            var H0 = H[0];\n	            var H1 = H[1];\n	            var H2 = H[2];\n	            var H3 = H[3];\n	            var H4 = H[4];\n	            var H5 = H[5];\n	            var H6 = H[6];\n	            var H7 = H[7];\n\n	            var H0h = H0.high;\n	            var H0l = H0.low;\n	            var H1h = H1.high;\n	            var H1l = H1.low;\n	            var H2h = H2.high;\n	            var H2l = H2.low;\n	            var H3h = H3.high;\n	            var H3l = H3.low;\n	            var H4h = H4.high;\n	            var H4l = H4.low;\n	            var H5h = H5.high;\n	            var H5l = H5.low;\n	            var H6h = H6.high;\n	            var H6l = H6.low;\n	            var H7h = H7.high;\n	            var H7l = H7.low;\n\n	            // Working variables\n	            var ah = H0h;\n	            var al = H0l;\n	            var bh = H1h;\n	            var bl = H1l;\n	            var ch = H2h;\n	            var cl = H2l;\n	            var dh = H3h;\n	            var dl = H3l;\n	            var eh = H4h;\n	            var el = H4l;\n	            var fh = H5h;\n	            var fl = H5l;\n	            var gh = H6h;\n	            var gl = H6l;\n	            var hh = H7h;\n	            var hl = H7l;\n\n	            // Rounds\n	            for (var i = 0; i < 80; i++) {\n	                var Wil;\n	                var Wih;\n\n	                // Shortcut\n	                var Wi = W[i];\n\n	                // Extend message\n	                if (i < 16) {\n	                    Wih = Wi.high = M[offset + i * 2]     | 0;\n	                    Wil = Wi.low  = M[offset + i * 2 + 1] | 0;\n	                } else {\n	                    // Gamma0\n	                    var gamma0x  = W[i - 15];\n	                    var gamma0xh = gamma0x.high;\n	                    var gamma0xl = gamma0x.low;\n	                    var gamma0h  = ((gamma0xh >>> 1) | (gamma0xl << 31)) ^ ((gamma0xh >>> 8) | (gamma0xl << 24)) ^ (gamma0xh >>> 7);\n	                    var gamma0l  = ((gamma0xl >>> 1) | (gamma0xh << 31)) ^ ((gamma0xl >>> 8) | (gamma0xh << 24)) ^ ((gamma0xl >>> 7) | (gamma0xh << 25));\n\n	                    // Gamma1\n	                    var gamma1x  = W[i - 2];\n	                    var gamma1xh = gamma1x.high;\n	                    var gamma1xl = gamma1x.low;\n	                    var gamma1h  = ((gamma1xh >>> 19) | (gamma1xl << 13)) ^ ((gamma1xh << 3) | (gamma1xl >>> 29)) ^ (gamma1xh >>> 6);\n	                    var gamma1l  = ((gamma1xl >>> 19) | (gamma1xh << 13)) ^ ((gamma1xl << 3) | (gamma1xh >>> 29)) ^ ((gamma1xl >>> 6) | (gamma1xh << 26));\n\n	                    // W[i] = gamma0 + W[i - 7] + gamma1 + W[i - 16]\n	                    var Wi7  = W[i - 7];\n	                    var Wi7h = Wi7.high;\n	                    var Wi7l = Wi7.low;\n\n	                    var Wi16  = W[i - 16];\n	                    var Wi16h = Wi16.high;\n	                    var Wi16l = Wi16.low;\n\n	                    Wil = gamma0l + Wi7l;\n	                    Wih = gamma0h + Wi7h + ((Wil >>> 0) < (gamma0l >>> 0) ? 1 : 0);\n	                    Wil = Wil + gamma1l;\n	                    Wih = Wih + gamma1h + ((Wil >>> 0) < (gamma1l >>> 0) ? 1 : 0);\n	                    Wil = Wil + Wi16l;\n	                    Wih = Wih + Wi16h + ((Wil >>> 0) < (Wi16l >>> 0) ? 1 : 0);\n\n	                    Wi.high = Wih;\n	                    Wi.low  = Wil;\n	                }\n\n	                var chh  = (eh & fh) ^ (~eh & gh);\n	                var chl  = (el & fl) ^ (~el & gl);\n	                var majh = (ah & bh) ^ (ah & ch) ^ (bh & ch);\n	                var majl = (al & bl) ^ (al & cl) ^ (bl & cl);\n\n	                var sigma0h = ((ah >>> 28) | (al << 4))  ^ ((ah << 30)  | (al >>> 2)) ^ ((ah << 25) | (al >>> 7));\n	                var sigma0l = ((al >>> 28) | (ah << 4))  ^ ((al << 30)  | (ah >>> 2)) ^ ((al << 25) | (ah >>> 7));\n	                var sigma1h = ((eh >>> 14) | (el << 18)) ^ ((eh >>> 18) | (el << 14)) ^ ((eh << 23) | (el >>> 9));\n	                var sigma1l = ((el >>> 14) | (eh << 18)) ^ ((el >>> 18) | (eh << 14)) ^ ((el << 23) | (eh >>> 9));\n\n	                // t1 = h + sigma1 + ch + K[i] + W[i]\n	                var Ki  = K[i];\n	                var Kih = Ki.high;\n	                var Kil = Ki.low;\n\n	                var t1l = hl + sigma1l;\n	                var t1h = hh + sigma1h + ((t1l >>> 0) < (hl >>> 0) ? 1 : 0);\n	                var t1l = t1l + chl;\n	                var t1h = t1h + chh + ((t1l >>> 0) < (chl >>> 0) ? 1 : 0);\n	                var t1l = t1l + Kil;\n	                var t1h = t1h + Kih + ((t1l >>> 0) < (Kil >>> 0) ? 1 : 0);\n	                var t1l = t1l + Wil;\n	                var t1h = t1h + Wih + ((t1l >>> 0) < (Wil >>> 0) ? 1 : 0);\n\n	                // t2 = sigma0 + maj\n	                var t2l = sigma0l + majl;\n	                var t2h = sigma0h + majh + ((t2l >>> 0) < (sigma0l >>> 0) ? 1 : 0);\n\n	                // Update working variables\n	                hh = gh;\n	                hl = gl;\n	                gh = fh;\n	                gl = fl;\n	                fh = eh;\n	                fl = el;\n	                el = (dl + t1l) | 0;\n	                eh = (dh + t1h + ((el >>> 0) < (dl >>> 0) ? 1 : 0)) | 0;\n	                dh = ch;\n	                dl = cl;\n	                ch = bh;\n	                cl = bl;\n	                bh = ah;\n	                bl = al;\n	                al = (t1l + t2l) | 0;\n	                ah = (t1h + t2h + ((al >>> 0) < (t1l >>> 0) ? 1 : 0)) | 0;\n	            }\n\n	            // Intermediate hash value\n	            H0l = H0.low  = (H0l + al);\n	            H0.high = (H0h + ah + ((H0l >>> 0) < (al >>> 0) ? 1 : 0));\n	            H1l = H1.low  = (H1l + bl);\n	            H1.high = (H1h + bh + ((H1l >>> 0) < (bl >>> 0) ? 1 : 0));\n	            H2l = H2.low  = (H2l + cl);\n	            H2.high = (H2h + ch + ((H2l >>> 0) < (cl >>> 0) ? 1 : 0));\n	            H3l = H3.low  = (H3l + dl);\n	            H3.high = (H3h + dh + ((H3l >>> 0) < (dl >>> 0) ? 1 : 0));\n	            H4l = H4.low  = (H4l + el);\n	            H4.high = (H4h + eh + ((H4l >>> 0) < (el >>> 0) ? 1 : 0));\n	            H5l = H5.low  = (H5l + fl);\n	            H5.high = (H5h + fh + ((H5l >>> 0) < (fl >>> 0) ? 1 : 0));\n	            H6l = H6.low  = (H6l + gl);\n	            H6.high = (H6h + gh + ((H6l >>> 0) < (gl >>> 0) ? 1 : 0));\n	            H7l = H7.low  = (H7l + hl);\n	            H7.high = (H7h + hh + ((H7l >>> 0) < (hl >>> 0) ? 1 : 0));\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x80 << (24 - nBitsLeft % 32);\n	            dataWords[(((nBitsLeft + 128) >>> 10) << 5) + 30] = Math.floor(nBitsTotal / 0x100000000);\n	            dataWords[(((nBitsLeft + 128) >>> 10) << 5) + 31] = nBitsTotal;\n	            data.sigBytes = dataWords.length * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Convert hash to 32-bit word array before returning\n	            var hash = this._hash.toX32();\n\n	            // Return final computed hash\n	            return hash;\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n	            clone._hash = this._hash.clone();\n\n	            return clone;\n	        },\n\n	        blockSize: 1024/32\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA512('message');\n	     *     var hash = CryptoJS.SHA512(wordArray);\n	     */\n	    C.SHA512 = Hasher._createHelper(SHA512);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA512(message, key);\n	     */\n	    C.HmacSHA512 = Hasher._createHmacHelper(SHA512);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_x64 = C.x64;\n	    var X64Word = C_x64.Word;\n	    var X64WordArray = C_x64.WordArray;\n	    var C_algo = C.algo;\n	    var SHA512 = C_algo.SHA512;\n\n	    /**\n	     * SHA-384 hash algorithm.\n	     */\n	    var SHA384 = C_algo.SHA384 = SHA512.extend({\n	        _doReset: function () {\n	            this._hash = new X64WordArray.init([\n	                new X64Word.init(0xcbbb9d5d, 0xc1059ed8), new X64Word.init(0x629a292a, 0x367cd507),\n	                new X64Word.init(0x9159015a, 0x3070dd17), new X64Word.init(0x152fecd8, 0xf70e5939),\n	                new X64Word.init(0x67332667, 0xffc00b31), new X64Word.init(0x8eb44a87, 0x68581511),\n	                new X64Word.init(0xdb0c2e0d, 0x64f98fa7), new X64Word.init(0x47b5481d, 0xbefa4fa4)\n	            ]);\n	        },\n\n	        _doFinalize: function () {\n	            var hash = SHA512._doFinalize.call(this);\n\n	            hash.sigBytes -= 16;\n\n	            return hash;\n	        }\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA384('message');\n	     *     var hash = CryptoJS.SHA384(wordArray);\n	     */\n	    C.SHA384 = SHA512._createHelper(SHA384);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA384(message, key);\n	     */\n	    C.HmacSHA384 = SHA512._createHmacHelper(SHA384);\n	}());\n\n\n	(function (Math) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var Hasher = C_lib.Hasher;\n	    var C_x64 = C.x64;\n	    var X64Word = C_x64.Word;\n	    var C_algo = C.algo;\n\n	    // Constants tables\n	    var RHO_OFFSETS = [];\n	    var PI_INDEXES  = [];\n	    var ROUND_CONSTANTS = [];\n\n	    // Compute Constants\n	    (function () {\n	        // Compute rho offset constants\n	        var x = 1, y = 0;\n	        for (var t = 0; t < 24; t++) {\n	            RHO_OFFSETS[x + 5 * y] = ((t + 1) * (t + 2) / 2) % 64;\n\n	            var newX = y % 5;\n	            var newY = (2 * x + 3 * y) % 5;\n	            x = newX;\n	            y = newY;\n	        }\n\n	        // Compute pi index constants\n	        for (var x = 0; x < 5; x++) {\n	            for (var y = 0; y < 5; y++) {\n	                PI_INDEXES[x + 5 * y] = y + ((2 * x + 3 * y) % 5) * 5;\n	            }\n	        }\n\n	        // Compute round constants\n	        var LFSR = 0x01;\n	        for (var i = 0; i < 24; i++) {\n	            var roundConstantMsw = 0;\n	            var roundConstantLsw = 0;\n\n	            for (var j = 0; j < 7; j++) {\n	                if (LFSR & 0x01) {\n	                    var bitPosition = (1 << j) - 1;\n	                    if (bitPosition < 32) {\n	                        roundConstantLsw ^= 1 << bitPosition;\n	                    } else /* if (bitPosition >= 32) */ {\n	                        roundConstantMsw ^= 1 << (bitPosition - 32);\n	                    }\n	                }\n\n	                // Compute next LFSR\n	                if (LFSR & 0x80) {\n	                    // Primitive polynomial over GF(2): x^8 + x^6 + x^5 + x^4 + 1\n	                    LFSR = (LFSR << 1) ^ 0x71;\n	                } else {\n	                    LFSR <<= 1;\n	                }\n	            }\n\n	            ROUND_CONSTANTS[i] = X64Word.create(roundConstantMsw, roundConstantLsw);\n	        }\n	    }());\n\n	    // Reusable objects for temporary values\n	    var T = [];\n	    (function () {\n	        for (var i = 0; i < 25; i++) {\n	            T[i] = X64Word.create();\n	        }\n	    }());\n\n	    /**\n	     * SHA-3 hash algorithm.\n	     */\n	    var SHA3 = C_algo.SHA3 = Hasher.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {number} outputLength\n	         *   The desired number of bits in the output hash.\n	         *   Only values permitted are: 224, 256, 384, 512.\n	         *   Default: 512\n	         */\n	        cfg: Hasher.cfg.extend({\n	            outputLength: 512\n	        }),\n\n	        _doReset: function () {\n	            var state = this._state = []\n	            for (var i = 0; i < 25; i++) {\n	                state[i] = new X64Word.init();\n	            }\n\n	            this.blockSize = (1600 - 2 * this.cfg.outputLength) / 32;\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcuts\n	            var state = this._state;\n	            var nBlockSizeLanes = this.blockSize / 2;\n\n	            // Absorb\n	            for (var i = 0; i < nBlockSizeLanes; i++) {\n	                // Shortcuts\n	                var M2i  = M[offset + 2 * i];\n	                var M2i1 = M[offset + 2 * i + 1];\n\n	                // Swap endian\n	                M2i = (\n	                    (((M2i << 8)  | (M2i >>> 24)) & 0x00ff00ff) |\n	                    (((M2i << 24) | (M2i >>> 8))  & 0xff00ff00)\n	                );\n	                M2i1 = (\n	                    (((M2i1 << 8)  | (M2i1 >>> 24)) & 0x00ff00ff) |\n	                    (((M2i1 << 24) | (M2i1 >>> 8))  & 0xff00ff00)\n	                );\n\n	                // Absorb message into state\n	                var lane = state[i];\n	                lane.high ^= M2i1;\n	                lane.low  ^= M2i;\n	            }\n\n	            // Rounds\n	            for (var round = 0; round < 24; round++) {\n	                // Theta\n	                for (var x = 0; x < 5; x++) {\n	                    // Mix column lanes\n	                    var tMsw = 0, tLsw = 0;\n	                    for (var y = 0; y < 5; y++) {\n	                        var lane = state[x + 5 * y];\n	                        tMsw ^= lane.high;\n	                        tLsw ^= lane.low;\n	                    }\n\n	                    // Temporary values\n	                    var Tx = T[x];\n	                    Tx.high = tMsw;\n	                    Tx.low  = tLsw;\n	                }\n	                for (var x = 0; x < 5; x++) {\n	                    // Shortcuts\n	                    var Tx4 = T[(x + 4) % 5];\n	                    var Tx1 = T[(x + 1) % 5];\n	                    var Tx1Msw = Tx1.high;\n	                    var Tx1Lsw = Tx1.low;\n\n	                    // Mix surrounding columns\n	                    var tMsw = Tx4.high ^ ((Tx1Msw << 1) | (Tx1Lsw >>> 31));\n	                    var tLsw = Tx4.low  ^ ((Tx1Lsw << 1) | (Tx1Msw >>> 31));\n	                    for (var y = 0; y < 5; y++) {\n	                        var lane = state[x + 5 * y];\n	                        lane.high ^= tMsw;\n	                        lane.low  ^= tLsw;\n	                    }\n	                }\n\n	                // Rho Pi\n	                for (var laneIndex = 1; laneIndex < 25; laneIndex++) {\n	                    var tMsw;\n	                    var tLsw;\n\n	                    // Shortcuts\n	                    var lane = state[laneIndex];\n	                    var laneMsw = lane.high;\n	                    var laneLsw = lane.low;\n	                    var rhoOffset = RHO_OFFSETS[laneIndex];\n\n	                    // Rotate lanes\n	                    if (rhoOffset < 32) {\n	                        tMsw = (laneMsw << rhoOffset) | (laneLsw >>> (32 - rhoOffset));\n	                        tLsw = (laneLsw << rhoOffset) | (laneMsw >>> (32 - rhoOffset));\n	                    } else /* if (rhoOffset >= 32) */ {\n	                        tMsw = (laneLsw << (rhoOffset - 32)) | (laneMsw >>> (64 - rhoOffset));\n	                        tLsw = (laneMsw << (rhoOffset - 32)) | (laneLsw >>> (64 - rhoOffset));\n	                    }\n\n	                    // Transpose lanes\n	                    var TPiLane = T[PI_INDEXES[laneIndex]];\n	                    TPiLane.high = tMsw;\n	                    TPiLane.low  = tLsw;\n	                }\n\n	                // Rho pi at x = y = 0\n	                var T0 = T[0];\n	                var state0 = state[0];\n	                T0.high = state0.high;\n	                T0.low  = state0.low;\n\n	                // Chi\n	                for (var x = 0; x < 5; x++) {\n	                    for (var y = 0; y < 5; y++) {\n	                        // Shortcuts\n	                        var laneIndex = x + 5 * y;\n	                        var lane = state[laneIndex];\n	                        var TLane = T[laneIndex];\n	                        var Tx1Lane = T[((x + 1) % 5) + 5 * y];\n	                        var Tx2Lane = T[((x + 2) % 5) + 5 * y];\n\n	                        // Mix rows\n	                        lane.high = TLane.high ^ (~Tx1Lane.high & Tx2Lane.high);\n	                        lane.low  = TLane.low  ^ (~Tx1Lane.low  & Tx2Lane.low);\n	                    }\n	                }\n\n	                // Iota\n	                var lane = state[0];\n	                var roundConstant = ROUND_CONSTANTS[round];\n	                lane.high ^= roundConstant.high;\n	                lane.low  ^= roundConstant.low;\n	            }\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n	            var blockSizeBits = this.blockSize * 32;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x1 << (24 - nBitsLeft % 32);\n	            dataWords[((Math.ceil((nBitsLeft + 1) / blockSizeBits) * blockSizeBits) >>> 5) - 1] |= 0x80;\n	            data.sigBytes = dataWords.length * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Shortcuts\n	            var state = this._state;\n	            var outputLengthBytes = this.cfg.outputLength / 8;\n	            var outputLengthLanes = outputLengthBytes / 8;\n\n	            // Squeeze\n	            var hashWords = [];\n	            for (var i = 0; i < outputLengthLanes; i++) {\n	                // Shortcuts\n	                var lane = state[i];\n	                var laneMsw = lane.high;\n	                var laneLsw = lane.low;\n\n	                // Swap endian\n	                laneMsw = (\n	                    (((laneMsw << 8)  | (laneMsw >>> 24)) & 0x00ff00ff) |\n	                    (((laneMsw << 24) | (laneMsw >>> 8))  & 0xff00ff00)\n	                );\n	                laneLsw = (\n	                    (((laneLsw << 8)  | (laneLsw >>> 24)) & 0x00ff00ff) |\n	                    (((laneLsw << 24) | (laneLsw >>> 8))  & 0xff00ff00)\n	                );\n\n	                // Squeeze state to retrieve hash\n	                hashWords.push(laneLsw);\n	                hashWords.push(laneMsw);\n	            }\n\n	            // Return final computed hash\n	            return new WordArray.init(hashWords, outputLengthBytes);\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n\n	            var state = clone._state = this._state.slice(0);\n	            for (var i = 0; i < 25; i++) {\n	                state[i] = state[i].clone();\n	            }\n\n	            return clone;\n	        }\n	    });\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.SHA3('message');\n	     *     var hash = CryptoJS.SHA3(wordArray);\n	     */\n	    C.SHA3 = Hasher._createHelper(SHA3);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacSHA3(message, key);\n	     */\n	    C.HmacSHA3 = Hasher._createHmacHelper(SHA3);\n	}(Math));\n\n\n	/** @preserve\n	(c) 2012 by C\xE9dric Mesnil. All rights reserved.\n\n	Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:\n\n	    - Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.\n	    - Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.\n\n	THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.\n	*/\n\n	(function (Math) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var Hasher = C_lib.Hasher;\n	    var C_algo = C.algo;\n\n	    // Constants table\n	    var _zl = WordArray.create([\n	        0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,\n	        7,  4, 13,  1, 10,  6, 15,  3, 12,  0,  9,  5,  2, 14, 11,  8,\n	        3, 10, 14,  4,  9, 15,  8,  1,  2,  7,  0,  6, 13, 11,  5, 12,\n	        1,  9, 11, 10,  0,  8, 12,  4, 13,  3,  7, 15, 14,  5,  6,  2,\n	        4,  0,  5,  9,  7, 12,  2, 10, 14,  1,  3,  8, 11,  6, 15, 13]);\n	    var _zr = WordArray.create([\n	        5, 14,  7,  0,  9,  2, 11,  4, 13,  6, 15,  8,  1, 10,  3, 12,\n	        6, 11,  3,  7,  0, 13,  5, 10, 14, 15,  8, 12,  4,  9,  1,  2,\n	        15,  5,  1,  3,  7, 14,  6,  9, 11,  8, 12,  2, 10,  0,  4, 13,\n	        8,  6,  4,  1,  3, 11, 15,  0,  5, 12,  2, 13,  9,  7, 10, 14,\n	        12, 15, 10,  4,  1,  5,  8,  7,  6,  2, 13, 14,  0,  3,  9, 11]);\n	    var _sl = WordArray.create([\n	         11, 14, 15, 12,  5,  8,  7,  9, 11, 13, 14, 15,  6,  7,  9,  8,\n	        7, 6,   8, 13, 11,  9,  7, 15,  7, 12, 15,  9, 11,  7, 13, 12,\n	        11, 13,  6,  7, 14,  9, 13, 15, 14,  8, 13,  6,  5, 12,  7,  5,\n	          11, 12, 14, 15, 14, 15,  9,  8,  9, 14,  5,  6,  8,  6,  5, 12,\n	        9, 15,  5, 11,  6,  8, 13, 12,  5, 12, 13, 14, 11,  8,  5,  6 ]);\n	    var _sr = WordArray.create([\n	        8,  9,  9, 11, 13, 15, 15,  5,  7,  7,  8, 11, 14, 14, 12,  6,\n	        9, 13, 15,  7, 12,  8,  9, 11,  7,  7, 12,  7,  6, 15, 13, 11,\n	        9,  7, 15, 11,  8,  6,  6, 14, 12, 13,  5, 14, 13, 13,  7,  5,\n	        15,  5,  8, 11, 14, 14,  6, 14,  6,  9, 12,  9, 12,  5, 15,  8,\n	        8,  5, 12,  9, 12,  5, 14,  6,  8, 13,  6,  5, 15, 13, 11, 11 ]);\n\n	    var _hl =  WordArray.create([ 0x00000000, 0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xA953FD4E]);\n	    var _hr =  WordArray.create([ 0x50A28BE6, 0x5C4DD124, 0x6D703EF3, 0x7A6D76E9, 0x00000000]);\n\n	    /**\n	     * RIPEMD160 hash algorithm.\n	     */\n	    var RIPEMD160 = C_algo.RIPEMD160 = Hasher.extend({\n	        _doReset: function () {\n	            this._hash  = WordArray.create([0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0]);\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n\n	            // Swap endian\n	            for (var i = 0; i < 16; i++) {\n	                // Shortcuts\n	                var offset_i = offset + i;\n	                var M_offset_i = M[offset_i];\n\n	                // Swap\n	                M[offset_i] = (\n	                    (((M_offset_i << 8)  | (M_offset_i >>> 24)) & 0x00ff00ff) |\n	                    (((M_offset_i << 24) | (M_offset_i >>> 8))  & 0xff00ff00)\n	                );\n	            }\n	            // Shortcut\n	            var H  = this._hash.words;\n	            var hl = _hl.words;\n	            var hr = _hr.words;\n	            var zl = _zl.words;\n	            var zr = _zr.words;\n	            var sl = _sl.words;\n	            var sr = _sr.words;\n\n	            // Working variables\n	            var al, bl, cl, dl, el;\n	            var ar, br, cr, dr, er;\n\n	            ar = al = H[0];\n	            br = bl = H[1];\n	            cr = cl = H[2];\n	            dr = dl = H[3];\n	            er = el = H[4];\n	            // Computation\n	            var t;\n	            for (var i = 0; i < 80; i += 1) {\n	                t = (al +  M[offset+zl[i]])|0;\n	                if (i<16){\n		            t +=  f1(bl,cl,dl) + hl[0];\n	                } else if (i<32) {\n		            t +=  f2(bl,cl,dl) + hl[1];\n	                } else if (i<48) {\n		            t +=  f3(bl,cl,dl) + hl[2];\n	                } else if (i<64) {\n		            t +=  f4(bl,cl,dl) + hl[3];\n	                } else {// if (i<80) {\n		            t +=  f5(bl,cl,dl) + hl[4];\n	                }\n	                t = t|0;\n	                t =  rotl(t,sl[i]);\n	                t = (t+el)|0;\n	                al = el;\n	                el = dl;\n	                dl = rotl(cl, 10);\n	                cl = bl;\n	                bl = t;\n\n	                t = (ar + M[offset+zr[i]])|0;\n	                if (i<16){\n		            t +=  f5(br,cr,dr) + hr[0];\n	                } else if (i<32) {\n		            t +=  f4(br,cr,dr) + hr[1];\n	                } else if (i<48) {\n		            t +=  f3(br,cr,dr) + hr[2];\n	                } else if (i<64) {\n		            t +=  f2(br,cr,dr) + hr[3];\n	                } else {// if (i<80) {\n		            t +=  f1(br,cr,dr) + hr[4];\n	                }\n	                t = t|0;\n	                t =  rotl(t,sr[i]) ;\n	                t = (t+er)|0;\n	                ar = er;\n	                er = dr;\n	                dr = rotl(cr, 10);\n	                cr = br;\n	                br = t;\n	            }\n	            // Intermediate hash value\n	            t    = (H[1] + cl + dr)|0;\n	            H[1] = (H[2] + dl + er)|0;\n	            H[2] = (H[3] + el + ar)|0;\n	            H[3] = (H[4] + al + br)|0;\n	            H[4] = (H[0] + bl + cr)|0;\n	            H[0] =  t;\n	        },\n\n	        _doFinalize: function () {\n	            // Shortcuts\n	            var data = this._data;\n	            var dataWords = data.words;\n\n	            var nBitsTotal = this._nDataBytes * 8;\n	            var nBitsLeft = data.sigBytes * 8;\n\n	            // Add padding\n	            dataWords[nBitsLeft >>> 5] |= 0x80 << (24 - nBitsLeft % 32);\n	            dataWords[(((nBitsLeft + 64) >>> 9) << 4) + 14] = (\n	                (((nBitsTotal << 8)  | (nBitsTotal >>> 24)) & 0x00ff00ff) |\n	                (((nBitsTotal << 24) | (nBitsTotal >>> 8))  & 0xff00ff00)\n	            );\n	            data.sigBytes = (dataWords.length + 1) * 4;\n\n	            // Hash final blocks\n	            this._process();\n\n	            // Shortcuts\n	            var hash = this._hash;\n	            var H = hash.words;\n\n	            // Swap endian\n	            for (var i = 0; i < 5; i++) {\n	                // Shortcut\n	                var H_i = H[i];\n\n	                // Swap\n	                H[i] = (((H_i << 8)  | (H_i >>> 24)) & 0x00ff00ff) |\n	                       (((H_i << 24) | (H_i >>> 8))  & 0xff00ff00);\n	            }\n\n	            // Return final computed hash\n	            return hash;\n	        },\n\n	        clone: function () {\n	            var clone = Hasher.clone.call(this);\n	            clone._hash = this._hash.clone();\n\n	            return clone;\n	        }\n	    });\n\n\n	    function f1(x, y, z) {\n	        return ((x) ^ (y) ^ (z));\n\n	    }\n\n	    function f2(x, y, z) {\n	        return (((x)&(y)) | ((~x)&(z)));\n	    }\n\n	    function f3(x, y, z) {\n	        return (((x) | (~(y))) ^ (z));\n	    }\n\n	    function f4(x, y, z) {\n	        return (((x) & (z)) | ((y)&(~(z))));\n	    }\n\n	    function f5(x, y, z) {\n	        return ((x) ^ ((y) |(~(z))));\n\n	    }\n\n	    function rotl(x,n) {\n	        return (x<<n) | (x>>>(32-n));\n	    }\n\n\n	    /**\n	     * Shortcut function to the hasher's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     *\n	     * @return {WordArray} The hash.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hash = CryptoJS.RIPEMD160('message');\n	     *     var hash = CryptoJS.RIPEMD160(wordArray);\n	     */\n	    C.RIPEMD160 = Hasher._createHelper(RIPEMD160);\n\n	    /**\n	     * Shortcut function to the HMAC's object interface.\n	     *\n	     * @param {WordArray|string} message The message to hash.\n	     * @param {WordArray|string} key The secret key.\n	     *\n	     * @return {WordArray} The HMAC.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var hmac = CryptoJS.HmacRIPEMD160(message, key);\n	     */\n	    C.HmacRIPEMD160 = Hasher._createHmacHelper(RIPEMD160);\n	}(Math));\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Base = C_lib.Base;\n	    var C_enc = C.enc;\n	    var Utf8 = C_enc.Utf8;\n	    var C_algo = C.algo;\n\n	    /**\n	     * HMAC algorithm.\n	     */\n	    var HMAC = C_algo.HMAC = Base.extend({\n	        /**\n	         * Initializes a newly created HMAC.\n	         *\n	         * @param {Hasher} hasher The hash algorithm to use.\n	         * @param {WordArray|string} key The secret key.\n	         *\n	         * @example\n	         *\n	         *     var hmacHasher = CryptoJS.algo.HMAC.create(CryptoJS.algo.SHA256, key);\n	         */\n	        init: function (hasher, key) {\n	            // Init hasher\n	            hasher = this._hasher = new hasher.init();\n\n	            // Convert string to WordArray, else assume WordArray already\n	            if (typeof key == 'string') {\n	                key = Utf8.parse(key);\n	            }\n\n	            // Shortcuts\n	            var hasherBlockSize = hasher.blockSize;\n	            var hasherBlockSizeBytes = hasherBlockSize * 4;\n\n	            // Allow arbitrary length keys\n	            if (key.sigBytes > hasherBlockSizeBytes) {\n	                key = hasher.finalize(key);\n	            }\n\n	            // Clamp excess bits\n	            key.clamp();\n\n	            // Clone key for inner and outer pads\n	            var oKey = this._oKey = key.clone();\n	            var iKey = this._iKey = key.clone();\n\n	            // Shortcuts\n	            var oKeyWords = oKey.words;\n	            var iKeyWords = iKey.words;\n\n	            // XOR keys with pad constants\n	            for (var i = 0; i < hasherBlockSize; i++) {\n	                oKeyWords[i] ^= 0x5c5c5c5c;\n	                iKeyWords[i] ^= 0x36363636;\n	            }\n	            oKey.sigBytes = iKey.sigBytes = hasherBlockSizeBytes;\n\n	            // Set initial values\n	            this.reset();\n	        },\n\n	        /**\n	         * Resets this HMAC to its initial state.\n	         *\n	         * @example\n	         *\n	         *     hmacHasher.reset();\n	         */\n	        reset: function () {\n	            // Shortcut\n	            var hasher = this._hasher;\n\n	            // Reset\n	            hasher.reset();\n	            hasher.update(this._iKey);\n	        },\n\n	        /**\n	         * Updates this HMAC with a message.\n	         *\n	         * @param {WordArray|string} messageUpdate The message to append.\n	         *\n	         * @return {HMAC} This HMAC instance.\n	         *\n	         * @example\n	         *\n	         *     hmacHasher.update('message');\n	         *     hmacHasher.update(wordArray);\n	         */\n	        update: function (messageUpdate) {\n	            this._hasher.update(messageUpdate);\n\n	            // Chainable\n	            return this;\n	        },\n\n	        /**\n	         * Finalizes the HMAC computation.\n	         * Note that the finalize operation is effectively a destructive, read-once operation.\n	         *\n	         * @param {WordArray|string} messageUpdate (Optional) A final message update.\n	         *\n	         * @return {WordArray} The HMAC.\n	         *\n	         * @example\n	         *\n	         *     var hmac = hmacHasher.finalize();\n	         *     var hmac = hmacHasher.finalize('message');\n	         *     var hmac = hmacHasher.finalize(wordArray);\n	         */\n	        finalize: function (messageUpdate) {\n	            // Shortcut\n	            var hasher = this._hasher;\n\n	            // Compute HMAC\n	            var innerHash = hasher.finalize(messageUpdate);\n	            hasher.reset();\n	            var hmac = hasher.finalize(this._oKey.clone().concat(innerHash));\n\n	            return hmac;\n	        }\n	    });\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Base = C_lib.Base;\n	    var WordArray = C_lib.WordArray;\n	    var C_algo = C.algo;\n	    var SHA256 = C_algo.SHA256;\n	    var HMAC = C_algo.HMAC;\n\n	    /**\n	     * Password-Based Key Derivation Function 2 algorithm.\n	     */\n	    var PBKDF2 = C_algo.PBKDF2 = Base.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {number} keySize The key size in words to generate. Default: 4 (128 bits)\n	         * @property {Hasher} hasher The hasher to use. Default: SHA256\n	         * @property {number} iterations The number of iterations to perform. Default: 250000\n	         */\n	        cfg: Base.extend({\n	            keySize: 128/32,\n	            hasher: SHA256,\n	            iterations: 250000\n	        }),\n\n	        /**\n	         * Initializes a newly created key derivation function.\n	         *\n	         * @param {Object} cfg (Optional) The configuration options to use for the derivation.\n	         *\n	         * @example\n	         *\n	         *     var kdf = CryptoJS.algo.PBKDF2.create();\n	         *     var kdf = CryptoJS.algo.PBKDF2.create({ keySize: 8 });\n	         *     var kdf = CryptoJS.algo.PBKDF2.create({ keySize: 8, iterations: 1000 });\n	         */\n	        init: function (cfg) {\n	            this.cfg = this.cfg.extend(cfg);\n	        },\n\n	        /**\n	         * Computes the Password-Based Key Derivation Function 2.\n	         *\n	         * @param {WordArray|string} password The password.\n	         * @param {WordArray|string} salt A salt.\n	         *\n	         * @return {WordArray} The derived key.\n	         *\n	         * @example\n	         *\n	         *     var key = kdf.compute(password, salt);\n	         */\n	        compute: function (password, salt) {\n	            // Shortcut\n	            var cfg = this.cfg;\n\n	            // Init HMAC\n	            var hmac = HMAC.create(cfg.hasher, password);\n\n	            // Initial values\n	            var derivedKey = WordArray.create();\n	            var blockIndex = WordArray.create([0x00000001]);\n\n	            // Shortcuts\n	            var derivedKeyWords = derivedKey.words;\n	            var blockIndexWords = blockIndex.words;\n	            var keySize = cfg.keySize;\n	            var iterations = cfg.iterations;\n\n	            // Generate key\n	            while (derivedKeyWords.length < keySize) {\n	                var block = hmac.update(salt).finalize(blockIndex);\n	                hmac.reset();\n\n	                // Shortcuts\n	                var blockWords = block.words;\n	                var blockWordsLength = blockWords.length;\n\n	                // Iterations\n	                var intermediate = block;\n	                for (var i = 1; i < iterations; i++) {\n	                    intermediate = hmac.finalize(intermediate);\n	                    hmac.reset();\n\n	                    // Shortcut\n	                    var intermediateWords = intermediate.words;\n\n	                    // XOR intermediate with block\n	                    for (var j = 0; j < blockWordsLength; j++) {\n	                        blockWords[j] ^= intermediateWords[j];\n	                    }\n	                }\n\n	                derivedKey.concat(block);\n	                blockIndexWords[0]++;\n	            }\n	            derivedKey.sigBytes = keySize * 4;\n\n	            return derivedKey;\n	        }\n	    });\n\n	    /**\n	     * Computes the Password-Based Key Derivation Function 2.\n	     *\n	     * @param {WordArray|string} password The password.\n	     * @param {WordArray|string} salt A salt.\n	     * @param {Object} cfg (Optional) The configuration options to use for this computation.\n	     *\n	     * @return {WordArray} The derived key.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var key = CryptoJS.PBKDF2(password, salt);\n	     *     var key = CryptoJS.PBKDF2(password, salt, { keySize: 8 });\n	     *     var key = CryptoJS.PBKDF2(password, salt, { keySize: 8, iterations: 1000 });\n	     */\n	    C.PBKDF2 = function (password, salt, cfg) {\n	        return PBKDF2.create(cfg).compute(password, salt);\n	    };\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Base = C_lib.Base;\n	    var WordArray = C_lib.WordArray;\n	    var C_algo = C.algo;\n	    var MD5 = C_algo.MD5;\n\n	    /**\n	     * This key derivation function is meant to conform with EVP_BytesToKey.\n	     * www.openssl.org/docs/crypto/EVP_BytesToKey.html\n	     */\n	    var EvpKDF = C_algo.EvpKDF = Base.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {number} keySize The key size in words to generate. Default: 4 (128 bits)\n	         * @property {Hasher} hasher The hash algorithm to use. Default: MD5\n	         * @property {number} iterations The number of iterations to perform. Default: 1\n	         */\n	        cfg: Base.extend({\n	            keySize: 128/32,\n	            hasher: MD5,\n	            iterations: 1\n	        }),\n\n	        /**\n	         * Initializes a newly created key derivation function.\n	         *\n	         * @param {Object} cfg (Optional) The configuration options to use for the derivation.\n	         *\n	         * @example\n	         *\n	         *     var kdf = CryptoJS.algo.EvpKDF.create();\n	         *     var kdf = CryptoJS.algo.EvpKDF.create({ keySize: 8 });\n	         *     var kdf = CryptoJS.algo.EvpKDF.create({ keySize: 8, iterations: 1000 });\n	         */\n	        init: function (cfg) {\n	            this.cfg = this.cfg.extend(cfg);\n	        },\n\n	        /**\n	         * Derives a key from a password.\n	         *\n	         * @param {WordArray|string} password The password.\n	         * @param {WordArray|string} salt A salt.\n	         *\n	         * @return {WordArray} The derived key.\n	         *\n	         * @example\n	         *\n	         *     var key = kdf.compute(password, salt);\n	         */\n	        compute: function (password, salt) {\n	            var block;\n\n	            // Shortcut\n	            var cfg = this.cfg;\n\n	            // Init hasher\n	            var hasher = cfg.hasher.create();\n\n	            // Initial values\n	            var derivedKey = WordArray.create();\n\n	            // Shortcuts\n	            var derivedKeyWords = derivedKey.words;\n	            var keySize = cfg.keySize;\n	            var iterations = cfg.iterations;\n\n	            // Generate key\n	            while (derivedKeyWords.length < keySize) {\n	                if (block) {\n	                    hasher.update(block);\n	                }\n	                block = hasher.update(password).finalize(salt);\n	                hasher.reset();\n\n	                // Iterations\n	                for (var i = 1; i < iterations; i++) {\n	                    block = hasher.finalize(block);\n	                    hasher.reset();\n	                }\n\n	                derivedKey.concat(block);\n	            }\n	            derivedKey.sigBytes = keySize * 4;\n\n	            return derivedKey;\n	        }\n	    });\n\n	    /**\n	     * Derives a key from a password.\n	     *\n	     * @param {WordArray|string} password The password.\n	     * @param {WordArray|string} salt A salt.\n	     * @param {Object} cfg (Optional) The configuration options to use for this computation.\n	     *\n	     * @return {WordArray} The derived key.\n	     *\n	     * @static\n	     *\n	     * @example\n	     *\n	     *     var key = CryptoJS.EvpKDF(password, salt);\n	     *     var key = CryptoJS.EvpKDF(password, salt, { keySize: 8 });\n	     *     var key = CryptoJS.EvpKDF(password, salt, { keySize: 8, iterations: 1000 });\n	     */\n	    C.EvpKDF = function (password, salt, cfg) {\n	        return EvpKDF.create(cfg).compute(password, salt);\n	    };\n	}());\n\n\n	/**\n	 * Cipher core components.\n	 */\n	CryptoJS.lib.Cipher || (function (undefined) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var Base = C_lib.Base;\n	    var WordArray = C_lib.WordArray;\n	    var BufferedBlockAlgorithm = C_lib.BufferedBlockAlgorithm;\n	    var C_enc = C.enc;\n	    var Utf8 = C_enc.Utf8;\n	    var Base64 = C_enc.Base64;\n	    var C_algo = C.algo;\n	    var EvpKDF = C_algo.EvpKDF;\n\n	    /**\n	     * Abstract base cipher template.\n	     *\n	     * @property {number} keySize This cipher's key size. Default: 4 (128 bits)\n	     * @property {number} ivSize This cipher's IV size. Default: 4 (128 bits)\n	     * @property {number} _ENC_XFORM_MODE A constant representing encryption mode.\n	     * @property {number} _DEC_XFORM_MODE A constant representing decryption mode.\n	     */\n	    var Cipher = C_lib.Cipher = BufferedBlockAlgorithm.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {WordArray} iv The IV to use for this operation.\n	         */\n	        cfg: Base.extend(),\n\n	        /**\n	         * Creates this cipher in encryption mode.\n	         *\n	         * @param {WordArray} key The key.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {Cipher} A cipher instance.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var cipher = CryptoJS.algo.AES.createEncryptor(keyWordArray, { iv: ivWordArray });\n	         */\n	        createEncryptor: function (key, cfg) {\n	            return this.create(this._ENC_XFORM_MODE, key, cfg);\n	        },\n\n	        /**\n	         * Creates this cipher in decryption mode.\n	         *\n	         * @param {WordArray} key The key.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {Cipher} A cipher instance.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var cipher = CryptoJS.algo.AES.createDecryptor(keyWordArray, { iv: ivWordArray });\n	         */\n	        createDecryptor: function (key, cfg) {\n	            return this.create(this._DEC_XFORM_MODE, key, cfg);\n	        },\n\n	        /**\n	         * Initializes a newly created cipher.\n	         *\n	         * @param {number} xformMode Either the encryption or decryption transormation mode constant.\n	         * @param {WordArray} key The key.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @example\n	         *\n	         *     var cipher = CryptoJS.algo.AES.create(CryptoJS.algo.AES._ENC_XFORM_MODE, keyWordArray, { iv: ivWordArray });\n	         */\n	        init: function (xformMode, key, cfg) {\n	            // Apply config defaults\n	            this.cfg = this.cfg.extend(cfg);\n\n	            // Store transform mode and key\n	            this._xformMode = xformMode;\n	            this._key = key;\n\n	            // Set initial values\n	            this.reset();\n	        },\n\n	        /**\n	         * Resets this cipher to its initial state.\n	         *\n	         * @example\n	         *\n	         *     cipher.reset();\n	         */\n	        reset: function () {\n	            // Reset data buffer\n	            BufferedBlockAlgorithm.reset.call(this);\n\n	            // Perform concrete-cipher logic\n	            this._doReset();\n	        },\n\n	        /**\n	         * Adds data to be encrypted or decrypted.\n	         *\n	         * @param {WordArray|string} dataUpdate The data to encrypt or decrypt.\n	         *\n	         * @return {WordArray} The data after processing.\n	         *\n	         * @example\n	         *\n	         *     var encrypted = cipher.process('data');\n	         *     var encrypted = cipher.process(wordArray);\n	         */\n	        process: function (dataUpdate) {\n	            // Append\n	            this._append(dataUpdate);\n\n	            // Process available blocks\n	            return this._process();\n	        },\n\n	        /**\n	         * Finalizes the encryption or decryption process.\n	         * Note that the finalize operation is effectively a destructive, read-once operation.\n	         *\n	         * @param {WordArray|string} dataUpdate The final data to encrypt or decrypt.\n	         *\n	         * @return {WordArray} The data after final processing.\n	         *\n	         * @example\n	         *\n	         *     var encrypted = cipher.finalize();\n	         *     var encrypted = cipher.finalize('data');\n	         *     var encrypted = cipher.finalize(wordArray);\n	         */\n	        finalize: function (dataUpdate) {\n	            // Final data update\n	            if (dataUpdate) {\n	                this._append(dataUpdate);\n	            }\n\n	            // Perform concrete-cipher logic\n	            var finalProcessedData = this._doFinalize();\n\n	            return finalProcessedData;\n	        },\n\n	        keySize: 128/32,\n\n	        ivSize: 128/32,\n\n	        _ENC_XFORM_MODE: 1,\n\n	        _DEC_XFORM_MODE: 2,\n\n	        /**\n	         * Creates shortcut functions to a cipher's object interface.\n	         *\n	         * @param {Cipher} cipher The cipher to create a helper for.\n	         *\n	         * @return {Object} An object with encrypt and decrypt shortcut functions.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var AES = CryptoJS.lib.Cipher._createHelper(CryptoJS.algo.AES);\n	         */\n	        _createHelper: (function () {\n	            function selectCipherStrategy(key) {\n	                if (typeof key == 'string') {\n	                    return PasswordBasedCipher;\n	                } else {\n	                    return SerializableCipher;\n	                }\n	            }\n\n	            return function (cipher) {\n	                return {\n	                    encrypt: function (message, key, cfg) {\n	                        return selectCipherStrategy(key).encrypt(cipher, message, key, cfg);\n	                    },\n\n	                    decrypt: function (ciphertext, key, cfg) {\n	                        return selectCipherStrategy(key).decrypt(cipher, ciphertext, key, cfg);\n	                    }\n	                };\n	            };\n	        }())\n	    });\n\n	    /**\n	     * Abstract base stream cipher template.\n	     *\n	     * @property {number} blockSize The number of 32-bit words this cipher operates on. Default: 1 (32 bits)\n	     */\n	    var StreamCipher = C_lib.StreamCipher = Cipher.extend({\n	        _doFinalize: function () {\n	            // Process partial blocks\n	            var finalProcessedBlocks = this._process(!!'flush');\n\n	            return finalProcessedBlocks;\n	        },\n\n	        blockSize: 1\n	    });\n\n	    /**\n	     * Mode namespace.\n	     */\n	    var C_mode = C.mode = {};\n\n	    /**\n	     * Abstract base block cipher mode template.\n	     */\n	    var BlockCipherMode = C_lib.BlockCipherMode = Base.extend({\n	        /**\n	         * Creates this mode for encryption.\n	         *\n	         * @param {Cipher} cipher A block cipher instance.\n	         * @param {Array} iv The IV words.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var mode = CryptoJS.mode.CBC.createEncryptor(cipher, iv.words);\n	         */\n	        createEncryptor: function (cipher, iv) {\n	            return this.Encryptor.create(cipher, iv);\n	        },\n\n	        /**\n	         * Creates this mode for decryption.\n	         *\n	         * @param {Cipher} cipher A block cipher instance.\n	         * @param {Array} iv The IV words.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var mode = CryptoJS.mode.CBC.createDecryptor(cipher, iv.words);\n	         */\n	        createDecryptor: function (cipher, iv) {\n	            return this.Decryptor.create(cipher, iv);\n	        },\n\n	        /**\n	         * Initializes a newly created mode.\n	         *\n	         * @param {Cipher} cipher A block cipher instance.\n	         * @param {Array} iv The IV words.\n	         *\n	         * @example\n	         *\n	         *     var mode = CryptoJS.mode.CBC.Encryptor.create(cipher, iv.words);\n	         */\n	        init: function (cipher, iv) {\n	            this._cipher = cipher;\n	            this._iv = iv;\n	        }\n	    });\n\n	    /**\n	     * Cipher Block Chaining mode.\n	     */\n	    var CBC = C_mode.CBC = (function () {\n	        /**\n	         * Abstract base CBC mode.\n	         */\n	        var CBC = BlockCipherMode.extend();\n\n	        /**\n	         * CBC encryptor.\n	         */\n	        CBC.Encryptor = CBC.extend({\n	            /**\n	             * Processes the data block at offset.\n	             *\n	             * @param {Array} words The data words to operate on.\n	             * @param {number} offset The offset where the block starts.\n	             *\n	             * @example\n	             *\n	             *     mode.processBlock(data.words, offset);\n	             */\n	            processBlock: function (words, offset) {\n	                // Shortcuts\n	                var cipher = this._cipher;\n	                var blockSize = cipher.blockSize;\n\n	                // XOR and encrypt\n	                xorBlock.call(this, words, offset, blockSize);\n	                cipher.encryptBlock(words, offset);\n\n	                // Remember this block to use with next block\n	                this._prevBlock = words.slice(offset, offset + blockSize);\n	            }\n	        });\n\n	        /**\n	         * CBC decryptor.\n	         */\n	        CBC.Decryptor = CBC.extend({\n	            /**\n	             * Processes the data block at offset.\n	             *\n	             * @param {Array} words The data words to operate on.\n	             * @param {number} offset The offset where the block starts.\n	             *\n	             * @example\n	             *\n	             *     mode.processBlock(data.words, offset);\n	             */\n	            processBlock: function (words, offset) {\n	                // Shortcuts\n	                var cipher = this._cipher;\n	                var blockSize = cipher.blockSize;\n\n	                // Remember this block to use with next block\n	                var thisBlock = words.slice(offset, offset + blockSize);\n\n	                // Decrypt and XOR\n	                cipher.decryptBlock(words, offset);\n	                xorBlock.call(this, words, offset, blockSize);\n\n	                // This block becomes the previous block\n	                this._prevBlock = thisBlock;\n	            }\n	        });\n\n	        function xorBlock(words, offset, blockSize) {\n	            var block;\n\n	            // Shortcut\n	            var iv = this._iv;\n\n	            // Choose mixing block\n	            if (iv) {\n	                block = iv;\n\n	                // Remove IV for subsequent blocks\n	                this._iv = undefined;\n	            } else {\n	                block = this._prevBlock;\n	            }\n\n	            // XOR blocks\n	            for (var i = 0; i < blockSize; i++) {\n	                words[offset + i] ^= block[i];\n	            }\n	        }\n\n	        return CBC;\n	    }());\n\n	    /**\n	     * Padding namespace.\n	     */\n	    var C_pad = C.pad = {};\n\n	    /**\n	     * PKCS #5/7 padding strategy.\n	     */\n	    var Pkcs7 = C_pad.Pkcs7 = {\n	        /**\n	         * Pads data using the algorithm defined in PKCS #5/7.\n	         *\n	         * @param {WordArray} data The data to pad.\n	         * @param {number} blockSize The multiple that the data should be padded to.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     CryptoJS.pad.Pkcs7.pad(wordArray, 4);\n	         */\n	        pad: function (data, blockSize) {\n	            // Shortcut\n	            var blockSizeBytes = blockSize * 4;\n\n	            // Count padding bytes\n	            var nPaddingBytes = blockSizeBytes - data.sigBytes % blockSizeBytes;\n\n	            // Create padding word\n	            var paddingWord = (nPaddingBytes << 24) | (nPaddingBytes << 16) | (nPaddingBytes << 8) | nPaddingBytes;\n\n	            // Create padding\n	            var paddingWords = [];\n	            for (var i = 0; i < nPaddingBytes; i += 4) {\n	                paddingWords.push(paddingWord);\n	            }\n	            var padding = WordArray.create(paddingWords, nPaddingBytes);\n\n	            // Add padding\n	            data.concat(padding);\n	        },\n\n	        /**\n	         * Unpads data that had been padded using the algorithm defined in PKCS #5/7.\n	         *\n	         * @param {WordArray} data The data to unpad.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     CryptoJS.pad.Pkcs7.unpad(wordArray);\n	         */\n	        unpad: function (data) {\n	            // Get number of padding bytes from last byte\n	            var nPaddingBytes = data.words[(data.sigBytes - 1) >>> 2] & 0xff;\n\n	            // Remove padding\n	            data.sigBytes -= nPaddingBytes;\n	        }\n	    };\n\n	    /**\n	     * Abstract base block cipher template.\n	     *\n	     * @property {number} blockSize The number of 32-bit words this cipher operates on. Default: 4 (128 bits)\n	     */\n	    var BlockCipher = C_lib.BlockCipher = Cipher.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {Mode} mode The block mode to use. Default: CBC\n	         * @property {Padding} padding The padding strategy to use. Default: Pkcs7\n	         */\n	        cfg: Cipher.cfg.extend({\n	            mode: CBC,\n	            padding: Pkcs7\n	        }),\n\n	        reset: function () {\n	            var modeCreator;\n\n	            // Reset cipher\n	            Cipher.reset.call(this);\n\n	            // Shortcuts\n	            var cfg = this.cfg;\n	            var iv = cfg.iv;\n	            var mode = cfg.mode;\n\n	            // Reset block mode\n	            if (this._xformMode == this._ENC_XFORM_MODE) {\n	                modeCreator = mode.createEncryptor;\n	            } else /* if (this._xformMode == this._DEC_XFORM_MODE) */ {\n	                modeCreator = mode.createDecryptor;\n	                // Keep at least one block in the buffer for unpadding\n	                this._minBufferSize = 1;\n	            }\n\n	            if (this._mode && this._mode.__creator == modeCreator) {\n	                this._mode.init(this, iv && iv.words);\n	            } else {\n	                this._mode = modeCreator.call(mode, this, iv && iv.words);\n	                this._mode.__creator = modeCreator;\n	            }\n	        },\n\n	        _doProcessBlock: function (words, offset) {\n	            this._mode.processBlock(words, offset);\n	        },\n\n	        _doFinalize: function () {\n	            var finalProcessedBlocks;\n\n	            // Shortcut\n	            var padding = this.cfg.padding;\n\n	            // Finalize\n	            if (this._xformMode == this._ENC_XFORM_MODE) {\n	                // Pad data\n	                padding.pad(this._data, this.blockSize);\n\n	                // Process final blocks\n	                finalProcessedBlocks = this._process(!!'flush');\n	            } else /* if (this._xformMode == this._DEC_XFORM_MODE) */ {\n	                // Process final blocks\n	                finalProcessedBlocks = this._process(!!'flush');\n\n	                // Unpad data\n	                padding.unpad(finalProcessedBlocks);\n	            }\n\n	            return finalProcessedBlocks;\n	        },\n\n	        blockSize: 128/32\n	    });\n\n	    /**\n	     * A collection of cipher parameters.\n	     *\n	     * @property {WordArray} ciphertext The raw ciphertext.\n	     * @property {WordArray} key The key to this ciphertext.\n	     * @property {WordArray} iv The IV used in the ciphering operation.\n	     * @property {WordArray} salt The salt used with a key derivation function.\n	     * @property {Cipher} algorithm The cipher algorithm.\n	     * @property {Mode} mode The block mode used in the ciphering operation.\n	     * @property {Padding} padding The padding scheme used in the ciphering operation.\n	     * @property {number} blockSize The block size of the cipher.\n	     * @property {Format} formatter The default formatting strategy to convert this cipher params object to a string.\n	     */\n	    var CipherParams = C_lib.CipherParams = Base.extend({\n	        /**\n	         * Initializes a newly created cipher params object.\n	         *\n	         * @param {Object} cipherParams An object with any of the possible cipher parameters.\n	         *\n	         * @example\n	         *\n	         *     var cipherParams = CryptoJS.lib.CipherParams.create({\n	         *         ciphertext: ciphertextWordArray,\n	         *         key: keyWordArray,\n	         *         iv: ivWordArray,\n	         *         salt: saltWordArray,\n	         *         algorithm: CryptoJS.algo.AES,\n	         *         mode: CryptoJS.mode.CBC,\n	         *         padding: CryptoJS.pad.PKCS7,\n	         *         blockSize: 4,\n	         *         formatter: CryptoJS.format.OpenSSL\n	         *     });\n	         */\n	        init: function (cipherParams) {\n	            this.mixIn(cipherParams);\n	        },\n\n	        /**\n	         * Converts this cipher params object to a string.\n	         *\n	         * @param {Format} formatter (Optional) The formatting strategy to use.\n	         *\n	         * @return {string} The stringified cipher params.\n	         *\n	         * @throws Error If neither the formatter nor the default formatter is set.\n	         *\n	         * @example\n	         *\n	         *     var string = cipherParams + '';\n	         *     var string = cipherParams.toString();\n	         *     var string = cipherParams.toString(CryptoJS.format.OpenSSL);\n	         */\n	        toString: function (formatter) {\n	            return (formatter || this.formatter).stringify(this);\n	        }\n	    });\n\n	    /**\n	     * Format namespace.\n	     */\n	    var C_format = C.format = {};\n\n	    /**\n	     * OpenSSL formatting strategy.\n	     */\n	    var OpenSSLFormatter = C_format.OpenSSL = {\n	        /**\n	         * Converts a cipher params object to an OpenSSL-compatible string.\n	         *\n	         * @param {CipherParams} cipherParams The cipher params object.\n	         *\n	         * @return {string} The OpenSSL-compatible string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var openSSLString = CryptoJS.format.OpenSSL.stringify(cipherParams);\n	         */\n	        stringify: function (cipherParams) {\n	            var wordArray;\n\n	            // Shortcuts\n	            var ciphertext = cipherParams.ciphertext;\n	            var salt = cipherParams.salt;\n\n	            // Format\n	            if (salt) {\n	                wordArray = WordArray.create([0x53616c74, 0x65645f5f]).concat(salt).concat(ciphertext);\n	            } else {\n	                wordArray = ciphertext;\n	            }\n\n	            return wordArray.toString(Base64);\n	        },\n\n	        /**\n	         * Converts an OpenSSL-compatible string to a cipher params object.\n	         *\n	         * @param {string} openSSLStr The OpenSSL-compatible string.\n	         *\n	         * @return {CipherParams} The cipher params object.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var cipherParams = CryptoJS.format.OpenSSL.parse(openSSLString);\n	         */\n	        parse: function (openSSLStr) {\n	            var salt;\n\n	            // Parse base64\n	            var ciphertext = Base64.parse(openSSLStr);\n\n	            // Shortcut\n	            var ciphertextWords = ciphertext.words;\n\n	            // Test for salt\n	            if (ciphertextWords[0] == 0x53616c74 && ciphertextWords[1] == 0x65645f5f) {\n	                // Extract salt\n	                salt = WordArray.create(ciphertextWords.slice(2, 4));\n\n	                // Remove salt from ciphertext\n	                ciphertextWords.splice(0, 4);\n	                ciphertext.sigBytes -= 16;\n	            }\n\n	            return CipherParams.create({ ciphertext: ciphertext, salt: salt });\n	        }\n	    };\n\n	    /**\n	     * A cipher wrapper that returns ciphertext as a serializable cipher params object.\n	     */\n	    var SerializableCipher = C_lib.SerializableCipher = Base.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {Formatter} format The formatting strategy to convert cipher param objects to and from a string. Default: OpenSSL\n	         */\n	        cfg: Base.extend({\n	            format: OpenSSLFormatter\n	        }),\n\n	        /**\n	         * Encrypts a message.\n	         *\n	         * @param {Cipher} cipher The cipher algorithm to use.\n	         * @param {WordArray|string} message The message to encrypt.\n	         * @param {WordArray} key The key.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {CipherParams} A cipher params object.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var ciphertextParams = CryptoJS.lib.SerializableCipher.encrypt(CryptoJS.algo.AES, message, key);\n	         *     var ciphertextParams = CryptoJS.lib.SerializableCipher.encrypt(CryptoJS.algo.AES, message, key, { iv: iv });\n	         *     var ciphertextParams = CryptoJS.lib.SerializableCipher.encrypt(CryptoJS.algo.AES, message, key, { iv: iv, format: CryptoJS.format.OpenSSL });\n	         */\n	        encrypt: function (cipher, message, key, cfg) {\n	            // Apply config defaults\n	            cfg = this.cfg.extend(cfg);\n\n	            // Encrypt\n	            var encryptor = cipher.createEncryptor(key, cfg);\n	            var ciphertext = encryptor.finalize(message);\n\n	            // Shortcut\n	            var cipherCfg = encryptor.cfg;\n\n	            // Create and return serializable cipher params\n	            return CipherParams.create({\n	                ciphertext: ciphertext,\n	                key: key,\n	                iv: cipherCfg.iv,\n	                algorithm: cipher,\n	                mode: cipherCfg.mode,\n	                padding: cipherCfg.padding,\n	                blockSize: cipher.blockSize,\n	                formatter: cfg.format\n	            });\n	        },\n\n	        /**\n	         * Decrypts serialized ciphertext.\n	         *\n	         * @param {Cipher} cipher The cipher algorithm to use.\n	         * @param {CipherParams|string} ciphertext The ciphertext to decrypt.\n	         * @param {WordArray} key The key.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {WordArray} The plaintext.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var plaintext = CryptoJS.lib.SerializableCipher.decrypt(CryptoJS.algo.AES, formattedCiphertext, key, { iv: iv, format: CryptoJS.format.OpenSSL });\n	         *     var plaintext = CryptoJS.lib.SerializableCipher.decrypt(CryptoJS.algo.AES, ciphertextParams, key, { iv: iv, format: CryptoJS.format.OpenSSL });\n	         */\n	        decrypt: function (cipher, ciphertext, key, cfg) {\n	            // Apply config defaults\n	            cfg = this.cfg.extend(cfg);\n\n	            // Convert string to CipherParams\n	            ciphertext = this._parse(ciphertext, cfg.format);\n\n	            // Decrypt\n	            var plaintext = cipher.createDecryptor(key, cfg).finalize(ciphertext.ciphertext);\n\n	            return plaintext;\n	        },\n\n	        /**\n	         * Converts serialized ciphertext to CipherParams,\n	         * else assumed CipherParams already and returns ciphertext unchanged.\n	         *\n	         * @param {CipherParams|string} ciphertext The ciphertext.\n	         * @param {Formatter} format The formatting strategy to use to parse serialized ciphertext.\n	         *\n	         * @return {CipherParams} The unserialized ciphertext.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var ciphertextParams = CryptoJS.lib.SerializableCipher._parse(ciphertextStringOrParams, format);\n	         */\n	        _parse: function (ciphertext, format) {\n	            if (typeof ciphertext == 'string') {\n	                return format.parse(ciphertext, this);\n	            } else {\n	                return ciphertext;\n	            }\n	        }\n	    });\n\n	    /**\n	     * Key derivation function namespace.\n	     */\n	    var C_kdf = C.kdf = {};\n\n	    /**\n	     * OpenSSL key derivation function.\n	     */\n	    var OpenSSLKdf = C_kdf.OpenSSL = {\n	        /**\n	         * Derives a key and IV from a password.\n	         *\n	         * @param {string} password The password to derive from.\n	         * @param {number} keySize The size in words of the key to generate.\n	         * @param {number} ivSize The size in words of the IV to generate.\n	         * @param {WordArray|string} salt (Optional) A 64-bit salt to use. If omitted, a salt will be generated randomly.\n	         *\n	         * @return {CipherParams} A cipher params object with the key, IV, and salt.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var derivedParams = CryptoJS.kdf.OpenSSL.execute('Password', 256/32, 128/32);\n	         *     var derivedParams = CryptoJS.kdf.OpenSSL.execute('Password', 256/32, 128/32, 'saltsalt');\n	         */\n	        execute: function (password, keySize, ivSize, salt, hasher) {\n	            // Generate random salt\n	            if (!salt) {\n	                salt = WordArray.random(64/8);\n	            }\n\n	            // Derive key and IV\n	            if (!hasher) {\n	                var key = EvpKDF.create({ keySize: keySize + ivSize }).compute(password, salt);\n	            } else {\n	                var key = EvpKDF.create({ keySize: keySize + ivSize, hasher: hasher }).compute(password, salt);\n	            }\n\n\n	            // Separate key and IV\n	            var iv = WordArray.create(key.words.slice(keySize), ivSize * 4);\n	            key.sigBytes = keySize * 4;\n\n	            // Return params\n	            return CipherParams.create({ key: key, iv: iv, salt: salt });\n	        }\n	    };\n\n	    /**\n	     * A serializable cipher wrapper that derives the key from a password,\n	     * and returns ciphertext as a serializable cipher params object.\n	     */\n	    var PasswordBasedCipher = C_lib.PasswordBasedCipher = SerializableCipher.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {KDF} kdf The key derivation function to use to generate a key and IV from a password. Default: OpenSSL\n	         */\n	        cfg: SerializableCipher.cfg.extend({\n	            kdf: OpenSSLKdf\n	        }),\n\n	        /**\n	         * Encrypts a message using a password.\n	         *\n	         * @param {Cipher} cipher The cipher algorithm to use.\n	         * @param {WordArray|string} message The message to encrypt.\n	         * @param {string} password The password.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {CipherParams} A cipher params object.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var ciphertextParams = CryptoJS.lib.PasswordBasedCipher.encrypt(CryptoJS.algo.AES, message, 'password');\n	         *     var ciphertextParams = CryptoJS.lib.PasswordBasedCipher.encrypt(CryptoJS.algo.AES, message, 'password', { format: CryptoJS.format.OpenSSL });\n	         */\n	        encrypt: function (cipher, message, password, cfg) {\n	            // Apply config defaults\n	            cfg = this.cfg.extend(cfg);\n\n	            // Derive key and other params\n	            var derivedParams = cfg.kdf.execute(password, cipher.keySize, cipher.ivSize, cfg.salt, cfg.hasher);\n\n	            // Add IV to config\n	            cfg.iv = derivedParams.iv;\n\n	            // Encrypt\n	            var ciphertext = SerializableCipher.encrypt.call(this, cipher, message, derivedParams.key, cfg);\n\n	            // Mix in derived params\n	            ciphertext.mixIn(derivedParams);\n\n	            return ciphertext;\n	        },\n\n	        /**\n	         * Decrypts serialized ciphertext using a password.\n	         *\n	         * @param {Cipher} cipher The cipher algorithm to use.\n	         * @param {CipherParams|string} ciphertext The ciphertext to decrypt.\n	         * @param {string} password The password.\n	         * @param {Object} cfg (Optional) The configuration options to use for this operation.\n	         *\n	         * @return {WordArray} The plaintext.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var plaintext = CryptoJS.lib.PasswordBasedCipher.decrypt(CryptoJS.algo.AES, formattedCiphertext, 'password', { format: CryptoJS.format.OpenSSL });\n	         *     var plaintext = CryptoJS.lib.PasswordBasedCipher.decrypt(CryptoJS.algo.AES, ciphertextParams, 'password', { format: CryptoJS.format.OpenSSL });\n	         */\n	        decrypt: function (cipher, ciphertext, password, cfg) {\n	            // Apply config defaults\n	            cfg = this.cfg.extend(cfg);\n\n	            // Convert string to CipherParams\n	            ciphertext = this._parse(ciphertext, cfg.format);\n\n	            // Derive key and other params\n	            var derivedParams = cfg.kdf.execute(password, cipher.keySize, cipher.ivSize, ciphertext.salt, cfg.hasher);\n\n	            // Add IV to config\n	            cfg.iv = derivedParams.iv;\n\n	            // Decrypt\n	            var plaintext = SerializableCipher.decrypt.call(this, cipher, ciphertext, derivedParams.key, cfg);\n\n	            return plaintext;\n	        }\n	    });\n	}());\n\n\n	/**\n	 * Cipher Feedback block mode.\n	 */\n	CryptoJS.mode.CFB = (function () {\n	    var CFB = CryptoJS.lib.BlockCipherMode.extend();\n\n	    CFB.Encryptor = CFB.extend({\n	        processBlock: function (words, offset) {\n	            // Shortcuts\n	            var cipher = this._cipher;\n	            var blockSize = cipher.blockSize;\n\n	            generateKeystreamAndEncrypt.call(this, words, offset, blockSize, cipher);\n\n	            // Remember this block to use with next block\n	            this._prevBlock = words.slice(offset, offset + blockSize);\n	        }\n	    });\n\n	    CFB.Decryptor = CFB.extend({\n	        processBlock: function (words, offset) {\n	            // Shortcuts\n	            var cipher = this._cipher;\n	            var blockSize = cipher.blockSize;\n\n	            // Remember this block to use with next block\n	            var thisBlock = words.slice(offset, offset + blockSize);\n\n	            generateKeystreamAndEncrypt.call(this, words, offset, blockSize, cipher);\n\n	            // This block becomes the previous block\n	            this._prevBlock = thisBlock;\n	        }\n	    });\n\n	    function generateKeystreamAndEncrypt(words, offset, blockSize, cipher) {\n	        var keystream;\n\n	        // Shortcut\n	        var iv = this._iv;\n\n	        // Generate keystream\n	        if (iv) {\n	            keystream = iv.slice(0);\n\n	            // Remove IV for subsequent blocks\n	            this._iv = undefined;\n	        } else {\n	            keystream = this._prevBlock;\n	        }\n	        cipher.encryptBlock(keystream, 0);\n\n	        // Encrypt\n	        for (var i = 0; i < blockSize; i++) {\n	            words[offset + i] ^= keystream[i];\n	        }\n	    }\n\n	    return CFB;\n	}());\n\n\n	/**\n	 * Counter block mode.\n	 */\n	CryptoJS.mode.CTR = (function () {\n	    var CTR = CryptoJS.lib.BlockCipherMode.extend();\n\n	    var Encryptor = CTR.Encryptor = CTR.extend({\n	        processBlock: function (words, offset) {\n	            // Shortcuts\n	            var cipher = this._cipher\n	            var blockSize = cipher.blockSize;\n	            var iv = this._iv;\n	            var counter = this._counter;\n\n	            // Generate keystream\n	            if (iv) {\n	                counter = this._counter = iv.slice(0);\n\n	                // Remove IV for subsequent blocks\n	                this._iv = undefined;\n	            }\n	            var keystream = counter.slice(0);\n	            cipher.encryptBlock(keystream, 0);\n\n	            // Increment counter\n	            counter[blockSize - 1] = (counter[blockSize - 1] + 1) | 0\n\n	            // Encrypt\n	            for (var i = 0; i < blockSize; i++) {\n	                words[offset + i] ^= keystream[i];\n	            }\n	        }\n	    });\n\n	    CTR.Decryptor = Encryptor;\n\n	    return CTR;\n	}());\n\n\n	/** @preserve\n	 * Counter block mode compatible with  Dr Brian Gladman fileenc.c\n	 * derived from CryptoJS.mode.CTR\n	 * Jan Hruby jhruby.web@gmail.com\n	 */\n	CryptoJS.mode.CTRGladman = (function () {\n	    var CTRGladman = CryptoJS.lib.BlockCipherMode.extend();\n\n		function incWord(word)\n		{\n			if (((word >> 24) & 0xff) === 0xff) { //overflow\n			var b1 = (word >> 16)&0xff;\n			var b2 = (word >> 8)&0xff;\n			var b3 = word & 0xff;\n\n			if (b1 === 0xff) // overflow b1\n			{\n			b1 = 0;\n			if (b2 === 0xff)\n			{\n				b2 = 0;\n				if (b3 === 0xff)\n				{\n					b3 = 0;\n				}\n				else\n				{\n					++b3;\n				}\n			}\n			else\n			{\n				++b2;\n			}\n			}\n			else\n			{\n			++b1;\n			}\n\n			word = 0;\n			word += (b1 << 16);\n			word += (b2 << 8);\n			word += b3;\n			}\n			else\n			{\n			word += (0x01 << 24);\n			}\n			return word;\n		}\n\n		function incCounter(counter)\n		{\n			if ((counter[0] = incWord(counter[0])) === 0)\n			{\n				// encr_data in fileenc.c from  Dr Brian Gladman's counts only with DWORD j < 8\n				counter[1] = incWord(counter[1]);\n			}\n			return counter;\n		}\n\n	    var Encryptor = CTRGladman.Encryptor = CTRGladman.extend({\n	        processBlock: function (words, offset) {\n	            // Shortcuts\n	            var cipher = this._cipher\n	            var blockSize = cipher.blockSize;\n	            var iv = this._iv;\n	            var counter = this._counter;\n\n	            // Generate keystream\n	            if (iv) {\n	                counter = this._counter = iv.slice(0);\n\n	                // Remove IV for subsequent blocks\n	                this._iv = undefined;\n	            }\n\n				incCounter(counter);\n\n				var keystream = counter.slice(0);\n	            cipher.encryptBlock(keystream, 0);\n\n	            // Encrypt\n	            for (var i = 0; i < blockSize; i++) {\n	                words[offset + i] ^= keystream[i];\n	            }\n	        }\n	    });\n\n	    CTRGladman.Decryptor = Encryptor;\n\n	    return CTRGladman;\n	}());\n\n\n\n\n	/**\n	 * Output Feedback block mode.\n	 */\n	CryptoJS.mode.OFB = (function () {\n	    var OFB = CryptoJS.lib.BlockCipherMode.extend();\n\n	    var Encryptor = OFB.Encryptor = OFB.extend({\n	        processBlock: function (words, offset) {\n	            // Shortcuts\n	            var cipher = this._cipher\n	            var blockSize = cipher.blockSize;\n	            var iv = this._iv;\n	            var keystream = this._keystream;\n\n	            // Generate keystream\n	            if (iv) {\n	                keystream = this._keystream = iv.slice(0);\n\n	                // Remove IV for subsequent blocks\n	                this._iv = undefined;\n	            }\n	            cipher.encryptBlock(keystream, 0);\n\n	            // Encrypt\n	            for (var i = 0; i < blockSize; i++) {\n	                words[offset + i] ^= keystream[i];\n	            }\n	        }\n	    });\n\n	    OFB.Decryptor = Encryptor;\n\n	    return OFB;\n	}());\n\n\n	/**\n	 * Electronic Codebook block mode.\n	 */\n	CryptoJS.mode.ECB = (function () {\n	    var ECB = CryptoJS.lib.BlockCipherMode.extend();\n\n	    ECB.Encryptor = ECB.extend({\n	        processBlock: function (words, offset) {\n	            this._cipher.encryptBlock(words, offset);\n	        }\n	    });\n\n	    ECB.Decryptor = ECB.extend({\n	        processBlock: function (words, offset) {\n	            this._cipher.decryptBlock(words, offset);\n	        }\n	    });\n\n	    return ECB;\n	}());\n\n\n	/**\n	 * ANSI X.923 padding strategy.\n	 */\n	CryptoJS.pad.AnsiX923 = {\n	    pad: function (data, blockSize) {\n	        // Shortcuts\n	        var dataSigBytes = data.sigBytes;\n	        var blockSizeBytes = blockSize * 4;\n\n	        // Count padding bytes\n	        var nPaddingBytes = blockSizeBytes - dataSigBytes % blockSizeBytes;\n\n	        // Compute last byte position\n	        var lastBytePos = dataSigBytes + nPaddingBytes - 1;\n\n	        // Pad\n	        data.clamp();\n	        data.words[lastBytePos >>> 2] |= nPaddingBytes << (24 - (lastBytePos % 4) * 8);\n	        data.sigBytes += nPaddingBytes;\n	    },\n\n	    unpad: function (data) {\n	        // Get number of padding bytes from last byte\n	        var nPaddingBytes = data.words[(data.sigBytes - 1) >>> 2] & 0xff;\n\n	        // Remove padding\n	        data.sigBytes -= nPaddingBytes;\n	    }\n	};\n\n\n	/**\n	 * ISO 10126 padding strategy.\n	 */\n	CryptoJS.pad.Iso10126 = {\n	    pad: function (data, blockSize) {\n	        // Shortcut\n	        var blockSizeBytes = blockSize * 4;\n\n	        // Count padding bytes\n	        var nPaddingBytes = blockSizeBytes - data.sigBytes % blockSizeBytes;\n\n	        // Pad\n	        data.concat(CryptoJS.lib.WordArray.random(nPaddingBytes - 1)).\n	             concat(CryptoJS.lib.WordArray.create([nPaddingBytes << 24], 1));\n	    },\n\n	    unpad: function (data) {\n	        // Get number of padding bytes from last byte\n	        var nPaddingBytes = data.words[(data.sigBytes - 1) >>> 2] & 0xff;\n\n	        // Remove padding\n	        data.sigBytes -= nPaddingBytes;\n	    }\n	};\n\n\n	/**\n	 * ISO/IEC 9797-1 Padding Method 2.\n	 */\n	CryptoJS.pad.Iso97971 = {\n	    pad: function (data, blockSize) {\n	        // Add 0x80 byte\n	        data.concat(CryptoJS.lib.WordArray.create([0x80000000], 1));\n\n	        // Zero pad the rest\n	        CryptoJS.pad.ZeroPadding.pad(data, blockSize);\n	    },\n\n	    unpad: function (data) {\n	        // Remove zero padding\n	        CryptoJS.pad.ZeroPadding.unpad(data);\n\n	        // Remove one more byte -- the 0x80 byte\n	        data.sigBytes--;\n	    }\n	};\n\n\n	/**\n	 * Zero padding strategy.\n	 */\n	CryptoJS.pad.ZeroPadding = {\n	    pad: function (data, blockSize) {\n	        // Shortcut\n	        var blockSizeBytes = blockSize * 4;\n\n	        // Pad\n	        data.clamp();\n	        data.sigBytes += blockSizeBytes - ((data.sigBytes % blockSizeBytes) || blockSizeBytes);\n	    },\n\n	    unpad: function (data) {\n	        // Shortcut\n	        var dataWords = data.words;\n\n	        // Unpad\n	        var i = data.sigBytes - 1;\n	        for (var i = data.sigBytes - 1; i >= 0; i--) {\n	            if (((dataWords[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff)) {\n	                data.sigBytes = i + 1;\n	                break;\n	            }\n	        }\n	    }\n	};\n\n\n	/**\n	 * A noop padding strategy.\n	 */\n	CryptoJS.pad.NoPadding = {\n	    pad: function () {\n	    },\n\n	    unpad: function () {\n	    }\n	};\n\n\n	(function (undefined) {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var CipherParams = C_lib.CipherParams;\n	    var C_enc = C.enc;\n	    var Hex = C_enc.Hex;\n	    var C_format = C.format;\n\n	    var HexFormatter = C_format.Hex = {\n	        /**\n	         * Converts the ciphertext of a cipher params object to a hexadecimally encoded string.\n	         *\n	         * @param {CipherParams} cipherParams The cipher params object.\n	         *\n	         * @return {string} The hexadecimally encoded string.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var hexString = CryptoJS.format.Hex.stringify(cipherParams);\n	         */\n	        stringify: function (cipherParams) {\n	            return cipherParams.ciphertext.toString(Hex);\n	        },\n\n	        /**\n	         * Converts a hexadecimally encoded ciphertext string to a cipher params object.\n	         *\n	         * @param {string} input The hexadecimally encoded string.\n	         *\n	         * @return {CipherParams} The cipher params object.\n	         *\n	         * @static\n	         *\n	         * @example\n	         *\n	         *     var cipherParams = CryptoJS.format.Hex.parse(hexString);\n	         */\n	        parse: function (input) {\n	            var ciphertext = Hex.parse(input);\n	            return CipherParams.create({ ciphertext: ciphertext });\n	        }\n	    };\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var BlockCipher = C_lib.BlockCipher;\n	    var C_algo = C.algo;\n\n	    // Lookup tables\n	    var SBOX = [];\n	    var INV_SBOX = [];\n	    var SUB_MIX_0 = [];\n	    var SUB_MIX_1 = [];\n	    var SUB_MIX_2 = [];\n	    var SUB_MIX_3 = [];\n	    var INV_SUB_MIX_0 = [];\n	    var INV_SUB_MIX_1 = [];\n	    var INV_SUB_MIX_2 = [];\n	    var INV_SUB_MIX_3 = [];\n\n	    // Compute lookup tables\n	    (function () {\n	        // Compute double table\n	        var d = [];\n	        for (var i = 0; i < 256; i++) {\n	            if (i < 128) {\n	                d[i] = i << 1;\n	            } else {\n	                d[i] = (i << 1) ^ 0x11b;\n	            }\n	        }\n\n	        // Walk GF(2^8)\n	        var x = 0;\n	        var xi = 0;\n	        for (var i = 0; i < 256; i++) {\n	            // Compute sbox\n	            var sx = xi ^ (xi << 1) ^ (xi << 2) ^ (xi << 3) ^ (xi << 4);\n	            sx = (sx >>> 8) ^ (sx & 0xff) ^ 0x63;\n	            SBOX[x] = sx;\n	            INV_SBOX[sx] = x;\n\n	            // Compute multiplication\n	            var x2 = d[x];\n	            var x4 = d[x2];\n	            var x8 = d[x4];\n\n	            // Compute sub bytes, mix columns tables\n	            var t = (d[sx] * 0x101) ^ (sx * 0x1010100);\n	            SUB_MIX_0[x] = (t << 24) | (t >>> 8);\n	            SUB_MIX_1[x] = (t << 16) | (t >>> 16);\n	            SUB_MIX_2[x] = (t << 8)  | (t >>> 24);\n	            SUB_MIX_3[x] = t;\n\n	            // Compute inv sub bytes, inv mix columns tables\n	            var t = (x8 * 0x1010101) ^ (x4 * 0x10001) ^ (x2 * 0x101) ^ (x * 0x1010100);\n	            INV_SUB_MIX_0[sx] = (t << 24) | (t >>> 8);\n	            INV_SUB_MIX_1[sx] = (t << 16) | (t >>> 16);\n	            INV_SUB_MIX_2[sx] = (t << 8)  | (t >>> 24);\n	            INV_SUB_MIX_3[sx] = t;\n\n	            // Compute next counter\n	            if (!x) {\n	                x = xi = 1;\n	            } else {\n	                x = x2 ^ d[d[d[x8 ^ x2]]];\n	                xi ^= d[d[xi]];\n	            }\n	        }\n	    }());\n\n	    // Precomputed Rcon lookup\n	    var RCON = [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];\n\n	    /**\n	     * AES block cipher algorithm.\n	     */\n	    var AES = C_algo.AES = BlockCipher.extend({\n	        _doReset: function () {\n	            var t;\n\n	            // Skip reset of nRounds has been set before and key did not change\n	            if (this._nRounds && this._keyPriorReset === this._key) {\n	                return;\n	            }\n\n	            // Shortcuts\n	            var key = this._keyPriorReset = this._key;\n	            var keyWords = key.words;\n	            var keySize = key.sigBytes / 4;\n\n	            // Compute number of rounds\n	            var nRounds = this._nRounds = keySize + 6;\n\n	            // Compute number of key schedule rows\n	            var ksRows = (nRounds + 1) * 4;\n\n	            // Compute key schedule\n	            var keySchedule = this._keySchedule = [];\n	            for (var ksRow = 0; ksRow < ksRows; ksRow++) {\n	                if (ksRow < keySize) {\n	                    keySchedule[ksRow] = keyWords[ksRow];\n	                } else {\n	                    t = keySchedule[ksRow - 1];\n\n	                    if (!(ksRow % keySize)) {\n	                        // Rot word\n	                        t = (t << 8) | (t >>> 24);\n\n	                        // Sub word\n	                        t = (SBOX[t >>> 24] << 24) | (SBOX[(t >>> 16) & 0xff] << 16) | (SBOX[(t >>> 8) & 0xff] << 8) | SBOX[t & 0xff];\n\n	                        // Mix Rcon\n	                        t ^= RCON[(ksRow / keySize) | 0] << 24;\n	                    } else if (keySize > 6 && ksRow % keySize == 4) {\n	                        // Sub word\n	                        t = (SBOX[t >>> 24] << 24) | (SBOX[(t >>> 16) & 0xff] << 16) | (SBOX[(t >>> 8) & 0xff] << 8) | SBOX[t & 0xff];\n	                    }\n\n	                    keySchedule[ksRow] = keySchedule[ksRow - keySize] ^ t;\n	                }\n	            }\n\n	            // Compute inv key schedule\n	            var invKeySchedule = this._invKeySchedule = [];\n	            for (var invKsRow = 0; invKsRow < ksRows; invKsRow++) {\n	                var ksRow = ksRows - invKsRow;\n\n	                if (invKsRow % 4) {\n	                    var t = keySchedule[ksRow];\n	                } else {\n	                    var t = keySchedule[ksRow - 4];\n	                }\n\n	                if (invKsRow < 4 || ksRow <= 4) {\n	                    invKeySchedule[invKsRow] = t;\n	                } else {\n	                    invKeySchedule[invKsRow] = INV_SUB_MIX_0[SBOX[t >>> 24]] ^ INV_SUB_MIX_1[SBOX[(t >>> 16) & 0xff]] ^\n	                                               INV_SUB_MIX_2[SBOX[(t >>> 8) & 0xff]] ^ INV_SUB_MIX_3[SBOX[t & 0xff]];\n	                }\n	            }\n	        },\n\n	        encryptBlock: function (M, offset) {\n	            this._doCryptBlock(M, offset, this._keySchedule, SUB_MIX_0, SUB_MIX_1, SUB_MIX_2, SUB_MIX_3, SBOX);\n	        },\n\n	        decryptBlock: function (M, offset) {\n	            // Swap 2nd and 4th rows\n	            var t = M[offset + 1];\n	            M[offset + 1] = M[offset + 3];\n	            M[offset + 3] = t;\n\n	            this._doCryptBlock(M, offset, this._invKeySchedule, INV_SUB_MIX_0, INV_SUB_MIX_1, INV_SUB_MIX_2, INV_SUB_MIX_3, INV_SBOX);\n\n	            // Inv swap 2nd and 4th rows\n	            var t = M[offset + 1];\n	            M[offset + 1] = M[offset + 3];\n	            M[offset + 3] = t;\n	        },\n\n	        _doCryptBlock: function (M, offset, keySchedule, SUB_MIX_0, SUB_MIX_1, SUB_MIX_2, SUB_MIX_3, SBOX) {\n	            // Shortcut\n	            var nRounds = this._nRounds;\n\n	            // Get input, add round key\n	            var s0 = M[offset]     ^ keySchedule[0];\n	            var s1 = M[offset + 1] ^ keySchedule[1];\n	            var s2 = M[offset + 2] ^ keySchedule[2];\n	            var s3 = M[offset + 3] ^ keySchedule[3];\n\n	            // Key schedule row counter\n	            var ksRow = 4;\n\n	            // Rounds\n	            for (var round = 1; round < nRounds; round++) {\n	                // Shift rows, sub bytes, mix columns, add round key\n	                var t0 = SUB_MIX_0[s0 >>> 24] ^ SUB_MIX_1[(s1 >>> 16) & 0xff] ^ SUB_MIX_2[(s2 >>> 8) & 0xff] ^ SUB_MIX_3[s3 & 0xff] ^ keySchedule[ksRow++];\n	                var t1 = SUB_MIX_0[s1 >>> 24] ^ SUB_MIX_1[(s2 >>> 16) & 0xff] ^ SUB_MIX_2[(s3 >>> 8) & 0xff] ^ SUB_MIX_3[s0 & 0xff] ^ keySchedule[ksRow++];\n	                var t2 = SUB_MIX_0[s2 >>> 24] ^ SUB_MIX_1[(s3 >>> 16) & 0xff] ^ SUB_MIX_2[(s0 >>> 8) & 0xff] ^ SUB_MIX_3[s1 & 0xff] ^ keySchedule[ksRow++];\n	                var t3 = SUB_MIX_0[s3 >>> 24] ^ SUB_MIX_1[(s0 >>> 16) & 0xff] ^ SUB_MIX_2[(s1 >>> 8) & 0xff] ^ SUB_MIX_3[s2 & 0xff] ^ keySchedule[ksRow++];\n\n	                // Update state\n	                s0 = t0;\n	                s1 = t1;\n	                s2 = t2;\n	                s3 = t3;\n	            }\n\n	            // Shift rows, sub bytes, add round key\n	            var t0 = ((SBOX[s0 >>> 24] << 24) | (SBOX[(s1 >>> 16) & 0xff] << 16) | (SBOX[(s2 >>> 8) & 0xff] << 8) | SBOX[s3 & 0xff]) ^ keySchedule[ksRow++];\n	            var t1 = ((SBOX[s1 >>> 24] << 24) | (SBOX[(s2 >>> 16) & 0xff] << 16) | (SBOX[(s3 >>> 8) & 0xff] << 8) | SBOX[s0 & 0xff]) ^ keySchedule[ksRow++];\n	            var t2 = ((SBOX[s2 >>> 24] << 24) | (SBOX[(s3 >>> 16) & 0xff] << 16) | (SBOX[(s0 >>> 8) & 0xff] << 8) | SBOX[s1 & 0xff]) ^ keySchedule[ksRow++];\n	            var t3 = ((SBOX[s3 >>> 24] << 24) | (SBOX[(s0 >>> 16) & 0xff] << 16) | (SBOX[(s1 >>> 8) & 0xff] << 8) | SBOX[s2 & 0xff]) ^ keySchedule[ksRow++];\n\n	            // Set output\n	            M[offset]     = t0;\n	            M[offset + 1] = t1;\n	            M[offset + 2] = t2;\n	            M[offset + 3] = t3;\n	        },\n\n	        keySize: 256/32\n	    });\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.AES.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.AES.decrypt(ciphertext, key, cfg);\n	     */\n	    C.AES = BlockCipher._createHelper(AES);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var WordArray = C_lib.WordArray;\n	    var BlockCipher = C_lib.BlockCipher;\n	    var C_algo = C.algo;\n\n	    // Permuted Choice 1 constants\n	    var PC1 = [\n	        57, 49, 41, 33, 25, 17, 9,  1,\n	        58, 50, 42, 34, 26, 18, 10, 2,\n	        59, 51, 43, 35, 27, 19, 11, 3,\n	        60, 52, 44, 36, 63, 55, 47, 39,\n	        31, 23, 15, 7,  62, 54, 46, 38,\n	        30, 22, 14, 6,  61, 53, 45, 37,\n	        29, 21, 13, 5,  28, 20, 12, 4\n	    ];\n\n	    // Permuted Choice 2 constants\n	    var PC2 = [\n	        14, 17, 11, 24, 1,  5,\n	        3,  28, 15, 6,  21, 10,\n	        23, 19, 12, 4,  26, 8,\n	        16, 7,  27, 20, 13, 2,\n	        41, 52, 31, 37, 47, 55,\n	        30, 40, 51, 45, 33, 48,\n	        44, 49, 39, 56, 34, 53,\n	        46, 42, 50, 36, 29, 32\n	    ];\n\n	    // Cumulative bit shift constants\n	    var BIT_SHIFTS = [1,  2,  4,  6,  8,  10, 12, 14, 15, 17, 19, 21, 23, 25, 27, 28];\n\n	    // SBOXes and round permutation constants\n	    var SBOX_P = [\n	        {\n	            0x0: 0x808200,\n	            0x10000000: 0x8000,\n	            0x20000000: 0x808002,\n	            0x30000000: 0x2,\n	            0x40000000: 0x200,\n	            0x50000000: 0x808202,\n	            0x60000000: 0x800202,\n	            0x70000000: 0x800000,\n	            0x80000000: 0x202,\n	            0x90000000: 0x800200,\n	            0xa0000000: 0x8200,\n	            0xb0000000: 0x808000,\n	            0xc0000000: 0x8002,\n	            0xd0000000: 0x800002,\n	            0xe0000000: 0x0,\n	            0xf0000000: 0x8202,\n	            0x8000000: 0x0,\n	            0x18000000: 0x808202,\n	            0x28000000: 0x8202,\n	            0x38000000: 0x8000,\n	            0x48000000: 0x808200,\n	            0x58000000: 0x200,\n	            0x68000000: 0x808002,\n	            0x78000000: 0x2,\n	            0x88000000: 0x800200,\n	            0x98000000: 0x8200,\n	            0xa8000000: 0x808000,\n	            0xb8000000: 0x800202,\n	            0xc8000000: 0x800002,\n	            0xd8000000: 0x8002,\n	            0xe8000000: 0x202,\n	            0xf8000000: 0x800000,\n	            0x1: 0x8000,\n	            0x10000001: 0x2,\n	            0x20000001: 0x808200,\n	            0x30000001: 0x800000,\n	            0x40000001: 0x808002,\n	            0x50000001: 0x8200,\n	            0x60000001: 0x200,\n	            0x70000001: 0x800202,\n	            0x80000001: 0x808202,\n	            0x90000001: 0x808000,\n	            0xa0000001: 0x800002,\n	            0xb0000001: 0x8202,\n	            0xc0000001: 0x202,\n	            0xd0000001: 0x800200,\n	            0xe0000001: 0x8002,\n	            0xf0000001: 0x0,\n	            0x8000001: 0x808202,\n	            0x18000001: 0x808000,\n	            0x28000001: 0x800000,\n	            0x38000001: 0x200,\n	            0x48000001: 0x8000,\n	            0x58000001: 0x800002,\n	            0x68000001: 0x2,\n	            0x78000001: 0x8202,\n	            0x88000001: 0x8002,\n	            0x98000001: 0x800202,\n	            0xa8000001: 0x202,\n	            0xb8000001: 0x808200,\n	            0xc8000001: 0x800200,\n	            0xd8000001: 0x0,\n	            0xe8000001: 0x8200,\n	            0xf8000001: 0x808002\n	        },\n	        {\n	            0x0: 0x40084010,\n	            0x1000000: 0x4000,\n	            0x2000000: 0x80000,\n	            0x3000000: 0x40080010,\n	            0x4000000: 0x40000010,\n	            0x5000000: 0x40084000,\n	            0x6000000: 0x40004000,\n	            0x7000000: 0x10,\n	            0x8000000: 0x84000,\n	            0x9000000: 0x40004010,\n	            0xa000000: 0x40000000,\n	            0xb000000: 0x84010,\n	            0xc000000: 0x80010,\n	            0xd000000: 0x0,\n	            0xe000000: 0x4010,\n	            0xf000000: 0x40080000,\n	            0x800000: 0x40004000,\n	            0x1800000: 0x84010,\n	            0x2800000: 0x10,\n	            0x3800000: 0x40004010,\n	            0x4800000: 0x40084010,\n	            0x5800000: 0x40000000,\n	            0x6800000: 0x80000,\n	            0x7800000: 0x40080010,\n	            0x8800000: 0x80010,\n	            0x9800000: 0x0,\n	            0xa800000: 0x4000,\n	            0xb800000: 0x40080000,\n	            0xc800000: 0x40000010,\n	            0xd800000: 0x84000,\n	            0xe800000: 0x40084000,\n	            0xf800000: 0x4010,\n	            0x10000000: 0x0,\n	            0x11000000: 0x40080010,\n	            0x12000000: 0x40004010,\n	            0x13000000: 0x40084000,\n	            0x14000000: 0x40080000,\n	            0x15000000: 0x10,\n	            0x16000000: 0x84010,\n	            0x17000000: 0x4000,\n	            0x18000000: 0x4010,\n	            0x19000000: 0x80000,\n	            0x1a000000: 0x80010,\n	            0x1b000000: 0x40000010,\n	            0x1c000000: 0x84000,\n	            0x1d000000: 0x40004000,\n	            0x1e000000: 0x40000000,\n	            0x1f000000: 0x40084010,\n	            0x10800000: 0x84010,\n	            0x11800000: 0x80000,\n	            0x12800000: 0x40080000,\n	            0x13800000: 0x4000,\n	            0x14800000: 0x40004000,\n	            0x15800000: 0x40084010,\n	            0x16800000: 0x10,\n	            0x17800000: 0x40000000,\n	            0x18800000: 0x40084000,\n	            0x19800000: 0x40000010,\n	            0x1a800000: 0x40004010,\n	            0x1b800000: 0x80010,\n	            0x1c800000: 0x0,\n	            0x1d800000: 0x4010,\n	            0x1e800000: 0x40080010,\n	            0x1f800000: 0x84000\n	        },\n	        {\n	            0x0: 0x104,\n	            0x100000: 0x0,\n	            0x200000: 0x4000100,\n	            0x300000: 0x10104,\n	            0x400000: 0x10004,\n	            0x500000: 0x4000004,\n	            0x600000: 0x4010104,\n	            0x700000: 0x4010000,\n	            0x800000: 0x4000000,\n	            0x900000: 0x4010100,\n	            0xa00000: 0x10100,\n	            0xb00000: 0x4010004,\n	            0xc00000: 0x4000104,\n	            0xd00000: 0x10000,\n	            0xe00000: 0x4,\n	            0xf00000: 0x100,\n	            0x80000: 0x4010100,\n	            0x180000: 0x4010004,\n	            0x280000: 0x0,\n	            0x380000: 0x4000100,\n	            0x480000: 0x4000004,\n	            0x580000: 0x10000,\n	            0x680000: 0x10004,\n	            0x780000: 0x104,\n	            0x880000: 0x4,\n	            0x980000: 0x100,\n	            0xa80000: 0x4010000,\n	            0xb80000: 0x10104,\n	            0xc80000: 0x10100,\n	            0xd80000: 0x4000104,\n	            0xe80000: 0x4010104,\n	            0xf80000: 0x4000000,\n	            0x1000000: 0x4010100,\n	            0x1100000: 0x10004,\n	            0x1200000: 0x10000,\n	            0x1300000: 0x4000100,\n	            0x1400000: 0x100,\n	            0x1500000: 0x4010104,\n	            0x1600000: 0x4000004,\n	            0x1700000: 0x0,\n	            0x1800000: 0x4000104,\n	            0x1900000: 0x4000000,\n	            0x1a00000: 0x4,\n	            0x1b00000: 0x10100,\n	            0x1c00000: 0x4010000,\n	            0x1d00000: 0x104,\n	            0x1e00000: 0x10104,\n	            0x1f00000: 0x4010004,\n	            0x1080000: 0x4000000,\n	            0x1180000: 0x104,\n	            0x1280000: 0x4010100,\n	            0x1380000: 0x0,\n	            0x1480000: 0x10004,\n	            0x1580000: 0x4000100,\n	            0x1680000: 0x100,\n	            0x1780000: 0x4010004,\n	            0x1880000: 0x10000,\n	            0x1980000: 0x4010104,\n	            0x1a80000: 0x10104,\n	            0x1b80000: 0x4000004,\n	            0x1c80000: 0x4000104,\n	            0x1d80000: 0x4010000,\n	            0x1e80000: 0x4,\n	            0x1f80000: 0x10100\n	        },\n	        {\n	            0x0: 0x80401000,\n	            0x10000: 0x80001040,\n	            0x20000: 0x401040,\n	            0x30000: 0x80400000,\n	            0x40000: 0x0,\n	            0x50000: 0x401000,\n	            0x60000: 0x80000040,\n	            0x70000: 0x400040,\n	            0x80000: 0x80000000,\n	            0x90000: 0x400000,\n	            0xa0000: 0x40,\n	            0xb0000: 0x80001000,\n	            0xc0000: 0x80400040,\n	            0xd0000: 0x1040,\n	            0xe0000: 0x1000,\n	            0xf0000: 0x80401040,\n	            0x8000: 0x80001040,\n	            0x18000: 0x40,\n	            0x28000: 0x80400040,\n	            0x38000: 0x80001000,\n	            0x48000: 0x401000,\n	            0x58000: 0x80401040,\n	            0x68000: 0x0,\n	            0x78000: 0x80400000,\n	            0x88000: 0x1000,\n	            0x98000: 0x80401000,\n	            0xa8000: 0x400000,\n	            0xb8000: 0x1040,\n	            0xc8000: 0x80000000,\n	            0xd8000: 0x400040,\n	            0xe8000: 0x401040,\n	            0xf8000: 0x80000040,\n	            0x100000: 0x400040,\n	            0x110000: 0x401000,\n	            0x120000: 0x80000040,\n	            0x130000: 0x0,\n	            0x140000: 0x1040,\n	            0x150000: 0x80400040,\n	            0x160000: 0x80401000,\n	            0x170000: 0x80001040,\n	            0x180000: 0x80401040,\n	            0x190000: 0x80000000,\n	            0x1a0000: 0x80400000,\n	            0x1b0000: 0x401040,\n	            0x1c0000: 0x80001000,\n	            0x1d0000: 0x400000,\n	            0x1e0000: 0x40,\n	            0x1f0000: 0x1000,\n	            0x108000: 0x80400000,\n	            0x118000: 0x80401040,\n	            0x128000: 0x0,\n	            0x138000: 0x401000,\n	            0x148000: 0x400040,\n	            0x158000: 0x80000000,\n	            0x168000: 0x80001040,\n	            0x178000: 0x40,\n	            0x188000: 0x80000040,\n	            0x198000: 0x1000,\n	            0x1a8000: 0x80001000,\n	            0x1b8000: 0x80400040,\n	            0x1c8000: 0x1040,\n	            0x1d8000: 0x80401000,\n	            0x1e8000: 0x400000,\n	            0x1f8000: 0x401040\n	        },\n	        {\n	            0x0: 0x80,\n	            0x1000: 0x1040000,\n	            0x2000: 0x40000,\n	            0x3000: 0x20000000,\n	            0x4000: 0x20040080,\n	            0x5000: 0x1000080,\n	            0x6000: 0x21000080,\n	            0x7000: 0x40080,\n	            0x8000: 0x1000000,\n	            0x9000: 0x20040000,\n	            0xa000: 0x20000080,\n	            0xb000: 0x21040080,\n	            0xc000: 0x21040000,\n	            0xd000: 0x0,\n	            0xe000: 0x1040080,\n	            0xf000: 0x21000000,\n	            0x800: 0x1040080,\n	            0x1800: 0x21000080,\n	            0x2800: 0x80,\n	            0x3800: 0x1040000,\n	            0x4800: 0x40000,\n	            0x5800: 0x20040080,\n	            0x6800: 0x21040000,\n	            0x7800: 0x20000000,\n	            0x8800: 0x20040000,\n	            0x9800: 0x0,\n	            0xa800: 0x21040080,\n	            0xb800: 0x1000080,\n	            0xc800: 0x20000080,\n	            0xd800: 0x21000000,\n	            0xe800: 0x1000000,\n	            0xf800: 0x40080,\n	            0x10000: 0x40000,\n	            0x11000: 0x80,\n	            0x12000: 0x20000000,\n	            0x13000: 0x21000080,\n	            0x14000: 0x1000080,\n	            0x15000: 0x21040000,\n	            0x16000: 0x20040080,\n	            0x17000: 0x1000000,\n	            0x18000: 0x21040080,\n	            0x19000: 0x21000000,\n	            0x1a000: 0x1040000,\n	            0x1b000: 0x20040000,\n	            0x1c000: 0x40080,\n	            0x1d000: 0x20000080,\n	            0x1e000: 0x0,\n	            0x1f000: 0x1040080,\n	            0x10800: 0x21000080,\n	            0x11800: 0x1000000,\n	            0x12800: 0x1040000,\n	            0x13800: 0x20040080,\n	            0x14800: 0x20000000,\n	            0x15800: 0x1040080,\n	            0x16800: 0x80,\n	            0x17800: 0x21040000,\n	            0x18800: 0x40080,\n	            0x19800: 0x21040080,\n	            0x1a800: 0x0,\n	            0x1b800: 0x21000000,\n	            0x1c800: 0x1000080,\n	            0x1d800: 0x40000,\n	            0x1e800: 0x20040000,\n	            0x1f800: 0x20000080\n	        },\n	        {\n	            0x0: 0x10000008,\n	            0x100: 0x2000,\n	            0x200: 0x10200000,\n	            0x300: 0x10202008,\n	            0x400: 0x10002000,\n	            0x500: 0x200000,\n	            0x600: 0x200008,\n	            0x700: 0x10000000,\n	            0x800: 0x0,\n	            0x900: 0x10002008,\n	            0xa00: 0x202000,\n	            0xb00: 0x8,\n	            0xc00: 0x10200008,\n	            0xd00: 0x202008,\n	            0xe00: 0x2008,\n	            0xf00: 0x10202000,\n	            0x80: 0x10200000,\n	            0x180: 0x10202008,\n	            0x280: 0x8,\n	            0x380: 0x200000,\n	            0x480: 0x202008,\n	            0x580: 0x10000008,\n	            0x680: 0x10002000,\n	            0x780: 0x2008,\n	            0x880: 0x200008,\n	            0x980: 0x2000,\n	            0xa80: 0x10002008,\n	            0xb80: 0x10200008,\n	            0xc80: 0x0,\n	            0xd80: 0x10202000,\n	            0xe80: 0x202000,\n	            0xf80: 0x10000000,\n	            0x1000: 0x10002000,\n	            0x1100: 0x10200008,\n	            0x1200: 0x10202008,\n	            0x1300: 0x2008,\n	            0x1400: 0x200000,\n	            0x1500: 0x10000000,\n	            0x1600: 0x10000008,\n	            0x1700: 0x202000,\n	            0x1800: 0x202008,\n	            0x1900: 0x0,\n	            0x1a00: 0x8,\n	            0x1b00: 0x10200000,\n	            0x1c00: 0x2000,\n	            0x1d00: 0x10002008,\n	            0x1e00: 0x10202000,\n	            0x1f00: 0x200008,\n	            0x1080: 0x8,\n	            0x1180: 0x202000,\n	            0x1280: 0x200000,\n	            0x1380: 0x10000008,\n	            0x1480: 0x10002000,\n	            0x1580: 0x2008,\n	            0x1680: 0x10202008,\n	            0x1780: 0x10200000,\n	            0x1880: 0x10202000,\n	            0x1980: 0x10200008,\n	            0x1a80: 0x2000,\n	            0x1b80: 0x202008,\n	            0x1c80: 0x200008,\n	            0x1d80: 0x0,\n	            0x1e80: 0x10000000,\n	            0x1f80: 0x10002008\n	        },\n	        {\n	            0x0: 0x100000,\n	            0x10: 0x2000401,\n	            0x20: 0x400,\n	            0x30: 0x100401,\n	            0x40: 0x2100401,\n	            0x50: 0x0,\n	            0x60: 0x1,\n	            0x70: 0x2100001,\n	            0x80: 0x2000400,\n	            0x90: 0x100001,\n	            0xa0: 0x2000001,\n	            0xb0: 0x2100400,\n	            0xc0: 0x2100000,\n	            0xd0: 0x401,\n	            0xe0: 0x100400,\n	            0xf0: 0x2000000,\n	            0x8: 0x2100001,\n	            0x18: 0x0,\n	            0x28: 0x2000401,\n	            0x38: 0x2100400,\n	            0x48: 0x100000,\n	            0x58: 0x2000001,\n	            0x68: 0x2000000,\n	            0x78: 0x401,\n	            0x88: 0x100401,\n	            0x98: 0x2000400,\n	            0xa8: 0x2100000,\n	            0xb8: 0x100001,\n	            0xc8: 0x400,\n	            0xd8: 0x2100401,\n	            0xe8: 0x1,\n	            0xf8: 0x100400,\n	            0x100: 0x2000000,\n	            0x110: 0x100000,\n	            0x120: 0x2000401,\n	            0x130: 0x2100001,\n	            0x140: 0x100001,\n	            0x150: 0x2000400,\n	            0x160: 0x2100400,\n	            0x170: 0x100401,\n	            0x180: 0x401,\n	            0x190: 0x2100401,\n	            0x1a0: 0x100400,\n	            0x1b0: 0x1,\n	            0x1c0: 0x0,\n	            0x1d0: 0x2100000,\n	            0x1e0: 0x2000001,\n	            0x1f0: 0x400,\n	            0x108: 0x100400,\n	            0x118: 0x2000401,\n	            0x128: 0x2100001,\n	            0x138: 0x1,\n	            0x148: 0x2000000,\n	            0x158: 0x100000,\n	            0x168: 0x401,\n	            0x178: 0x2100400,\n	            0x188: 0x2000001,\n	            0x198: 0x2100000,\n	            0x1a8: 0x0,\n	            0x1b8: 0x2100401,\n	            0x1c8: 0x100401,\n	            0x1d8: 0x400,\n	            0x1e8: 0x2000400,\n	            0x1f8: 0x100001\n	        },\n	        {\n	            0x0: 0x8000820,\n	            0x1: 0x20000,\n	            0x2: 0x8000000,\n	            0x3: 0x20,\n	            0x4: 0x20020,\n	            0x5: 0x8020820,\n	            0x6: 0x8020800,\n	            0x7: 0x800,\n	            0x8: 0x8020000,\n	            0x9: 0x8000800,\n	            0xa: 0x20800,\n	            0xb: 0x8020020,\n	            0xc: 0x820,\n	            0xd: 0x0,\n	            0xe: 0x8000020,\n	            0xf: 0x20820,\n	            0x80000000: 0x800,\n	            0x80000001: 0x8020820,\n	            0x80000002: 0x8000820,\n	            0x80000003: 0x8000000,\n	            0x80000004: 0x8020000,\n	            0x80000005: 0x20800,\n	            0x80000006: 0x20820,\n	            0x80000007: 0x20,\n	            0x80000008: 0x8000020,\n	            0x80000009: 0x820,\n	            0x8000000a: 0x20020,\n	            0x8000000b: 0x8020800,\n	            0x8000000c: 0x0,\n	            0x8000000d: 0x8020020,\n	            0x8000000e: 0x8000800,\n	            0x8000000f: 0x20000,\n	            0x10: 0x20820,\n	            0x11: 0x8020800,\n	            0x12: 0x20,\n	            0x13: 0x800,\n	            0x14: 0x8000800,\n	            0x15: 0x8000020,\n	            0x16: 0x8020020,\n	            0x17: 0x20000,\n	            0x18: 0x0,\n	            0x19: 0x20020,\n	            0x1a: 0x8020000,\n	            0x1b: 0x8000820,\n	            0x1c: 0x8020820,\n	            0x1d: 0x20800,\n	            0x1e: 0x820,\n	            0x1f: 0x8000000,\n	            0x80000010: 0x20000,\n	            0x80000011: 0x800,\n	            0x80000012: 0x8020020,\n	            0x80000013: 0x20820,\n	            0x80000014: 0x20,\n	            0x80000015: 0x8020000,\n	            0x80000016: 0x8000000,\n	            0x80000017: 0x8000820,\n	            0x80000018: 0x8020820,\n	            0x80000019: 0x8000020,\n	            0x8000001a: 0x8000800,\n	            0x8000001b: 0x0,\n	            0x8000001c: 0x20800,\n	            0x8000001d: 0x820,\n	            0x8000001e: 0x20020,\n	            0x8000001f: 0x8020800\n	        }\n	    ];\n\n	    // Masks that select the SBOX input\n	    var SBOX_MASK = [\n	        0xf8000001, 0x1f800000, 0x01f80000, 0x001f8000,\n	        0x0001f800, 0x00001f80, 0x000001f8, 0x8000001f\n	    ];\n\n	    /**\n	     * DES block cipher algorithm.\n	     */\n	    var DES = C_algo.DES = BlockCipher.extend({\n	        _doReset: function () {\n	            // Shortcuts\n	            var key = this._key;\n	            var keyWords = key.words;\n\n	            // Select 56 bits according to PC1\n	            var keyBits = [];\n	            for (var i = 0; i < 56; i++) {\n	                var keyBitPos = PC1[i] - 1;\n	                keyBits[i] = (keyWords[keyBitPos >>> 5] >>> (31 - keyBitPos % 32)) & 1;\n	            }\n\n	            // Assemble 16 subkeys\n	            var subKeys = this._subKeys = [];\n	            for (var nSubKey = 0; nSubKey < 16; nSubKey++) {\n	                // Create subkey\n	                var subKey = subKeys[nSubKey] = [];\n\n	                // Shortcut\n	                var bitShift = BIT_SHIFTS[nSubKey];\n\n	                // Select 48 bits according to PC2\n	                for (var i = 0; i < 24; i++) {\n	                    // Select from the left 28 key bits\n	                    subKey[(i / 6) | 0] |= keyBits[((PC2[i] - 1) + bitShift) % 28] << (31 - i % 6);\n\n	                    // Select from the right 28 key bits\n	                    subKey[4 + ((i / 6) | 0)] |= keyBits[28 + (((PC2[i + 24] - 1) + bitShift) % 28)] << (31 - i % 6);\n	                }\n\n	                // Since each subkey is applied to an expanded 32-bit input,\n	                // the subkey can be broken into 8 values scaled to 32-bits,\n	                // which allows the key to be used without expansion\n	                subKey[0] = (subKey[0] << 1) | (subKey[0] >>> 31);\n	                for (var i = 1; i < 7; i++) {\n	                    subKey[i] = subKey[i] >>> ((i - 1) * 4 + 3);\n	                }\n	                subKey[7] = (subKey[7] << 5) | (subKey[7] >>> 27);\n	            }\n\n	            // Compute inverse subkeys\n	            var invSubKeys = this._invSubKeys = [];\n	            for (var i = 0; i < 16; i++) {\n	                invSubKeys[i] = subKeys[15 - i];\n	            }\n	        },\n\n	        encryptBlock: function (M, offset) {\n	            this._doCryptBlock(M, offset, this._subKeys);\n	        },\n\n	        decryptBlock: function (M, offset) {\n	            this._doCryptBlock(M, offset, this._invSubKeys);\n	        },\n\n	        _doCryptBlock: function (M, offset, subKeys) {\n	            // Get input\n	            this._lBlock = M[offset];\n	            this._rBlock = M[offset + 1];\n\n	            // Initial permutation\n	            exchangeLR.call(this, 4,  0x0f0f0f0f);\n	            exchangeLR.call(this, 16, 0x0000ffff);\n	            exchangeRL.call(this, 2,  0x33333333);\n	            exchangeRL.call(this, 8,  0x00ff00ff);\n	            exchangeLR.call(this, 1,  0x55555555);\n\n	            // Rounds\n	            for (var round = 0; round < 16; round++) {\n	                // Shortcuts\n	                var subKey = subKeys[round];\n	                var lBlock = this._lBlock;\n	                var rBlock = this._rBlock;\n\n	                // Feistel function\n	                var f = 0;\n	                for (var i = 0; i < 8; i++) {\n	                    f |= SBOX_P[i][((rBlock ^ subKey[i]) & SBOX_MASK[i]) >>> 0];\n	                }\n	                this._lBlock = rBlock;\n	                this._rBlock = lBlock ^ f;\n	            }\n\n	            // Undo swap from last round\n	            var t = this._lBlock;\n	            this._lBlock = this._rBlock;\n	            this._rBlock = t;\n\n	            // Final permutation\n	            exchangeLR.call(this, 1,  0x55555555);\n	            exchangeRL.call(this, 8,  0x00ff00ff);\n	            exchangeRL.call(this, 2,  0x33333333);\n	            exchangeLR.call(this, 16, 0x0000ffff);\n	            exchangeLR.call(this, 4,  0x0f0f0f0f);\n\n	            // Set output\n	            M[offset] = this._lBlock;\n	            M[offset + 1] = this._rBlock;\n	        },\n\n	        keySize: 64/32,\n\n	        ivSize: 64/32,\n\n	        blockSize: 64/32\n	    });\n\n	    // Swap bits across the left and right words\n	    function exchangeLR(offset, mask) {\n	        var t = ((this._lBlock >>> offset) ^ this._rBlock) & mask;\n	        this._rBlock ^= t;\n	        this._lBlock ^= t << offset;\n	    }\n\n	    function exchangeRL(offset, mask) {\n	        var t = ((this._rBlock >>> offset) ^ this._lBlock) & mask;\n	        this._lBlock ^= t;\n	        this._rBlock ^= t << offset;\n	    }\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.DES.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.DES.decrypt(ciphertext, key, cfg);\n	     */\n	    C.DES = BlockCipher._createHelper(DES);\n\n	    /**\n	     * Triple-DES block cipher algorithm.\n	     */\n	    var TripleDES = C_algo.TripleDES = BlockCipher.extend({\n	        _doReset: function () {\n	            // Shortcuts\n	            var key = this._key;\n	            var keyWords = key.words;\n	            // Make sure the key length is valid (64, 128 or >= 192 bit)\n	            if (keyWords.length !== 2 && keyWords.length !== 4 && keyWords.length < 6) {\n	                throw new Error('Invalid key length - 3DES requires the key length to be 64, 128, 192 or >192.');\n	            }\n\n	            // Extend the key according to the keying options defined in 3DES standard\n	            var key1 = keyWords.slice(0, 2);\n	            var key2 = keyWords.length < 4 ? keyWords.slice(0, 2) : keyWords.slice(2, 4);\n	            var key3 = keyWords.length < 6 ? keyWords.slice(0, 2) : keyWords.slice(4, 6);\n\n	            // Create DES instances\n	            this._des1 = DES.createEncryptor(WordArray.create(key1));\n	            this._des2 = DES.createEncryptor(WordArray.create(key2));\n	            this._des3 = DES.createEncryptor(WordArray.create(key3));\n	        },\n\n	        encryptBlock: function (M, offset) {\n	            this._des1.encryptBlock(M, offset);\n	            this._des2.decryptBlock(M, offset);\n	            this._des3.encryptBlock(M, offset);\n	        },\n\n	        decryptBlock: function (M, offset) {\n	            this._des3.decryptBlock(M, offset);\n	            this._des2.encryptBlock(M, offset);\n	            this._des1.decryptBlock(M, offset);\n	        },\n\n	        keySize: 192/32,\n\n	        ivSize: 64/32,\n\n	        blockSize: 64/32\n	    });\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.TripleDES.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.TripleDES.decrypt(ciphertext, key, cfg);\n	     */\n	    C.TripleDES = BlockCipher._createHelper(TripleDES);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var StreamCipher = C_lib.StreamCipher;\n	    var C_algo = C.algo;\n\n	    /**\n	     * RC4 stream cipher algorithm.\n	     */\n	    var RC4 = C_algo.RC4 = StreamCipher.extend({\n	        _doReset: function () {\n	            // Shortcuts\n	            var key = this._key;\n	            var keyWords = key.words;\n	            var keySigBytes = key.sigBytes;\n\n	            // Init sbox\n	            var S = this._S = [];\n	            for (var i = 0; i < 256; i++) {\n	                S[i] = i;\n	            }\n\n	            // Key setup\n	            for (var i = 0, j = 0; i < 256; i++) {\n	                var keyByteIndex = i % keySigBytes;\n	                var keyByte = (keyWords[keyByteIndex >>> 2] >>> (24 - (keyByteIndex % 4) * 8)) & 0xff;\n\n	                j = (j + S[i] + keyByte) % 256;\n\n	                // Swap\n	                var t = S[i];\n	                S[i] = S[j];\n	                S[j] = t;\n	            }\n\n	            // Counters\n	            this._i = this._j = 0;\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            M[offset] ^= generateKeystreamWord.call(this);\n	        },\n\n	        keySize: 256/32,\n\n	        ivSize: 0\n	    });\n\n	    function generateKeystreamWord() {\n	        // Shortcuts\n	        var S = this._S;\n	        var i = this._i;\n	        var j = this._j;\n\n	        // Generate keystream word\n	        var keystreamWord = 0;\n	        for (var n = 0; n < 4; n++) {\n	            i = (i + 1) % 256;\n	            j = (j + S[i]) % 256;\n\n	            // Swap\n	            var t = S[i];\n	            S[i] = S[j];\n	            S[j] = t;\n\n	            keystreamWord |= S[(S[i] + S[j]) % 256] << (24 - n * 8);\n	        }\n\n	        // Update counters\n	        this._i = i;\n	        this._j = j;\n\n	        return keystreamWord;\n	    }\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.RC4.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.RC4.decrypt(ciphertext, key, cfg);\n	     */\n	    C.RC4 = StreamCipher._createHelper(RC4);\n\n	    /**\n	     * Modified RC4 stream cipher algorithm.\n	     */\n	    var RC4Drop = C_algo.RC4Drop = RC4.extend({\n	        /**\n	         * Configuration options.\n	         *\n	         * @property {number} drop The number of keystream words to drop. Default 192\n	         */\n	        cfg: RC4.cfg.extend({\n	            drop: 192\n	        }),\n\n	        _doReset: function () {\n	            RC4._doReset.call(this);\n\n	            // Drop\n	            for (var i = this.cfg.drop; i > 0; i--) {\n	                generateKeystreamWord.call(this);\n	            }\n	        }\n	    });\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.RC4Drop.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.RC4Drop.decrypt(ciphertext, key, cfg);\n	     */\n	    C.RC4Drop = StreamCipher._createHelper(RC4Drop);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var StreamCipher = C_lib.StreamCipher;\n	    var C_algo = C.algo;\n\n	    // Reusable objects\n	    var S  = [];\n	    var C_ = [];\n	    var G  = [];\n\n	    /**\n	     * Rabbit stream cipher algorithm\n	     */\n	    var Rabbit = C_algo.Rabbit = StreamCipher.extend({\n	        _doReset: function () {\n	            // Shortcuts\n	            var K = this._key.words;\n	            var iv = this.cfg.iv;\n\n	            // Swap endian\n	            for (var i = 0; i < 4; i++) {\n	                K[i] = (((K[i] << 8)  | (K[i] >>> 24)) & 0x00ff00ff) |\n	                       (((K[i] << 24) | (K[i] >>> 8))  & 0xff00ff00);\n	            }\n\n	            // Generate initial state values\n	            var X = this._X = [\n	                K[0], (K[3] << 16) | (K[2] >>> 16),\n	                K[1], (K[0] << 16) | (K[3] >>> 16),\n	                K[2], (K[1] << 16) | (K[0] >>> 16),\n	                K[3], (K[2] << 16) | (K[1] >>> 16)\n	            ];\n\n	            // Generate initial counter values\n	            var C = this._C = [\n	                (K[2] << 16) | (K[2] >>> 16), (K[0] & 0xffff0000) | (K[1] & 0x0000ffff),\n	                (K[3] << 16) | (K[3] >>> 16), (K[1] & 0xffff0000) | (K[2] & 0x0000ffff),\n	                (K[0] << 16) | (K[0] >>> 16), (K[2] & 0xffff0000) | (K[3] & 0x0000ffff),\n	                (K[1] << 16) | (K[1] >>> 16), (K[3] & 0xffff0000) | (K[0] & 0x0000ffff)\n	            ];\n\n	            // Carry bit\n	            this._b = 0;\n\n	            // Iterate the system four times\n	            for (var i = 0; i < 4; i++) {\n	                nextState.call(this);\n	            }\n\n	            // Modify the counters\n	            for (var i = 0; i < 8; i++) {\n	                C[i] ^= X[(i + 4) & 7];\n	            }\n\n	            // IV setup\n	            if (iv) {\n	                // Shortcuts\n	                var IV = iv.words;\n	                var IV_0 = IV[0];\n	                var IV_1 = IV[1];\n\n	                // Generate four subvectors\n	                var i0 = (((IV_0 << 8) | (IV_0 >>> 24)) & 0x00ff00ff) | (((IV_0 << 24) | (IV_0 >>> 8)) & 0xff00ff00);\n	                var i2 = (((IV_1 << 8) | (IV_1 >>> 24)) & 0x00ff00ff) | (((IV_1 << 24) | (IV_1 >>> 8)) & 0xff00ff00);\n	                var i1 = (i0 >>> 16) | (i2 & 0xffff0000);\n	                var i3 = (i2 << 16)  | (i0 & 0x0000ffff);\n\n	                // Modify counter values\n	                C[0] ^= i0;\n	                C[1] ^= i1;\n	                C[2] ^= i2;\n	                C[3] ^= i3;\n	                C[4] ^= i0;\n	                C[5] ^= i1;\n	                C[6] ^= i2;\n	                C[7] ^= i3;\n\n	                // Iterate the system four times\n	                for (var i = 0; i < 4; i++) {\n	                    nextState.call(this);\n	                }\n	            }\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcut\n	            var X = this._X;\n\n	            // Iterate the system\n	            nextState.call(this);\n\n	            // Generate four keystream words\n	            S[0] = X[0] ^ (X[5] >>> 16) ^ (X[3] << 16);\n	            S[1] = X[2] ^ (X[7] >>> 16) ^ (X[5] << 16);\n	            S[2] = X[4] ^ (X[1] >>> 16) ^ (X[7] << 16);\n	            S[3] = X[6] ^ (X[3] >>> 16) ^ (X[1] << 16);\n\n	            for (var i = 0; i < 4; i++) {\n	                // Swap endian\n	                S[i] = (((S[i] << 8)  | (S[i] >>> 24)) & 0x00ff00ff) |\n	                       (((S[i] << 24) | (S[i] >>> 8))  & 0xff00ff00);\n\n	                // Encrypt\n	                M[offset + i] ^= S[i];\n	            }\n	        },\n\n	        blockSize: 128/32,\n\n	        ivSize: 64/32\n	    });\n\n	    function nextState() {\n	        // Shortcuts\n	        var X = this._X;\n	        var C = this._C;\n\n	        // Save old counter values\n	        for (var i = 0; i < 8; i++) {\n	            C_[i] = C[i];\n	        }\n\n	        // Calculate new counter values\n	        C[0] = (C[0] + 0x4d34d34d + this._b) | 0;\n	        C[1] = (C[1] + 0xd34d34d3 + ((C[0] >>> 0) < (C_[0] >>> 0) ? 1 : 0)) | 0;\n	        C[2] = (C[2] + 0x34d34d34 + ((C[1] >>> 0) < (C_[1] >>> 0) ? 1 : 0)) | 0;\n	        C[3] = (C[3] + 0x4d34d34d + ((C[2] >>> 0) < (C_[2] >>> 0) ? 1 : 0)) | 0;\n	        C[4] = (C[4] + 0xd34d34d3 + ((C[3] >>> 0) < (C_[3] >>> 0) ? 1 : 0)) | 0;\n	        C[5] = (C[5] + 0x34d34d34 + ((C[4] >>> 0) < (C_[4] >>> 0) ? 1 : 0)) | 0;\n	        C[6] = (C[6] + 0x4d34d34d + ((C[5] >>> 0) < (C_[5] >>> 0) ? 1 : 0)) | 0;\n	        C[7] = (C[7] + 0xd34d34d3 + ((C[6] >>> 0) < (C_[6] >>> 0) ? 1 : 0)) | 0;\n	        this._b = (C[7] >>> 0) < (C_[7] >>> 0) ? 1 : 0;\n\n	        // Calculate the g-values\n	        for (var i = 0; i < 8; i++) {\n	            var gx = X[i] + C[i];\n\n	            // Construct high and low argument for squaring\n	            var ga = gx & 0xffff;\n	            var gb = gx >>> 16;\n\n	            // Calculate high and low result of squaring\n	            var gh = ((((ga * ga) >>> 17) + ga * gb) >>> 15) + gb * gb;\n	            var gl = (((gx & 0xffff0000) * gx) | 0) + (((gx & 0x0000ffff) * gx) | 0);\n\n	            // High XOR low\n	            G[i] = gh ^ gl;\n	        }\n\n	        // Calculate new state values\n	        X[0] = (G[0] + ((G[7] << 16) | (G[7] >>> 16)) + ((G[6] << 16) | (G[6] >>> 16))) | 0;\n	        X[1] = (G[1] + ((G[0] << 8)  | (G[0] >>> 24)) + G[7]) | 0;\n	        X[2] = (G[2] + ((G[1] << 16) | (G[1] >>> 16)) + ((G[0] << 16) | (G[0] >>> 16))) | 0;\n	        X[3] = (G[3] + ((G[2] << 8)  | (G[2] >>> 24)) + G[1]) | 0;\n	        X[4] = (G[4] + ((G[3] << 16) | (G[3] >>> 16)) + ((G[2] << 16) | (G[2] >>> 16))) | 0;\n	        X[5] = (G[5] + ((G[4] << 8)  | (G[4] >>> 24)) + G[3]) | 0;\n	        X[6] = (G[6] + ((G[5] << 16) | (G[5] >>> 16)) + ((G[4] << 16) | (G[4] >>> 16))) | 0;\n	        X[7] = (G[7] + ((G[6] << 8)  | (G[6] >>> 24)) + G[5]) | 0;\n	    }\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.Rabbit.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.Rabbit.decrypt(ciphertext, key, cfg);\n	     */\n	    C.Rabbit = StreamCipher._createHelper(Rabbit);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var StreamCipher = C_lib.StreamCipher;\n	    var C_algo = C.algo;\n\n	    // Reusable objects\n	    var S  = [];\n	    var C_ = [];\n	    var G  = [];\n\n	    /**\n	     * Rabbit stream cipher algorithm.\n	     *\n	     * This is a legacy version that neglected to convert the key to little-endian.\n	     * This error doesn't affect the cipher's security,\n	     * but it does affect its compatibility with other implementations.\n	     */\n	    var RabbitLegacy = C_algo.RabbitLegacy = StreamCipher.extend({\n	        _doReset: function () {\n	            // Shortcuts\n	            var K = this._key.words;\n	            var iv = this.cfg.iv;\n\n	            // Generate initial state values\n	            var X = this._X = [\n	                K[0], (K[3] << 16) | (K[2] >>> 16),\n	                K[1], (K[0] << 16) | (K[3] >>> 16),\n	                K[2], (K[1] << 16) | (K[0] >>> 16),\n	                K[3], (K[2] << 16) | (K[1] >>> 16)\n	            ];\n\n	            // Generate initial counter values\n	            var C = this._C = [\n	                (K[2] << 16) | (K[2] >>> 16), (K[0] & 0xffff0000) | (K[1] & 0x0000ffff),\n	                (K[3] << 16) | (K[3] >>> 16), (K[1] & 0xffff0000) | (K[2] & 0x0000ffff),\n	                (K[0] << 16) | (K[0] >>> 16), (K[2] & 0xffff0000) | (K[3] & 0x0000ffff),\n	                (K[1] << 16) | (K[1] >>> 16), (K[3] & 0xffff0000) | (K[0] & 0x0000ffff)\n	            ];\n\n	            // Carry bit\n	            this._b = 0;\n\n	            // Iterate the system four times\n	            for (var i = 0; i < 4; i++) {\n	                nextState.call(this);\n	            }\n\n	            // Modify the counters\n	            for (var i = 0; i < 8; i++) {\n	                C[i] ^= X[(i + 4) & 7];\n	            }\n\n	            // IV setup\n	            if (iv) {\n	                // Shortcuts\n	                var IV = iv.words;\n	                var IV_0 = IV[0];\n	                var IV_1 = IV[1];\n\n	                // Generate four subvectors\n	                var i0 = (((IV_0 << 8) | (IV_0 >>> 24)) & 0x00ff00ff) | (((IV_0 << 24) | (IV_0 >>> 8)) & 0xff00ff00);\n	                var i2 = (((IV_1 << 8) | (IV_1 >>> 24)) & 0x00ff00ff) | (((IV_1 << 24) | (IV_1 >>> 8)) & 0xff00ff00);\n	                var i1 = (i0 >>> 16) | (i2 & 0xffff0000);\n	                var i3 = (i2 << 16)  | (i0 & 0x0000ffff);\n\n	                // Modify counter values\n	                C[0] ^= i0;\n	                C[1] ^= i1;\n	                C[2] ^= i2;\n	                C[3] ^= i3;\n	                C[4] ^= i0;\n	                C[5] ^= i1;\n	                C[6] ^= i2;\n	                C[7] ^= i3;\n\n	                // Iterate the system four times\n	                for (var i = 0; i < 4; i++) {\n	                    nextState.call(this);\n	                }\n	            }\n	        },\n\n	        _doProcessBlock: function (M, offset) {\n	            // Shortcut\n	            var X = this._X;\n\n	            // Iterate the system\n	            nextState.call(this);\n\n	            // Generate four keystream words\n	            S[0] = X[0] ^ (X[5] >>> 16) ^ (X[3] << 16);\n	            S[1] = X[2] ^ (X[7] >>> 16) ^ (X[5] << 16);\n	            S[2] = X[4] ^ (X[1] >>> 16) ^ (X[7] << 16);\n	            S[3] = X[6] ^ (X[3] >>> 16) ^ (X[1] << 16);\n\n	            for (var i = 0; i < 4; i++) {\n	                // Swap endian\n	                S[i] = (((S[i] << 8)  | (S[i] >>> 24)) & 0x00ff00ff) |\n	                       (((S[i] << 24) | (S[i] >>> 8))  & 0xff00ff00);\n\n	                // Encrypt\n	                M[offset + i] ^= S[i];\n	            }\n	        },\n\n	        blockSize: 128/32,\n\n	        ivSize: 64/32\n	    });\n\n	    function nextState() {\n	        // Shortcuts\n	        var X = this._X;\n	        var C = this._C;\n\n	        // Save old counter values\n	        for (var i = 0; i < 8; i++) {\n	            C_[i] = C[i];\n	        }\n\n	        // Calculate new counter values\n	        C[0] = (C[0] + 0x4d34d34d + this._b) | 0;\n	        C[1] = (C[1] + 0xd34d34d3 + ((C[0] >>> 0) < (C_[0] >>> 0) ? 1 : 0)) | 0;\n	        C[2] = (C[2] + 0x34d34d34 + ((C[1] >>> 0) < (C_[1] >>> 0) ? 1 : 0)) | 0;\n	        C[3] = (C[3] + 0x4d34d34d + ((C[2] >>> 0) < (C_[2] >>> 0) ? 1 : 0)) | 0;\n	        C[4] = (C[4] + 0xd34d34d3 + ((C[3] >>> 0) < (C_[3] >>> 0) ? 1 : 0)) | 0;\n	        C[5] = (C[5] + 0x34d34d34 + ((C[4] >>> 0) < (C_[4] >>> 0) ? 1 : 0)) | 0;\n	        C[6] = (C[6] + 0x4d34d34d + ((C[5] >>> 0) < (C_[5] >>> 0) ? 1 : 0)) | 0;\n	        C[7] = (C[7] + 0xd34d34d3 + ((C[6] >>> 0) < (C_[6] >>> 0) ? 1 : 0)) | 0;\n	        this._b = (C[7] >>> 0) < (C_[7] >>> 0) ? 1 : 0;\n\n	        // Calculate the g-values\n	        for (var i = 0; i < 8; i++) {\n	            var gx = X[i] + C[i];\n\n	            // Construct high and low argument for squaring\n	            var ga = gx & 0xffff;\n	            var gb = gx >>> 16;\n\n	            // Calculate high and low result of squaring\n	            var gh = ((((ga * ga) >>> 17) + ga * gb) >>> 15) + gb * gb;\n	            var gl = (((gx & 0xffff0000) * gx) | 0) + (((gx & 0x0000ffff) * gx) | 0);\n\n	            // High XOR low\n	            G[i] = gh ^ gl;\n	        }\n\n	        // Calculate new state values\n	        X[0] = (G[0] + ((G[7] << 16) | (G[7] >>> 16)) + ((G[6] << 16) | (G[6] >>> 16))) | 0;\n	        X[1] = (G[1] + ((G[0] << 8)  | (G[0] >>> 24)) + G[7]) | 0;\n	        X[2] = (G[2] + ((G[1] << 16) | (G[1] >>> 16)) + ((G[0] << 16) | (G[0] >>> 16))) | 0;\n	        X[3] = (G[3] + ((G[2] << 8)  | (G[2] >>> 24)) + G[1]) | 0;\n	        X[4] = (G[4] + ((G[3] << 16) | (G[3] >>> 16)) + ((G[2] << 16) | (G[2] >>> 16))) | 0;\n	        X[5] = (G[5] + ((G[4] << 8)  | (G[4] >>> 24)) + G[3]) | 0;\n	        X[6] = (G[6] + ((G[5] << 16) | (G[5] >>> 16)) + ((G[4] << 16) | (G[4] >>> 16))) | 0;\n	        X[7] = (G[7] + ((G[6] << 8)  | (G[6] >>> 24)) + G[5]) | 0;\n	    }\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.RabbitLegacy.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.RabbitLegacy.decrypt(ciphertext, key, cfg);\n	     */\n	    C.RabbitLegacy = StreamCipher._createHelper(RabbitLegacy);\n	}());\n\n\n	(function () {\n	    // Shortcuts\n	    var C = CryptoJS;\n	    var C_lib = C.lib;\n	    var BlockCipher = C_lib.BlockCipher;\n	    var C_algo = C.algo;\n\n	    const N = 16;\n\n	    //Origin pbox and sbox, derived from PI\n	    const ORIG_P = [\n	        0x243F6A88, 0x85A308D3, 0x13198A2E, 0x03707344,\n	        0xA4093822, 0x299F31D0, 0x082EFA98, 0xEC4E6C89,\n	        0x452821E6, 0x38D01377, 0xBE5466CF, 0x34E90C6C,\n	        0xC0AC29B7, 0xC97C50DD, 0x3F84D5B5, 0xB5470917,\n	        0x9216D5D9, 0x8979FB1B\n	    ];\n\n	    const ORIG_S = [\n	        [   0xD1310BA6, 0x98DFB5AC, 0x2FFD72DB, 0xD01ADFB7,\n	            0xB8E1AFED, 0x6A267E96, 0xBA7C9045, 0xF12C7F99,\n	            0x24A19947, 0xB3916CF7, 0x0801F2E2, 0x858EFC16,\n	            0x636920D8, 0x71574E69, 0xA458FEA3, 0xF4933D7E,\n	            0x0D95748F, 0x728EB658, 0x718BCD58, 0x82154AEE,\n	            0x7B54A41D, 0xC25A59B5, 0x9C30D539, 0x2AF26013,\n	            0xC5D1B023, 0x286085F0, 0xCA417918, 0xB8DB38EF,\n	            0x8E79DCB0, 0x603A180E, 0x6C9E0E8B, 0xB01E8A3E,\n	            0xD71577C1, 0xBD314B27, 0x78AF2FDA, 0x55605C60,\n	            0xE65525F3, 0xAA55AB94, 0x57489862, 0x63E81440,\n	            0x55CA396A, 0x2AAB10B6, 0xB4CC5C34, 0x1141E8CE,\n	            0xA15486AF, 0x7C72E993, 0xB3EE1411, 0x636FBC2A,\n	            0x2BA9C55D, 0x741831F6, 0xCE5C3E16, 0x9B87931E,\n	            0xAFD6BA33, 0x6C24CF5C, 0x7A325381, 0x28958677,\n	            0x3B8F4898, 0x6B4BB9AF, 0xC4BFE81B, 0x66282193,\n	            0x61D809CC, 0xFB21A991, 0x487CAC60, 0x5DEC8032,\n	            0xEF845D5D, 0xE98575B1, 0xDC262302, 0xEB651B88,\n	            0x23893E81, 0xD396ACC5, 0x0F6D6FF3, 0x83F44239,\n	            0x2E0B4482, 0xA4842004, 0x69C8F04A, 0x9E1F9B5E,\n	            0x21C66842, 0xF6E96C9A, 0x670C9C61, 0xABD388F0,\n	            0x6A51A0D2, 0xD8542F68, 0x960FA728, 0xAB5133A3,\n	            0x6EEF0B6C, 0x137A3BE4, 0xBA3BF050, 0x7EFB2A98,\n	            0xA1F1651D, 0x39AF0176, 0x66CA593E, 0x82430E88,\n	            0x8CEE8619, 0x456F9FB4, 0x7D84A5C3, 0x3B8B5EBE,\n	            0xE06F75D8, 0x85C12073, 0x401A449F, 0x56C16AA6,\n	            0x4ED3AA62, 0x363F7706, 0x1BFEDF72, 0x429B023D,\n	            0x37D0D724, 0xD00A1248, 0xDB0FEAD3, 0x49F1C09B,\n	            0x075372C9, 0x80991B7B, 0x25D479D8, 0xF6E8DEF7,\n	            0xE3FE501A, 0xB6794C3B, 0x976CE0BD, 0x04C006BA,\n	            0xC1A94FB6, 0x409F60C4, 0x5E5C9EC2, 0x196A2463,\n	            0x68FB6FAF, 0x3E6C53B5, 0x1339B2EB, 0x3B52EC6F,\n	            0x6DFC511F, 0x9B30952C, 0xCC814544, 0xAF5EBD09,\n	            0xBEE3D004, 0xDE334AFD, 0x660F2807, 0x192E4BB3,\n	            0xC0CBA857, 0x45C8740F, 0xD20B5F39, 0xB9D3FBDB,\n	            0x5579C0BD, 0x1A60320A, 0xD6A100C6, 0x402C7279,\n	            0x679F25FE, 0xFB1FA3CC, 0x8EA5E9F8, 0xDB3222F8,\n	            0x3C7516DF, 0xFD616B15, 0x2F501EC8, 0xAD0552AB,\n	            0x323DB5FA, 0xFD238760, 0x53317B48, 0x3E00DF82,\n	            0x9E5C57BB, 0xCA6F8CA0, 0x1A87562E, 0xDF1769DB,\n	            0xD542A8F6, 0x287EFFC3, 0xAC6732C6, 0x8C4F5573,\n	            0x695B27B0, 0xBBCA58C8, 0xE1FFA35D, 0xB8F011A0,\n	            0x10FA3D98, 0xFD2183B8, 0x4AFCB56C, 0x2DD1D35B,\n	            0x9A53E479, 0xB6F84565, 0xD28E49BC, 0x4BFB9790,\n	            0xE1DDF2DA, 0xA4CB7E33, 0x62FB1341, 0xCEE4C6E8,\n	            0xEF20CADA, 0x36774C01, 0xD07E9EFE, 0x2BF11FB4,\n	            0x95DBDA4D, 0xAE909198, 0xEAAD8E71, 0x6B93D5A0,\n	            0xD08ED1D0, 0xAFC725E0, 0x8E3C5B2F, 0x8E7594B7,\n	            0x8FF6E2FB, 0xF2122B64, 0x8888B812, 0x900DF01C,\n	            0x4FAD5EA0, 0x688FC31C, 0xD1CFF191, 0xB3A8C1AD,\n	            0x2F2F2218, 0xBE0E1777, 0xEA752DFE, 0x8B021FA1,\n	            0xE5A0CC0F, 0xB56F74E8, 0x18ACF3D6, 0xCE89E299,\n	            0xB4A84FE0, 0xFD13E0B7, 0x7CC43B81, 0xD2ADA8D9,\n	            0x165FA266, 0x80957705, 0x93CC7314, 0x211A1477,\n	            0xE6AD2065, 0x77B5FA86, 0xC75442F5, 0xFB9D35CF,\n	            0xEBCDAF0C, 0x7B3E89A0, 0xD6411BD3, 0xAE1E7E49,\n	            0x00250E2D, 0x2071B35E, 0x226800BB, 0x57B8E0AF,\n	            0x2464369B, 0xF009B91E, 0x5563911D, 0x59DFA6AA,\n	            0x78C14389, 0xD95A537F, 0x207D5BA2, 0x02E5B9C5,\n	            0x83260376, 0x6295CFA9, 0x11C81968, 0x4E734A41,\n	            0xB3472DCA, 0x7B14A94A, 0x1B510052, 0x9A532915,\n	            0xD60F573F, 0xBC9BC6E4, 0x2B60A476, 0x81E67400,\n	            0x08BA6FB5, 0x571BE91F, 0xF296EC6B, 0x2A0DD915,\n	            0xB6636521, 0xE7B9F9B6, 0xFF34052E, 0xC5855664,\n	            0x53B02D5D, 0xA99F8FA1, 0x08BA4799, 0x6E85076A   ],\n	        [   0x4B7A70E9, 0xB5B32944, 0xDB75092E, 0xC4192623,\n	            0xAD6EA6B0, 0x49A7DF7D, 0x9CEE60B8, 0x8FEDB266,\n	            0xECAA8C71, 0x699A17FF, 0x5664526C, 0xC2B19EE1,\n	            0x193602A5, 0x75094C29, 0xA0591340, 0xE4183A3E,\n	            0x3F54989A, 0x5B429D65, 0x6B8FE4D6, 0x99F73FD6,\n	            0xA1D29C07, 0xEFE830F5, 0x4D2D38E6, 0xF0255DC1,\n	            0x4CDD2086, 0x8470EB26, 0x6382E9C6, 0x021ECC5E,\n	            0x09686B3F, 0x3EBAEFC9, 0x3C971814, 0x6B6A70A1,\n	            0x687F3584, 0x52A0E286, 0xB79C5305, 0xAA500737,\n	            0x3E07841C, 0x7FDEAE5C, 0x8E7D44EC, 0x5716F2B8,\n	            0xB03ADA37, 0xF0500C0D, 0xF01C1F04, 0x0200B3FF,\n	            0xAE0CF51A, 0x3CB574B2, 0x25837A58, 0xDC0921BD,\n	            0xD19113F9, 0x7CA92FF6, 0x94324773, 0x22F54701,\n	            0x3AE5E581, 0x37C2DADC, 0xC8B57634, 0x9AF3DDA7,\n	            0xA9446146, 0x0FD0030E, 0xECC8C73E, 0xA4751E41,\n	            0xE238CD99, 0x3BEA0E2F, 0x3280BBA1, 0x183EB331,\n	            0x4E548B38, 0x4F6DB908, 0x6F420D03, 0xF60A04BF,\n	            0x2CB81290, 0x24977C79, 0x5679B072, 0xBCAF89AF,\n	            0xDE9A771F, 0xD9930810, 0xB38BAE12, 0xDCCF3F2E,\n	            0x5512721F, 0x2E6B7124, 0x501ADDE6, 0x9F84CD87,\n	            0x7A584718, 0x7408DA17, 0xBC9F9ABC, 0xE94B7D8C,\n	            0xEC7AEC3A, 0xDB851DFA, 0x63094366, 0xC464C3D2,\n	            0xEF1C1847, 0x3215D908, 0xDD433B37, 0x24C2BA16,\n	            0x12A14D43, 0x2A65C451, 0x50940002, 0x133AE4DD,\n	            0x71DFF89E, 0x10314E55, 0x81AC77D6, 0x5F11199B,\n	            0x043556F1, 0xD7A3C76B, 0x3C11183B, 0x5924A509,\n	            0xF28FE6ED, 0x97F1FBFA, 0x9EBABF2C, 0x1E153C6E,\n	            0x86E34570, 0xEAE96FB1, 0x860E5E0A, 0x5A3E2AB3,\n	            0x771FE71C, 0x4E3D06FA, 0x2965DCB9, 0x99E71D0F,\n	            0x803E89D6, 0x5266C825, 0x2E4CC978, 0x9C10B36A,\n	            0xC6150EBA, 0x94E2EA78, 0xA5FC3C53, 0x1E0A2DF4,\n	            0xF2F74EA7, 0x361D2B3D, 0x1939260F, 0x19C27960,\n	            0x5223A708, 0xF71312B6, 0xEBADFE6E, 0xEAC31F66,\n	            0xE3BC4595, 0xA67BC883, 0xB17F37D1, 0x018CFF28,\n	            0xC332DDEF, 0xBE6C5AA5, 0x65582185, 0x68AB9802,\n	            0xEECEA50F, 0xDB2F953B, 0x2AEF7DAD, 0x5B6E2F84,\n	            0x1521B628, 0x29076170, 0xECDD4775, 0x619F1510,\n	            0x13CCA830, 0xEB61BD96, 0x0334FE1E, 0xAA0363CF,\n	            0xB5735C90, 0x4C70A239, 0xD59E9E0B, 0xCBAADE14,\n	            0xEECC86BC, 0x60622CA7, 0x9CAB5CAB, 0xB2F3846E,\n	            0x648B1EAF, 0x19BDF0CA, 0xA02369B9, 0x655ABB50,\n	            0x40685A32, 0x3C2AB4B3, 0x319EE9D5, 0xC021B8F7,\n	            0x9B540B19, 0x875FA099, 0x95F7997E, 0x623D7DA8,\n	            0xF837889A, 0x97E32D77, 0x11ED935F, 0x16681281,\n	            0x0E358829, 0xC7E61FD6, 0x96DEDFA1, 0x7858BA99,\n	            0x57F584A5, 0x1B227263, 0x9B83C3FF, 0x1AC24696,\n	            0xCDB30AEB, 0x532E3054, 0x8FD948E4, 0x6DBC3128,\n	            0x58EBF2EF, 0x34C6FFEA, 0xFE28ED61, 0xEE7C3C73,\n	            0x5D4A14D9, 0xE864B7E3, 0x42105D14, 0x203E13E0,\n	            0x45EEE2B6, 0xA3AAABEA, 0xDB6C4F15, 0xFACB4FD0,\n	            0xC742F442, 0xEF6ABBB5, 0x654F3B1D, 0x41CD2105,\n	            0xD81E799E, 0x86854DC7, 0xE44B476A, 0x3D816250,\n	            0xCF62A1F2, 0x5B8D2646, 0xFC8883A0, 0xC1C7B6A3,\n	            0x7F1524C3, 0x69CB7492, 0x47848A0B, 0x5692B285,\n	            0x095BBF00, 0xAD19489D, 0x1462B174, 0x23820E00,\n	            0x58428D2A, 0x0C55F5EA, 0x1DADF43E, 0x233F7061,\n	            0x3372F092, 0x8D937E41, 0xD65FECF1, 0x6C223BDB,\n	            0x7CDE3759, 0xCBEE7460, 0x4085F2A7, 0xCE77326E,\n	            0xA6078084, 0x19F8509E, 0xE8EFD855, 0x61D99735,\n	            0xA969A7AA, 0xC50C06C2, 0x5A04ABFC, 0x800BCADC,\n	            0x9E447A2E, 0xC3453484, 0xFDD56705, 0x0E1E9EC9,\n	            0xDB73DBD3, 0x105588CD, 0x675FDA79, 0xE3674340,\n	            0xC5C43465, 0x713E38D8, 0x3D28F89E, 0xF16DFF20,\n	            0x153E21E7, 0x8FB03D4A, 0xE6E39F2B, 0xDB83ADF7   ],\n	        [   0xE93D5A68, 0x948140F7, 0xF64C261C, 0x94692934,\n	            0x411520F7, 0x7602D4F7, 0xBCF46B2E, 0xD4A20068,\n	            0xD4082471, 0x3320F46A, 0x43B7D4B7, 0x500061AF,\n	            0x1E39F62E, 0x97244546, 0x14214F74, 0xBF8B8840,\n	            0x4D95FC1D, 0x96B591AF, 0x70F4DDD3, 0x66A02F45,\n	            0xBFBC09EC, 0x03BD9785, 0x7FAC6DD0, 0x31CB8504,\n	            0x96EB27B3, 0x55FD3941, 0xDA2547E6, 0xABCA0A9A,\n	            0x28507825, 0x530429F4, 0x0A2C86DA, 0xE9B66DFB,\n	            0x68DC1462, 0xD7486900, 0x680EC0A4, 0x27A18DEE,\n	            0x4F3FFEA2, 0xE887AD8C, 0xB58CE006, 0x7AF4D6B6,\n	            0xAACE1E7C, 0xD3375FEC, 0xCE78A399, 0x406B2A42,\n	            0x20FE9E35, 0xD9F385B9, 0xEE39D7AB, 0x3B124E8B,\n	            0x1DC9FAF7, 0x4B6D1856, 0x26A36631, 0xEAE397B2,\n	            0x3A6EFA74, 0xDD5B4332, 0x6841E7F7, 0xCA7820FB,\n	            0xFB0AF54E, 0xD8FEB397, 0x454056AC, 0xBA489527,\n	            0x55533A3A, 0x20838D87, 0xFE6BA9B7, 0xD096954B,\n	            0x55A867BC, 0xA1159A58, 0xCCA92963, 0x99E1DB33,\n	            0xA62A4A56, 0x3F3125F9, 0x5EF47E1C, 0x9029317C,\n	            0xFDF8E802, 0x04272F70, 0x80BB155C, 0x05282CE3,\n	            0x95C11548, 0xE4C66D22, 0x48C1133F, 0xC70F86DC,\n	            0x07F9C9EE, 0x41041F0F, 0x404779A4, 0x5D886E17,\n	            0x325F51EB, 0xD59BC0D1, 0xF2BCC18F, 0x41113564,\n	            0x257B7834, 0x602A9C60, 0xDFF8E8A3, 0x1F636C1B,\n	            0x0E12B4C2, 0x02E1329E, 0xAF664FD1, 0xCAD18115,\n	            0x6B2395E0, 0x333E92E1, 0x3B240B62, 0xEEBEB922,\n	            0x85B2A20E, 0xE6BA0D99, 0xDE720C8C, 0x2DA2F728,\n	            0xD0127845, 0x95B794FD, 0x647D0862, 0xE7CCF5F0,\n	            0x5449A36F, 0x877D48FA, 0xC39DFD27, 0xF33E8D1E,\n	            0x0A476341, 0x992EFF74, 0x3A6F6EAB, 0xF4F8FD37,\n	            0xA812DC60, 0xA1EBDDF8, 0x991BE14C, 0xDB6E6B0D,\n	            0xC67B5510, 0x6D672C37, 0x2765D43B, 0xDCD0E804,\n	            0xF1290DC7, 0xCC00FFA3, 0xB5390F92, 0x690FED0B,\n	            0x667B9FFB, 0xCEDB7D9C, 0xA091CF0B, 0xD9155EA3,\n	            0xBB132F88, 0x515BAD24, 0x7B9479BF, 0x763BD6EB,\n	            0x37392EB3, 0xCC115979, 0x8026E297, 0xF42E312D,\n	            0x6842ADA7, 0xC66A2B3B, 0x12754CCC, 0x782EF11C,\n	            0x6A124237, 0xB79251E7, 0x06A1BBE6, 0x4BFB6350,\n	            0x1A6B1018, 0x11CAEDFA, 0x3D25BDD8, 0xE2E1C3C9,\n	            0x44421659, 0x0A121386, 0xD90CEC6E, 0xD5ABEA2A,\n	            0x64AF674E, 0xDA86A85F, 0xBEBFE988, 0x64E4C3FE,\n	            0x9DBC8057, 0xF0F7C086, 0x60787BF8, 0x6003604D,\n	            0xD1FD8346, 0xF6381FB0, 0x7745AE04, 0xD736FCCC,\n	            0x83426B33, 0xF01EAB71, 0xB0804187, 0x3C005E5F,\n	            0x77A057BE, 0xBDE8AE24, 0x55464299, 0xBF582E61,\n	            0x4E58F48F, 0xF2DDFDA2, 0xF474EF38, 0x8789BDC2,\n	            0x5366F9C3, 0xC8B38E74, 0xB475F255, 0x46FCD9B9,\n	            0x7AEB2661, 0x8B1DDF84, 0x846A0E79, 0x915F95E2,\n	            0x466E598E, 0x20B45770, 0x8CD55591, 0xC902DE4C,\n	            0xB90BACE1, 0xBB8205D0, 0x11A86248, 0x7574A99E,\n	            0xB77F19B6, 0xE0A9DC09, 0x662D09A1, 0xC4324633,\n	            0xE85A1F02, 0x09F0BE8C, 0x4A99A025, 0x1D6EFE10,\n	            0x1AB93D1D, 0x0BA5A4DF, 0xA186F20F, 0x2868F169,\n	            0xDCB7DA83, 0x573906FE, 0xA1E2CE9B, 0x4FCD7F52,\n	            0x50115E01, 0xA70683FA, 0xA002B5C4, 0x0DE6D027,\n	            0x9AF88C27, 0x773F8641, 0xC3604C06, 0x61A806B5,\n	            0xF0177A28, 0xC0F586E0, 0x006058AA, 0x30DC7D62,\n	            0x11E69ED7, 0x2338EA63, 0x53C2DD94, 0xC2C21634,\n	            0xBBCBEE56, 0x90BCB6DE, 0xEBFC7DA1, 0xCE591D76,\n	            0x6F05E409, 0x4B7C0188, 0x39720A3D, 0x7C927C24,\n	            0x86E3725F, 0x724D9DB9, 0x1AC15BB4, 0xD39EB8FC,\n	            0xED545578, 0x08FCA5B5, 0xD83D7CD3, 0x4DAD0FC4,\n	            0x1E50EF5E, 0xB161E6F8, 0xA28514D9, 0x6C51133C,\n	            0x6FD5C7E7, 0x56E14EC4, 0x362ABFCE, 0xDDC6C837,\n	            0xD79A3234, 0x92638212, 0x670EFA8E, 0x406000E0  ],\n	        [   0x3A39CE37, 0xD3FAF5CF, 0xABC27737, 0x5AC52D1B,\n	            0x5CB0679E, 0x4FA33742, 0xD3822740, 0x99BC9BBE,\n	            0xD5118E9D, 0xBF0F7315, 0xD62D1C7E, 0xC700C47B,\n	            0xB78C1B6B, 0x21A19045, 0xB26EB1BE, 0x6A366EB4,\n	            0x5748AB2F, 0xBC946E79, 0xC6A376D2, 0x6549C2C8,\n	            0x530FF8EE, 0x468DDE7D, 0xD5730A1D, 0x4CD04DC6,\n	            0x2939BBDB, 0xA9BA4650, 0xAC9526E8, 0xBE5EE304,\n	            0xA1FAD5F0, 0x6A2D519A, 0x63EF8CE2, 0x9A86EE22,\n	            0xC089C2B8, 0x43242EF6, 0xA51E03AA, 0x9CF2D0A4,\n	            0x83C061BA, 0x9BE96A4D, 0x8FE51550, 0xBA645BD6,\n	            0x2826A2F9, 0xA73A3AE1, 0x4BA99586, 0xEF5562E9,\n	            0xC72FEFD3, 0xF752F7DA, 0x3F046F69, 0x77FA0A59,\n	            0x80E4A915, 0x87B08601, 0x9B09E6AD, 0x3B3EE593,\n	            0xE990FD5A, 0x9E34D797, 0x2CF0B7D9, 0x022B8B51,\n	            0x96D5AC3A, 0x017DA67D, 0xD1CF3ED6, 0x7C7D2D28,\n	            0x1F9F25CF, 0xADF2B89B, 0x5AD6B472, 0x5A88F54C,\n	            0xE029AC71, 0xE019A5E6, 0x47B0ACFD, 0xED93FA9B,\n	            0xE8D3C48D, 0x283B57CC, 0xF8D56629, 0x79132E28,\n	            0x785F0191, 0xED756055, 0xF7960E44, 0xE3D35E8C,\n	            0x15056DD4, 0x88F46DBA, 0x03A16125, 0x0564F0BD,\n	            0xC3EB9E15, 0x3C9057A2, 0x97271AEC, 0xA93A072A,\n	            0x1B3F6D9B, 0x1E6321F5, 0xF59C66FB, 0x26DCF319,\n	            0x7533D928, 0xB155FDF5, 0x03563482, 0x8ABA3CBB,\n	            0x28517711, 0xC20AD9F8, 0xABCC5167, 0xCCAD925F,\n	            0x4DE81751, 0x3830DC8E, 0x379D5862, 0x9320F991,\n	            0xEA7A90C2, 0xFB3E7BCE, 0x5121CE64, 0x774FBE32,\n	            0xA8B6E37E, 0xC3293D46, 0x48DE5369, 0x6413E680,\n	            0xA2AE0810, 0xDD6DB224, 0x69852DFD, 0x09072166,\n	            0xB39A460A, 0x6445C0DD, 0x586CDECF, 0x1C20C8AE,\n	            0x5BBEF7DD, 0x1B588D40, 0xCCD2017F, 0x6BB4E3BB,\n	            0xDDA26A7E, 0x3A59FF45, 0x3E350A44, 0xBCB4CDD5,\n	            0x72EACEA8, 0xFA6484BB, 0x8D6612AE, 0xBF3C6F47,\n	            0xD29BE463, 0x542F5D9E, 0xAEC2771B, 0xF64E6370,\n	            0x740E0D8D, 0xE75B1357, 0xF8721671, 0xAF537D5D,\n	            0x4040CB08, 0x4EB4E2CC, 0x34D2466A, 0x0115AF84,\n	            0xE1B00428, 0x95983A1D, 0x06B89FB4, 0xCE6EA048,\n	            0x6F3F3B82, 0x3520AB82, 0x011A1D4B, 0x277227F8,\n	            0x611560B1, 0xE7933FDC, 0xBB3A792B, 0x344525BD,\n	            0xA08839E1, 0x51CE794B, 0x2F32C9B7, 0xA01FBAC9,\n	            0xE01CC87E, 0xBCC7D1F6, 0xCF0111C3, 0xA1E8AAC7,\n	            0x1A908749, 0xD44FBD9A, 0xD0DADECB, 0xD50ADA38,\n	            0x0339C32A, 0xC6913667, 0x8DF9317C, 0xE0B12B4F,\n	            0xF79E59B7, 0x43F5BB3A, 0xF2D519FF, 0x27D9459C,\n	            0xBF97222C, 0x15E6FC2A, 0x0F91FC71, 0x9B941525,\n	            0xFAE59361, 0xCEB69CEB, 0xC2A86459, 0x12BAA8D1,\n	            0xB6C1075E, 0xE3056A0C, 0x10D25065, 0xCB03A442,\n	            0xE0EC6E0E, 0x1698DB3B, 0x4C98A0BE, 0x3278E964,\n	            0x9F1F9532, 0xE0D392DF, 0xD3A0342B, 0x8971F21E,\n	            0x1B0A7441, 0x4BA3348C, 0xC5BE7120, 0xC37632D8,\n	            0xDF359F8D, 0x9B992F2E, 0xE60B6F47, 0x0FE3F11D,\n	            0xE54CDA54, 0x1EDAD891, 0xCE6279CF, 0xCD3E7E6F,\n	            0x1618B166, 0xFD2C1D05, 0x848FD2C5, 0xF6FB2299,\n	            0xF523F357, 0xA6327623, 0x93A83531, 0x56CCCD02,\n	            0xACF08162, 0x5A75EBB5, 0x6E163697, 0x88D273CC,\n	            0xDE966292, 0x81B949D0, 0x4C50901B, 0x71C65614,\n	            0xE6C6C7BD, 0x327A140A, 0x45E1D006, 0xC3F27B9A,\n	            0xC9AA53FD, 0x62A80F00, 0xBB25BFE2, 0x35BDD2F6,\n	            0x71126905, 0xB2040222, 0xB6CBCF7C, 0xCD769C2B,\n	            0x53113EC0, 0x1640E3D3, 0x38ABBD60, 0x2547ADF0,\n	            0xBA38209C, 0xF746CE76, 0x77AFA1C5, 0x20756060,\n	            0x85CBFE4E, 0x8AE88DD8, 0x7AAAF9B0, 0x4CF9AA7E,\n	            0x1948C25C, 0x02FB8A8C, 0x01C36AE4, 0xD6EBE1F9,\n	            0x90D4F869, 0xA65CDEA0, 0x3F09252D, 0xC208E69F,\n	            0xB74E6132, 0xCE77E25B, 0x578FDFE3, 0x3AC372E6  ]\n	    ];\n\n	    var BLOWFISH_CTX = {\n	        pbox: [],\n	        sbox: []\n	    }\n\n	    function F(ctx, x){\n	        let a = (x >> 24) & 0xFF;\n	        let b = (x >> 16) & 0xFF;\n	        let c = (x >> 8) & 0xFF;\n	        let d = x & 0xFF;\n\n	        let y = ctx.sbox[0][a] + ctx.sbox[1][b];\n	        y = y ^ ctx.sbox[2][c];\n	        y = y + ctx.sbox[3][d];\n\n	        return y;\n	    }\n\n	    function BlowFish_Encrypt(ctx, left, right){\n	        let Xl = left;\n	        let Xr = right;\n	        let temp;\n\n	        for(let i = 0; i < N; ++i){\n	            Xl = Xl ^ ctx.pbox[i];\n	            Xr = F(ctx, Xl) ^ Xr;\n\n	            temp = Xl;\n	            Xl = Xr;\n	            Xr = temp;\n	        }\n\n	        temp = Xl;\n	        Xl = Xr;\n	        Xr = temp;\n\n	        Xr = Xr ^ ctx.pbox[N];\n	        Xl = Xl ^ ctx.pbox[N + 1];\n\n	        return {left: Xl, right: Xr};\n	    }\n\n	    function BlowFish_Decrypt(ctx, left, right){\n	        let Xl = left;\n	        let Xr = right;\n	        let temp;\n\n	        for(let i = N + 1; i > 1; --i){\n	            Xl = Xl ^ ctx.pbox[i];\n	            Xr = F(ctx, Xl) ^ Xr;\n\n	            temp = Xl;\n	            Xl = Xr;\n	            Xr = temp;\n	        }\n\n	        temp = Xl;\n	        Xl = Xr;\n	        Xr = temp;\n\n	        Xr = Xr ^ ctx.pbox[1];\n	        Xl = Xl ^ ctx.pbox[0];\n\n	        return {left: Xl, right: Xr};\n	    }\n\n	    /**\n	     * Initialization ctx's pbox and sbox.\n	     *\n	     * @param {Object} ctx The object has pbox and sbox.\n	     * @param {Array} key An array of 32-bit words.\n	     * @param {int} keysize The length of the key.\n	     *\n	     * @example\n	     *\n	     *     BlowFishInit(BLOWFISH_CTX, key, 128/32);\n	     */\n	    function BlowFishInit(ctx, key, keysize)\n	    {\n	        for(let Row = 0; Row < 4; Row++)\n	        {\n	            ctx.sbox[Row] = [];\n	            for(let Col = 0; Col < 256; Col++)\n	            {\n	                ctx.sbox[Row][Col] = ORIG_S[Row][Col];\n	            }\n	        }\n\n	        let keyIndex = 0;\n	        for(let index = 0; index < N + 2; index++)\n	        {\n	            ctx.pbox[index] = ORIG_P[index] ^ key[keyIndex];\n	            keyIndex++;\n	            if(keyIndex >= keysize)\n	            {\n	                keyIndex = 0;\n	            }\n	        }\n\n	        let Data1 = 0;\n	        let Data2 = 0;\n	        let res = 0;\n	        for(let i = 0; i < N + 2; i += 2)\n	        {\n	            res = BlowFish_Encrypt(ctx, Data1, Data2);\n	            Data1 = res.left;\n	            Data2 = res.right;\n	            ctx.pbox[i] = Data1;\n	            ctx.pbox[i + 1] = Data2;\n	        }\n\n	        for(let i = 0; i < 4; i++)\n	        {\n	            for(let j = 0; j < 256; j += 2)\n	            {\n	                res = BlowFish_Encrypt(ctx, Data1, Data2);\n	                Data1 = res.left;\n	                Data2 = res.right;\n	                ctx.sbox[i][j] = Data1;\n	                ctx.sbox[i][j + 1] = Data2;\n	            }\n	        }\n\n	        return true;\n	    }\n\n	    /**\n	     * Blowfish block cipher algorithm.\n	     */\n	    var Blowfish = C_algo.Blowfish = BlockCipher.extend({\n	        _doReset: function () {\n	            // Skip reset of nRounds has been set before and key did not change\n	            if (this._keyPriorReset === this._key) {\n	                return;\n	            }\n\n	            // Shortcuts\n	            var key = this._keyPriorReset = this._key;\n	            var keyWords = key.words;\n	            var keySize = key.sigBytes / 4;\n\n	            //Initialization pbox and sbox\n	            BlowFishInit(BLOWFISH_CTX, keyWords, keySize);\n	        },\n\n	        encryptBlock: function (M, offset) {\n	            var res = BlowFish_Encrypt(BLOWFISH_CTX, M[offset], M[offset + 1]);\n	            M[offset] = res.left;\n	            M[offset + 1] = res.right;\n	        },\n\n	        decryptBlock: function (M, offset) {\n	            var res = BlowFish_Decrypt(BLOWFISH_CTX, M[offset], M[offset + 1]);\n	            M[offset] = res.left;\n	            M[offset + 1] = res.right;\n	        },\n\n	        blockSize: 64/32,\n\n	        keySize: 128/32,\n\n	        ivSize: 64/32\n	    });\n\n	    /**\n	     * Shortcut functions to the cipher's object interface.\n	     *\n	     * @example\n	     *\n	     *     var ciphertext = CryptoJS.Blowfish.encrypt(message, key, cfg);\n	     *     var plaintext  = CryptoJS.Blowfish.decrypt(ciphertext, key, cfg);\n	     */\n	    C.Blowfish = BlockCipher._createHelper(Blowfish);\n	}());\n\n\n	return CryptoJS;\n\n}));";
      if (!re) throw new Error("CryptoJS source asset unavailable");
      var k = c.evalCode(re + "\nif (!globalThis.CryptoJS) throw new Error('CryptoJS global unavailable'); true;", "crypto-js.js");
      c.unwrapResult(k).dispose();
      var g = c.evalCode('["__native_fetch","__native_cancel","__native_log","__parse_url","__get_scraper_id","__get_scraper_settings","__get_tmdb_api_key","__cheerio_load","__cheerio_select","__cheerio_find","__cheerio_text","__cheerio_html","__cheerio_innerHtml","__cheerio_attr","__cheerio_next","__cheerio_prev","__cheerio_parent","__cheerio_children","__cheerio_filter","__cheerio_eq"].forEach(function(name){ try { globalThis[name] = undefined; delete globalThis[name]; } catch (_) {} }); true;', "nuvio-plugin-bridge-cleanup.js");
      c.unwrapResult(g).dispose();
      var I = String(a.code || ""),
        te = typeof TextEncoder == "function" ? new TextEncoder().encode(I).byteLength : unescape(encodeURIComponent(I)).length;
      if (te > Number(i.maxCodeBytes || 1024 * 1024)) throw new Error("Plugin code exceeds quota");
      var ee = c.evalCode("var module = { exports: {} }; var exports = module.exports; (function(){\n" + I + "\n})(); globalThis.__pluginModuleExports = module.exports;", String(a.filename || "plugin.js"));
      c.unwrapResult(ee).dispose();
      var q;
      function nt() {
        if (q) {
          try {
            q.dispose();
          } catch (S) {}
          q = null;
        }
      }
      try {
        var Cn = c.evalCode("(async function(){ try { var exported = globalThis.__pluginModuleExports || {}; var fn = exported.getStreams || globalThis.getStreams; if (typeof fn !== 'function') return JSON.stringify([]); var args = " + JSON.stringify(a.args || {}) + "; var value = await fn(args.tmdbId, args.mediaType, args.season, args.episode); return JSON.stringify(Array.isArray(value) ? value : []); } catch (_) { try { console.error('getStreams failed:', _); } catch (__) {} return JSON.stringify([]); } })()", "nuvio-plugin-call.js");
        q = c.unwrapResult(Cn);
      } catch (S) {
        s.cleanup(), ie = null, Te({
          type: "pluginLog",
          level: "error",
          message: "getStreams invocation failed: " + String((S == null ? void 0 : S.message) || S)
        }), Te({
          type: "result",
          results: []
        });
        return;
      }
      var Lt,
        Ne = null;
      try {
        Lt = yield A1(c, E, q, a.timeoutMs);
      } catch (S) {
        Ne = S;
      }
      if (Ne) {
        nt(), s.cleanup(), ie = null, Te({
          type: "pluginLog",
          level: "error",
          message: "getStreams promise failed: " + String((Ne == null ? void 0 : Ne.message) || Ne)
        }), Te({
          type: "result",
          results: []
        });
        return;
      }
      nt();
      var Mt = c.unwrapResult(Lt),
        lt = c.dump(Mt);
      Mt.dispose();
      var ke;
      try {
        ke = JSON.parse(typeof lt == "string" ? lt : String(lt || "[]"));
      } catch (S) {
        Te({
          type: "pluginLog",
          level: "error",
          message: "Plugin returned invalid JSON: " + String((S == null ? void 0 : S.message) || S)
        }), ke = [];
      }
      ke = Array.isArray(ke) ? ke.slice(0, Number(i.maxResultsPerScraper || 25)) : [], s.cleanup(), ie = null, Te({
        type: "result",
        results: ke
      });
    });
  }
  m0.onmessage = function (e) {
    var t,
      n,
      r = (e == null ? void 0 : e.data) || {};
    if (r.type === "fetchResult" && ie) {
      r.error ? (t = ie.settleFetch) == null || t.call(ie, r.requestId, null, new Error(r.error)) : (n = ie.settleFetch) == null || n.call(ie, r.requestId, r.payload || {}, null);
      return;
    }
    r.type === "execute" && c1(r).catch(a => {
      var i, s;
      ie && ((i = ie.rejectPending) == null || i.call(ie, a), (s = ie.cleanup) == null || s.call(ie)), ie = null, Te({
        type: "error",
        error: String((a == null ? void 0 : a.message) || a)
      });
    });
  };
})();