(function(scope){
'use strict';

function F(arity, fun, wrapper) {
  wrapper.a = arity;
  wrapper.f = fun;
  return wrapper;
}

function F2(fun) {
  return F(2, fun, function(a) { return function(b) { return fun(a,b); }; })
}
function F3(fun) {
  return F(3, fun, function(a) {
    return function(b) { return function(c) { return fun(a, b, c); }; };
  });
}
function F4(fun) {
  return F(4, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return fun(a, b, c, d); }; }; };
  });
}
function F5(fun) {
  return F(5, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return fun(a, b, c, d, e); }; }; }; };
  });
}
function F6(fun) {
  return F(6, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return fun(a, b, c, d, e, f); }; }; }; }; };
  });
}
function F7(fun) {
  return F(7, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return fun(a, b, c, d, e, f, g); }; }; }; }; }; };
  });
}
function F8(fun) {
  return F(8, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return function(h) {
    return fun(a, b, c, d, e, f, g, h); }; }; }; }; }; }; };
  });
}
function F9(fun) {
  return F(9, fun, function(a) { return function(b) { return function(c) {
    return function(d) { return function(e) { return function(f) {
    return function(g) { return function(h) { return function(i) {
    return fun(a, b, c, d, e, f, g, h, i); }; }; }; }; }; }; }; };
  });
}

function A2(fun, a, b) {
  return fun.a === 2 ? fun.f(a, b) : fun(a)(b);
}
function A3(fun, a, b, c) {
  return fun.a === 3 ? fun.f(a, b, c) : fun(a)(b)(c);
}
function A4(fun, a, b, c, d) {
  return fun.a === 4 ? fun.f(a, b, c, d) : fun(a)(b)(c)(d);
}
function A5(fun, a, b, c, d, e) {
  return fun.a === 5 ? fun.f(a, b, c, d, e) : fun(a)(b)(c)(d)(e);
}
function A6(fun, a, b, c, d, e, f) {
  return fun.a === 6 ? fun.f(a, b, c, d, e, f) : fun(a)(b)(c)(d)(e)(f);
}
function A7(fun, a, b, c, d, e, f, g) {
  return fun.a === 7 ? fun.f(a, b, c, d, e, f, g) : fun(a)(b)(c)(d)(e)(f)(g);
}
function A8(fun, a, b, c, d, e, f, g, h) {
  return fun.a === 8 ? fun.f(a, b, c, d, e, f, g, h) : fun(a)(b)(c)(d)(e)(f)(g)(h);
}
function A9(fun, a, b, c, d, e, f, g, h, i) {
  return fun.a === 9 ? fun.f(a, b, c, d, e, f, g, h, i) : fun(a)(b)(c)(d)(e)(f)(g)(h)(i);
}

console.warn('Compiled in DEV mode. Follow the advice at https://elm-lang.org/0.19.1/optimize for better performance and smaller assets.');

/* Ideally we would write
 *
 * ```
 * const \_List_Nil = \_\_List_Nil;
 * ```
 *
 * to forward this call `elm/core:List.Nil_elm_builtin` however the elm
 * compiler puts the javascript for `elm/core:List.Nil_elm_builtin` after the
 * javascript below in the elm.js file and so with the above definition we get
 * "XXX is undefined" errors.
 *
 */
const _List_nilKey_UNUSED = 0;
const _List_nilKey = "Nil_elm_builtin";
const _List_Nil = { $: _List_nilKey };

const _List_Cons = (hd, tl) => A2($elm$core$List$Cons_elm_builtin, hd, tl);

const _List_fromArray = (arr) =>
  arr.reduceRight((out, val) => A2($elm$core$List$Cons_elm_builtin, val, out), $elm$core$List$Nil_elm_builtin);

const _List_toArray = (xs) => {
  const out = [];
  while (true) {
    if (xs.$ === _List_nilKey) {
      return out;
    }
    out.push(xs.a);
    xs = xs.b;
  }
};

const _List_sortWith = F2((f, xs) =>
  _List_fromArray(
    _List_toArray(xs).sort((a, b) => {
      const ord = A2(f, a, b);
      return ord === $elm$core$Basics$EQ ? 0 : ord === $elm$core$Basics$LT ? -1 : 1;
    })
  )
);


var _JsArray_empty = [];

function _JsArray_singleton(value) {
  return [value];
}

function _JsArray_length(array) {
  return array.length;
}

var _JsArray_initialize = F3(function (size, offset, func) {
  var result = new Array(size);

  for (var i = 0; i < size; i++) {
    result[i] = func(offset + i);
  }

  return result;
});

var _JsArray_initializeFromList = F2(function (max, ls) {
  var result = new Array(max);

  for (var i = 0; i < max && ls.b; i++) {
    result[i] = ls.a;
    ls = ls.b;
  }

  result.length = i;
  return _Utils_Tuple2(result, ls);
});

var _JsArray_unsafeGet = F2(function (index, array) {
  return array[index];
});

var _JsArray_unsafeSet = F3(function (index, value, array) {
  var length = array.length;
  var result = new Array(length);

  for (var i = 0; i < length; i++) {
    result[i] = array[i];
  }

  result[index] = value;
  return result;
});

var _JsArray_push = F2(function (value, array) {
  var length = array.length;
  var result = new Array(length + 1);

  for (var i = 0; i < length; i++) {
    result[i] = array[i];
  }

  result[length] = value;
  return result;
});

var _JsArray_foldl = F3(function (func, acc, array) {
  var length = array.length;

  for (var i = 0; i < length; i++) {
    acc = A2(func, array[i], acc);
  }

  return acc;
});

var _JsArray_foldr = F3(function (func, acc, array) {
  for (var i = array.length - 1; i >= 0; i--) {
    acc = A2(func, array[i], acc);
  }

  return acc;
});

var _JsArray_map = F2(function (func, array) {
  var length = array.length;
  var result = new Array(length);

  for (var i = 0; i < length; i++) {
    result[i] = func(array[i]);
  }

  return result;
});

var _JsArray_indexedMap = F3(function (func, offset, array) {
  var length = array.length;
  var result = new Array(length);

  for (var i = 0; i < length; i++) {
    result[i] = A2(func, offset + i, array[i]);
  }

  return result;
});

var _JsArray_slice = F3(function (from, to, array) {
  return array.slice(from, to);
});

var _JsArray_appendN = F3(function (n, dest, source) {
  var destLen = dest.length;
  var itemsToCopy = n - destLen;

  if (itemsToCopy > source.length) {
    itemsToCopy = source.length;
  }

  var size = destLen + itemsToCopy;
  var result = new Array(size);

  for (var i = 0; i < destLen; i++) {
    result[i] = dest[i];
  }

  for (var i = 0; i < itemsToCopy; i++) {
    result[i + destLen] = source[i];
  }

  return result;
});


// LOG

const _Debug_log_UNUSED = F2(function (tag, value) {
  return value;
});

const _Debug_log = F2(function (tag, value) {
  console.log(tag + ": " + _Debug_toString(value));
  return value;
});

// TODOS

function _Debug_todo(moduleName, region) {
  return function (message) {
    _Debug_crash(8, moduleName, region, message);
  };
}

function _Debug_todoCase(moduleName, region, value) {
  return function (message) {
    _Debug_crash(9, moduleName, region, value, message);
  };
}

// TO STRING

function _Debug_toString_UNUSED(value) {
  return "<internals>";
}

function _Debug_toString(value) {
  return _Debug_toAnsiString(false, value);
}

function _Debug_toAnsiString(ansi, value) {
  if (typeof value === "function") {
    return _Debug_internalColor(ansi, "<function>");
  }

  if (typeof value === "boolean") {
    return _Debug_ctorColor(ansi, value ? "True" : "False");
  }

  if (typeof value === "number") {
    return _Debug_numberColor(ansi, value + "");
  }

  if (value instanceof String) {
    return _Debug_charColor(ansi, "'" + _Debug_addSlashes(value, true) + "'");
  }

  if (typeof value === "string") {
    return _Debug_stringColor(ansi, '"' + _Debug_addSlashes(value, false) + '"');
  }

  if (typeof value === "object" && "$" in value) {
    const tag = value.$;

    if (typeof tag === "number") {
      return _Debug_internalColor(ansi, "<internals>");
    }

    if (tag[0] === "#") {
      const output = [];
      for (const [k, v] of Object.entries(value)) {
        if (k === "$") continue;
        output.push(_Debug_toAnsiString(ansi, v));
      }
      return "(" + output.join(",") + ")";
    }

    if (tag === "Set_elm_builtin") {
      return (
        _Debug_ctorColor(ansi, "Set") +
        _Debug_fadeColor(ansi, ".fromList") +
        " " +
        _Debug_toAnsiString(ansi, $elm$core$Set$toList(value))
      );
    }

    if (tag === "RBNode_elm_builtin" || tag === "RBEmpty_elm_builtin") {
      return (
        _Debug_ctorColor(ansi, "Dict") +
        _Debug_fadeColor(ansi, ".fromList") +
        " " +
        _Debug_toAnsiString(ansi, $elm$core$Dict$toList(value))
      );
    }

    if (tag === "Array_elm_builtin") {
      return (
        _Debug_ctorColor(ansi, "Array") +
        _Debug_fadeColor(ansi, ".fromList") +
        " " +
        _Debug_toAnsiString(ansi, $elm$core$Array$toList(value))
      );
    }

    if (tag === "Cons_elm_builtin" || tag === "Nil_elm_builtin") {
      return (
        "[" +
        _List_toArray(value)
          .map((v) => _Debug_toAnsiString(ansi, v))
          .join(",") +
        "]"
      );
    }

    const parts = Object.entries(value).map(([k, v]) => {
      if (k === "$") {
        return _Debug_ctorColor(ansi, v);
      }
      const str = _Debug_toAnsiString(ansi, v);
      const c0 = str[0];
      const parenless =
        c0 === "{" || c0 === "(" || c0 === "[" || c0 === "<" || c0 === '"' || str.indexOf(" ") < 0;
      return parenless ? str : "(" + str + ")";
    });
    return parts.join(" ");
  }

  if (typeof DataView === "function" && value instanceof DataView) {
    return _Debug_stringColor(ansi, "<" + value.byteLength + " bytes>");
  }

  if (typeof File !== "undefined" && value instanceof File) {
    return _Debug_internalColor(ansi, "<" + value.name + ">");
  }

  if (typeof value === "object") {
    const keyValuePairs = Object.entries(value).map(([k, v]) => {
      const field = k[0] === "_" ? k.slice(1) : k;
      return _Debug_fadeColor(ansi, field) + " = " + _Debug_toAnsiString(ansi, v);
    });
    return "{ " + keyValuePairs.join(", ") + " }";
  }

  return _Debug_internalColor(ansi, "<internals>");
}

function _Debug_addSlashes(str, isChar) {
  const s = str
    .replace(/\\/g, "\\\\")
    .replace(/\n/g, "\\n")
    .replace(/\t/g, "\\t")
    .replace(/\r/g, "\\r")
    .replace(/\v/g, "\\v")
    .replace(/\0/g, "\\0");

  if (isChar) {
    return s.replace(/\'/g, "\\'");
  } else {
    return s.replace(/\"/g, '\\"');
  }
}

function _Debug_ctorColor(ansi, string) {
  return ansi ? "\x1b[96m" + string + "\x1b[0m" : string;
}

function _Debug_numberColor(ansi, string) {
  return ansi ? "\x1b[95m" + string + "\x1b[0m" : string;
}

function _Debug_stringColor(ansi, string) {
  return ansi ? "\x1b[93m" + string + "\x1b[0m" : string;
}

function _Debug_charColor(ansi, string) {
  return ansi ? "\x1b[92m" + string + "\x1b[0m" : string;
}

function _Debug_fadeColor(ansi, string) {
  return ansi ? "\x1b[37m" + string + "\x1b[0m" : string;
}

function _Debug_internalColor(ansi, string) {
  return ansi ? "\x1b[36m" + string + "\x1b[0m" : string;
}

function _Debug_toHexDigit(n) {
  return String.fromCharCode(n < 10 ? 48 + n : 55 + n);
}

// CRASH

function _Debug_runtimeCrashReason_UNUSED(reason) {}

function _Debug_runtimeCrashReason(reason) {
  switch (reason) {
    case "subMap":
      return function (fact2, fact3, fact4) {
        throw new Error(
          "Bug in elm runtime: attempting to subMap an effect from a command only effect module."
        );
      };

    case "cmdMap":
      return function (fact2, fact3, fact4) {
        throw new Error(
          "Bug in elm runtime: attempting to cmdMap an effect from a subscription only effect module."
        );
      };

    case "procIdAlreadyRegistered":
      return function (fact2, fact3, fact4) {
        throw new Error(`Bug in elm runtime: state for process ${fact2} is already registered!`);
      };

    case "procIdNotRegistered":
      return function (fact2, fact3, fact4) {
        throw new Error(`Bug in elm runtime: state for process ${fact2} been has not registered!`);
      };

    case "cannotBeStepped":
      return function (fact2, fact3, fact4) {
        throw new Error(
          `Bug in elm runtime: attempting to step process with id ${fact2} whilst it is processing an async action!`
        );
      };

    case "procIdAlreadyReady":
      return function (fact2, fact3, fact4) {
        throw new Error(
          `Bug in elm runtime: process ${fact2} already has a ready flag set (with value ${fact3}). Refusing to reset the value before it is cleared`
        );
      };

    case "subscriptionProcessMissing":
      return function (fact2, fact3, fact4) {
        throw new Error(
          `Bug in elm runtime: expected there to be a subscriptionProcess with id ${fact2}.`
        );
      };

    case "failedUnwrap":
      return function (fact2, fact3, fact4) {
        throw new Error(
          `Bug in elm runtime: trying to unwrap an new type but the js object had the following keys: ${Object.keys(
            fact2
          ).join(", ")}`
        );
      };

    case "EffectModule":
      return function (fact2, fact3, fact4) {
        throw new Error(
          `Effect modules are not supported, if you are using elm/* libraries you will need to switch to a custom version.`
        );
      };

    case "PlatformLeaf":
      return function (home, fact3, fact4) {
        throw new Error(
          `Trying to create a command or a subscription for event manager ${home}.
Effect modules are not supported, if you are using elm/* libraries you will need to switch to a custom version.`
        );
      };
  }
  throw new Error(`Unknown reason for runtime crash: ${fact1}!`);
}

function _Debug_crash_UNUSED(identifier) {
  throw new Error("Error in whilst running elm app id:" + identifier);
}

function _Debug_crash(identifier, fact1, fact2, fact3, fact4) {
  switch (identifier) {
    case 0:
      throw new Error(
        'What node should I take over? In JavaScript I need something like:\n\n    Elm.Main.init({\n        node: document.getElementById("elm-node")\n    })\n\nYou need to do this with any Browser.sandbox or Browser.element program.'
      );

    case 1:
      throw new Error(
        "Browser.application programs cannot handle URLs like this:\n\n    " +
          document.location.href +
          "\n\nWhat is the root? The root of your file system? Try looking at this program with `elm reactor` or some other server."
      );

    case 2: {
      const jsonErrorString = fact1;
      throw new Error(
        "Problem with the flags given to your Elm program on initialization.\n\n" + jsonErrorString
      );
    }

    case 3: {
      const portName = fact1;
      throw new Error(
        "There can only be one port named `" + portName + "`, but your program has multiple."
      );
    }

    case 4: {
      const portName = fact1;
      const problem = fact2;
      throw new Error(
        "Trying to send an unexpected type of value through port `" + portName + "`:\n" + problem
      );
    }

    case 5:
      throw new Error(
        'Trying to use `(==)` on functions.\nThere is no way to know if functions are "the same" in the Elm sense.\nRead more about this at https://package.elm-lang.org/packages/elm/core/latest/Basics#== which describes why it is this way and what the better version will look like.'
      );

    case 6: {
      const moduleName = fact1;
      throw new Error(
        "Your page is loading multiple Elm scripts with a module named " +
          moduleName +
          ". Maybe a duplicate script is getting loaded accidentally? If not, rename one of them so I know which is which!"
      );
    }

    case 8: {
      const moduleName = fact1;
      const region = fact2;
      const message = fact3;
      throw new Error(
        "TODO in module `" + moduleName + "` " + _Debug_regionToString(region) + "\n\n" + message
      );
    }

    case 9: {
      const moduleName = fact1;
      const region = fact2;
      const value = fact3;
      const message = fact4;
      throw new Error(
        "TODO in module `" +
          moduleName +
          "` from the `case` expression " +
          _Debug_regionToString(region) +
          "\n\nIt received the following value:\n\n    " +
          _Debug_toString(value).replace("\n", "\n    ") +
          "\n\nBut the branch that handles it says:\n\n    " +
          message.replace("\n", "\n    ")
      );
    }

    case 10:
      throw new Error("Bug in https://github.com/elm/virtual-dom/issues");

    case 11:
      throw new Error("Cannot perform mod 0. Division by zero error.");

    case 12: {
      fact1(fact2, fact3, fact4);
      throw new Error(`Unknown bug in elm runtime tag: ${fact1}!`);
    }
  }
}

function _Debug_regionToString(region) {
  if (region.start.line === region.end.line) {
    return "on line " + region.start.line;
  }
  return "on lines " + region.start.line + " through " + region.end.line;
}


// MATH

const _Basics_pow = F2(Math.pow);

const _Basics_cos = Math.cos;
const _Basics_sin = Math.sin;
const _Basics_tan = Math.tan;
const _Basics_acos = Math.acos;
const _Basics_asin = Math.asin;
const _Basics_atan = Math.atan;
const _Basics_atan2 = F2(Math.atan2);

const _Basics_ceiling = Math.ceil;
const _Basics_floor = Math.floor;
const _Basics_round = Math.round;
const _Basics_sqrt = Math.sqrt;
const _Basics_log = Math.log;

const _Basics_modBy0 = () => _Debug_crash(11);

const _Basics_fudgeType = (x) => x;

const _Basics_unwrapTypeWrapper = (wrapped) => {
  const entries = Object.entries(wrapped);
  if (entries.length !== 2) {
    _Debug_crash(12, _Debug_runtimeCrashReason("failedUnwrap"), wrapped);
  }
  if (entries[0][0] === "$") {
    return entries[1][1];
  } else {
    return entries[0][1];
  }
};

const _Basics_unwrapTypeWrapper_UNUSED = (wrapped) => wrapped;

const _Basics_isDebug = true;
const _Basics_isDebug_UNUSED = false;


// EQUALITY

const _Utils_eq = (x, y) => {
  const stack = [];
  while (_Utils_eqHelp(x, y, 0, stack)) {
    const pair = stack.pop();
    if (pair === undefined) {
      return true;
    }
    [x, y] = pair;
  }
  return false;
};

function _Utils_eqHelp(x, y, depth, stack) {
  if (x === y) {
    return true;
  }

  if (typeof x !== "object" || x === null || y === null) {
    if (typeof x === "function") {
      _Debug_crash(5);
    }
    return false;
  }

  if (depth > 100) {
    stack.push([x, y]);
    return true;
  }

  if (_Basics_isDebug) {
    if (x.$ === "Set_elm_builtin") {
      x = $elm$core$Set$toList(x);
      y = $elm$core$Set$toList(y);
    } else if (x.$ === "RBNode_elm_builtin" || x.$ === "RBEmpty_elm_builtin") {
      x = $elm$core$Dict$toList(x);
      y = $elm$core$Dict$toList(y);
    }
  } else {
    if (x.$ < 0) {
      x = $elm$core$Dict$toList(x);
      y = $elm$core$Dict$toList(y);
    }
  }

  /* The compiler ensures that the elm types of x and y are the same.
   * Therefore, x and y must have the same keys.
   */
  for (const key of Object.keys(x)) {
    if (!_Utils_eqHelp(x[key], y[key], depth + 1, stack)) {
      return false;
    }
  }
  return true;
}

const _Utils_equal = F2(_Utils_eq);
const _Utils_notEqual = F2(function (a, b) {
  return !_Utils_eq(a, b);
});

// COMPARISONS

// Code in Generate/JavaScript/Expression.hs and Basics.elm depends on the
// particular integer values assigned to LT, EQ, and GT. Comparable types are:
// numbers, characters, strings, lists of comparable things, and tuples of
// comparable things.
function _Utils_cmp(x, y, ord) {
  // Handle numbers, strings and characters in production mode.
  if (typeof x !== "object") {
    return x === y ? /*EQ*/ 0 : x < y ? /*LT*/ -1 : /*GT*/ 1;
  }

  // Handle characters in debug mode.
  if (_Basics_isDebug && x instanceof String) {
    const a = x.valueOf();
    const b = y.valueOf();
    return a === b ? 0 : a < b ? -1 : 1;
  }

  // Handle tuples.
  const isTuple = _Basics_isDebug ? x.$[0] === "#" : x.$ === undefined;
  if (isTuple) {
    const ordA = _Utils_cmp(x.a, y.a);
    if (ordA !== 0) {
      return ordA;
    }
    const ordB = _Utils_cmp(x.a, y.a);
    if (ordB !== 0) {
      return ordB;
    }
    return _Utils_cmp(x.c, y.c);
  }

  // Handle lists: traverse conses until end of a list or a mismatch. If the
  // all the elements in one list are equal to all the elements in other list
  // but the first list is longer than the first list is greater (and visa
  // versa).
  while (true) {
    if (x.$ === _List_nilKey) {
      if (y.$ === _List_nilKey) {
        return 0;
      } else {
        return -1;
      }
    } else if (y.$ === _List_nilKey) {
      return 1;
    }
    const ord = _Utils_cmp(x.a, y.a);
    if (ord !== 0) {
      return ord;
    }
    x = x.b;
    y = y.b;
  }
}

const _Utils_compare = F2((x, y) => _Utils_cmp(x, y));

// COMMON VALUES

const _Utils_Tuple0_UNUSED = 0;
const _Utils_Tuple0 = { $: "#0" };

const _Utils_Tuple2_UNUSED = (a, b) => ({ a, b });
const _Utils_Tuple2 = (a, b) => ({ $: "#2", a, b });

const _Utils_Tuple3_UNUSED = (a, b, c) => ({ a, b, c });
const _Utils_Tuple3 = (a, b, c) => ({ $: "#3", a, b, c });

const _Utils_chr_UNUSED = (c) => c;
const _Utils_chr = (c) => new String(c);

// RECORDS

const _Utils_update = (oldRecord, updatedFields) => Object.assign({}, oldRecord, updatedFields);

// APPEND

const _Utils_ap = (xs, ys) => {
  // append Strings
  if (typeof xs === "string") {
    return xs + ys;
  }

  // append Lists
  return A2($elm$core$List$append, xs, ys);
};


const _Channel_channels = new WeakMap();
let _Channel_channelId = 0;

const _Channel_rawUnbounded = (_) => {
  const id = {
    id: _Channel_channelId++,
  };
  _Channel_channels.set(id, {
    messages: [],
    wakers: new Set(),
  });
  return _Utils_Tuple2(_Channel_rawSendImpl(id), id);
};

const _Channel_rawTryRecv = (channelId) => {
  const channel = _Channel_channels.get(channelId);
  if (_Basics_isDebug && channel === undefined) {
    _Debug_crash(
      12,
      _Debug_runtimeCrashReason("channelIdNotRegistered"),
      channelId && channelId.a && channelId.a.id
    );
  }

  const msg = channel.messages.shift();
  if (msg === undefined) {
    return $elm$core$Maybe$Nothing;
  } else {
    return $elm$core$Maybe$Just(msg);
  }
};

const _Channel_rawRecv = F2((channelId, onMsg) => {
  const channel = _Channel_channels.get(channelId);
  if (_Basics_isDebug && channel === undefined) {
    _Debug_crash(
      12,
      _Debug_runtimeCrashReason("channelIdNotRegistered"),
      channelId && channelId.a && channelId.a.id
    );
  }
  const msg = channel.messages.shift();
  if (msg !== undefined) {
    onMsg(msg);
    return (x) => x;
  }
  const onWake = (msg) => {
    return onMsg(msg);
  };
  channel.wakers.add(onWake);
  return (x) => {
    channel.wakers.delete(onWake);
    return x;
  };
});

const _Channel_rawSendImpl = F2((channelId, msg) => {
  const channel = _Channel_channels.get(channelId);
  if (_Basics_isDebug && channel === undefined) {
    _Debug_crash(
      12,
      _Debug_runtimeCrashReason("channelIdNotRegistered"),
      channelId && channelId.a && channelId.a.id
    );
  }

  const wakerIter = channel.wakers[Symbol.iterator]();
  const { value: nextWaker, done } = wakerIter.next();
  if (done) {
    channel.messages.push(msg);
  } else {
    channel.wakers.delete(nextWaker);
    nextWaker(msg);
  }
  return _Utils_Tuple0;
});

const _Channel_rawSend = F2((sender, msg) => {
  sender(msg);
});


var _String_cons = F2(function (chr, str) {
  return chr + str;
});

function _String_uncons(string) {
  var word = string.charCodeAt(0);
  return !isNaN(word)
    ? $elm$core$Maybe$Just(
        0xd800 <= word && word <= 0xdbff
          ? _Utils_Tuple2(_Utils_chr(string[0] + string[1]), string.slice(2))
          : _Utils_Tuple2(_Utils_chr(string[0]), string.slice(1))
      )
    : $elm$core$Maybe$Nothing;
}

var _String_append = F2(function (a, b) {
  return a + b;
});

function _String_length(str) {
  return str.length;
}

var _String_map = F2(function (func, string) {
  var len = string.length;
  var array = new Array(len);
  var i = 0;
  while (i < len) {
    var word = string.charCodeAt(i);
    if (0xd800 <= word && word <= 0xdbff) {
      array[i] = func(_Utils_chr(string[i] + string[i + 1]));
      i += 2;
      continue;
    }
    array[i] = func(_Utils_chr(string[i]));
    i++;
  }
  return array.join("");
});

var _String_filter = F2(function (isGood, str) {
  var arr = [];
  var len = str.length;
  var i = 0;
  while (i < len) {
    var char = str[i];
    var word = str.charCodeAt(i);
    i++;
    if (0xd800 <= word && word <= 0xdbff) {
      char += str[i];
      i++;
    }

    if (isGood(_Utils_chr(char))) {
      arr.push(char);
    }
  }
  return arr.join("");
});

function _String_reverse(str) {
  var len = str.length;
  var arr = new Array(len);
  var i = 0;
  while (i < len) {
    var word = str.charCodeAt(i);
    if (0xd800 <= word && word <= 0xdbff) {
      arr[len - i] = str[i + 1];
      i++;
      arr[len - i] = str[i - 1];
      i++;
    } else {
      arr[len - i] = str[i];
      i++;
    }
  }
  return arr.join("");
}

var _String_foldl = F3(function (func, state, string) {
  var len = string.length;
  var i = 0;
  while (i < len) {
    var char = string[i];
    var word = string.charCodeAt(i);
    i++;
    if (0xd800 <= word && word <= 0xdbff) {
      char += string[i];
      i++;
    }
    state = A2(func, _Utils_chr(char), state);
  }
  return state;
});

var _String_foldr = F3(function (func, state, string) {
  var i = string.length;
  while (i--) {
    var char = string[i];
    var word = string.charCodeAt(i);
    if (0xdc00 <= word && word <= 0xdfff) {
      i--;
      char = string[i] + char;
    }
    state = A2(func, _Utils_chr(char), state);
  }
  return state;
});

var _String_split = F2(function (sep, str) {
  return str.split(sep);
});

var _String_join = F2(function (sep, strs) {
  return strs.join(sep);
});

var _String_slice = F3(function (start, end, str) {
  return str.slice(start, end);
});

function _String_trim(str) {
  return str.trim();
}

function _String_trimLeft(str) {
  return str.replace(/^\s+/, "");
}

function _String_trimRight(str) {
  return str.replace(/\s+$/, "");
}

function _String_words(str) {
  return _List_fromArray(str.trim().split(/\s+/g));
}

function _String_lines(str) {
  return _List_fromArray(str.split(/\r\n|\r|\n/g));
}

function _String_toUpper(str) {
  return str.toUpperCase();
}

function _String_toLower(str) {
  return str.toLowerCase();
}

var _String_any = F2(function (isGood, string) {
  var i = string.length;
  while (i--) {
    var char = string[i];
    var word = string.charCodeAt(i);
    if (0xdc00 <= word && word <= 0xdfff) {
      i--;
      char = string[i] + char;
    }
    if (isGood(_Utils_chr(char))) {
      return true;
    }
  }
  return false;
});

var _String_all = F2(function (isGood, string) {
  var i = string.length;
  while (i--) {
    var char = string[i];
    var word = string.charCodeAt(i);
    if (0xdc00 <= word && word <= 0xdfff) {
      i--;
      char = string[i] + char;
    }
    if (!isGood(_Utils_chr(char))) {
      return false;
    }
  }
  return true;
});

var _String_contains = F2(function (sub, str) {
  return str.indexOf(sub) > -1;
});

var _String_startsWith = F2(function (sub, str) {
  return str.indexOf(sub) === 0;
});

var _String_endsWith = F2(function (sub, str) {
  return str.length >= sub.length && str.lastIndexOf(sub) === str.length - sub.length;
});

var _String_indexes = F2(function (sub, str) {
  var subLen = sub.length;

  if (subLen < 1) {
    return $elm$core$List$Nil_elm_builtin;
  }

  var i = 0;
  var is = [];

  while ((i = str.indexOf(sub, i)) > -1) {
    is.push(i);
    i = i + subLen;
  }

  return _List_fromArray(is);
});

// TO STRING

function _String_fromNumber(number) {
  return number + "";
}

// INT CONVERSIONS

function _String_toInt(str) {
  var total = 0;
  var code0 = str.charCodeAt(0);
  var start = code0 == 0x2b /* + */ || code0 == 0x2d /* - */ ? 1 : 0;

  for (var i = start; i < str.length; ++i) {
    var code = str.charCodeAt(i);
    if (code < 0x30 || 0x39 < code) {
      return $elm$core$Maybe$Nothing;
    }
    total = 10 * total + code - 0x30;
  }

  return i == start ? $elm$core$Maybe$Nothing : $elm$core$Maybe$Just(code0 == 0x2d ? -total : total);
}

// FLOAT CONVERSIONS

function _String_toFloat(s) {
  // check if it is a hex, octal, or binary number
  if (s.length === 0 || /[\sxbo]/.test(s)) {
    return $elm$core$Maybe$Nothing;
  }
  var n = +s;
  // faster isNaN check
  return n === n ? $elm$core$Maybe$Just(n) : $elm$core$Maybe$Nothing;
}

function _String_fromList(chars) {
  return _List_toArray(chars).join("");
}


function _Char_toCode(char) {
  const code = char.charCodeAt(0);
  if (0xd800 <= code && code <= 0xdbff) {
    return (code - 0xd800) * 0x400 + char.charCodeAt(1) - 0xdc00 + 0x10000;
  }
  return code;
}

function _Char_fromCode(code) {
  return _Utils_chr(
    code < 0 || 0x10ffff < code
      ? "\uFFFD"
      : code <= 0xffff
      ? String.fromCharCode(code)
      : ((code -= 0x10000),
        String.fromCharCode(Math.floor(code / 0x400) + 0xd800, (code % 0x400) + 0xdc00))
  );
}

function _Char_toUpper(char) {
  return _Utils_chr(char.toUpperCase());
}

function _Char_toLower(char) {
  return _Utils_chr(char.toLowerCase());
}

function _Char_toLocaleUpper(char) {
  return _Utils_chr(char.toLocaleUpperCase());
}

function _Char_toLocaleLower(char) {
  return _Utils_chr(char.toLocaleLowerCase());
}



/**/
function _Json_errorToString(error)
{
	return $elm$json$Json$Decode$errorToString(error);
}
//*/


// CORE DECODERS

function _Json_succeed(msg)
{
	return {
		$: 0,
		a: msg
	};
}

function _Json_fail(msg)
{
	return {
		$: 1,
		a: msg
	};
}

function _Json_decodePrim(decoder)
{
	return { $: 2, b: decoder };
}

var _Json_decodeInt = _Json_decodePrim(function(value) {
	return (typeof value !== 'number')
		? _Json_expecting('an INT', value)
		:
	(-2147483647 < value && value < 2147483647 && (value | 0) === value)
		? $elm$core$Result$Ok(value)
		:
	(isFinite(value) && !(value % 1))
		? $elm$core$Result$Ok(value)
		: _Json_expecting('an INT', value);
});

var _Json_decodeBool = _Json_decodePrim(function(value) {
	return (typeof value === 'boolean')
		? $elm$core$Result$Ok(value)
		: _Json_expecting('a BOOL', value);
});

var _Json_decodeFloat = _Json_decodePrim(function(value) {
	return (typeof value === 'number')
		? $elm$core$Result$Ok(value)
		: _Json_expecting('a FLOAT', value);
});

var _Json_decodeValue = _Json_decodePrim(function(value) {
	return $elm$core$Result$Ok(_Json_wrap(value));
});

var _Json_decodeString = _Json_decodePrim(function(value) {
	return (typeof value === 'string')
		? $elm$core$Result$Ok(value)
		: (value instanceof String)
			? $elm$core$Result$Ok(value + '')
			: _Json_expecting('a STRING', value);
});

function _Json_decodeList(decoder) { return { $: 3, b: decoder }; }
function _Json_decodeArray(decoder) { return { $: 4, b: decoder }; }

function _Json_decodeNull(value) { return { $: 5, c: value }; }

var _Json_decodeField = F2(function(field, decoder)
{
	return {
		$: 6,
		d: field,
		b: decoder
	};
});

var _Json_decodeIndex = F2(function(index, decoder)
{
	return {
		$: 7,
		e: index,
		b: decoder
	};
});

function _Json_decodeKeyValuePairs(decoder)
{
	return {
		$: 8,
		b: decoder
	};
}

function _Json_mapMany(f, decoders)
{
	return {
		$: 9,
		f: f,
		g: decoders
	};
}

var _Json_andThen = F2(function(callback, decoder)
{
	return {
		$: 10,
		b: decoder,
		h: callback
	};
});

function _Json_oneOf(decoders)
{
	return {
		$: 11,
		g: decoders
	};
}


// DECODING OBJECTS

var _Json_map1 = F2(function(f, d1)
{
	return _Json_mapMany(f, [d1]);
});

var _Json_map2 = F3(function(f, d1, d2)
{
	return _Json_mapMany(f, [d1, d2]);
});

var _Json_map3 = F4(function(f, d1, d2, d3)
{
	return _Json_mapMany(f, [d1, d2, d3]);
});

var _Json_map4 = F5(function(f, d1, d2, d3, d4)
{
	return _Json_mapMany(f, [d1, d2, d3, d4]);
});

var _Json_map5 = F6(function(f, d1, d2, d3, d4, d5)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5]);
});

var _Json_map6 = F7(function(f, d1, d2, d3, d4, d5, d6)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6]);
});

var _Json_map7 = F8(function(f, d1, d2, d3, d4, d5, d6, d7)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6, d7]);
});

var _Json_map8 = F9(function(f, d1, d2, d3, d4, d5, d6, d7, d8)
{
	return _Json_mapMany(f, [d1, d2, d3, d4, d5, d6, d7, d8]);
});


// DECODE

var _Json_runOnString = F2(function(decoder, string)
{
	try
	{
		var value = JSON.parse(string);
		return _Json_runHelp(decoder, value);
	}
	catch (e)
	{
		return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, 'This is not valid JSON! ' + e.message, _Json_wrap(string)));
	}
});

var _Json_run = F2(function(decoder, value)
{
	return _Json_runHelp(decoder, _Json_unwrap(value));
});

function _Json_runHelp(decoder, value)
{
	switch (decoder.$)
	{
		case 2:
			return decoder.b(value);

		case 5:
			return (value === null)
				? $elm$core$Result$Ok(decoder.c)
				: _Json_expecting('null', value);

		case 3:
			if (!_Json_isArray(value))
			{
				return _Json_expecting('a LIST', value);
			}
			return _Json_runArrayDecoder(decoder.b, value, _List_fromArray);

		case 4:
			if (!_Json_isArray(value))
			{
				return _Json_expecting('an ARRAY', value);
			}
			return _Json_runArrayDecoder(decoder.b, value, _Json_toElmArray);

		case 6:
			var field = decoder.d;
			if (typeof value !== 'object' || value === null || !(field in value))
			{
				return _Json_expecting('an OBJECT with a field named `' + field + '`', value);
			}
			var result = _Json_runHelp(decoder.b, value[field]);
			return ($elm$core$Result$isOk(result)) ? result : $elm$core$Result$Err(A2($elm$json$Json$Decode$Field, field, result.a));

		case 7:
			var index = decoder.e;
			if (!_Json_isArray(value))
			{
				return _Json_expecting('an ARRAY', value);
			}
			if (index >= value.length)
			{
				return _Json_expecting('a LONGER array. Need index ' + index + ' but only see ' + value.length + ' entries', value);
			}
			var result = _Json_runHelp(decoder.b, value[index]);
			return ($elm$core$Result$isOk(result)) ? result : $elm$core$Result$Err(A2($elm$json$Json$Decode$Index, index, result.a));

		case 8:
			if (typeof value !== 'object' || value === null || _Json_isArray(value))
			{
				return _Json_expecting('an OBJECT', value);
			}

			var keyValuePairs = _List_Nil;
			// TODO test perf of Object.keys and switch when support is good enough
			for (var key in value)
			{
				if (value.hasOwnProperty(key))
				{
					var result = _Json_runHelp(decoder.b, value[key]);
					if (!$elm$core$Result$isOk(result))
					{
						return $elm$core$Result$Err(A2($elm$json$Json$Decode$Field, key, result.a));
					}
					keyValuePairs = _List_Cons(_Utils_Tuple2(key, result.a), keyValuePairs);
				}
			}
			return $elm$core$Result$Ok($elm$core$List$reverse(keyValuePairs));

		case 9:
			var answer = decoder.f;
			var decoders = decoder.g;
			for (var i = 0; i < decoders.length; i++)
			{
				var result = _Json_runHelp(decoders[i], value);
				if (!$elm$core$Result$isOk(result))
				{
					return result;
				}
				answer = answer(result.a);
			}
			return $elm$core$Result$Ok(answer);

		case 10:
			var result = _Json_runHelp(decoder.b, value);
			return (!$elm$core$Result$isOk(result))
				? result
				: _Json_runHelp(decoder.h(result.a), value);

		case 11:
			var errors = _List_Nil;
			for (var temp = decoder.g; temp.b; temp = temp.b) // WHILE_CONS
			{
				var result = _Json_runHelp(temp.a, value);
				if ($elm$core$Result$isOk(result))
				{
					return result;
				}
				errors = _List_Cons(result.a, errors);
			}
			return $elm$core$Result$Err($elm$json$Json$Decode$OneOf($elm$core$List$reverse(errors)));

		case 1:
			return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, decoder.a, _Json_wrap(value)));

		case 0:
			return $elm$core$Result$Ok(decoder.a);
	}
}

function _Json_runArrayDecoder(decoder, value, toElmValue)
{
	var len = value.length;
	var array = new Array(len);
	for (var i = 0; i < len; i++)
	{
		var result = _Json_runHelp(decoder, value[i]);
		if (!$elm$core$Result$isOk(result))
		{
			return $elm$core$Result$Err(A2($elm$json$Json$Decode$Index, i, result.a));
		}
		array[i] = result.a;
	}
	return $elm$core$Result$Ok(toElmValue(array));
}

function _Json_isArray(value)
{
	return Array.isArray(value) || (typeof FileList !== 'undefined' && value instanceof FileList);
}

function _Json_toElmArray(array)
{
	return A2($elm$core$Array$initialize, array.length, function(i) { return array[i]; });
}

function _Json_expecting(type, value)
{
	return $elm$core$Result$Err(A2($elm$json$Json$Decode$Failure, 'Expecting ' + type, _Json_wrap(value)));
}


// EQUALITY

function _Json_equality(x, y)
{
	if (x === y)
	{
		return true;
	}

	if (x.$ !== y.$)
	{
		return false;
	}

	switch (x.$)
	{
		case 0:
		case 1:
			return x.a === y.a;

		case 2:
			return x.b === y.b;

		case 5:
			return x.c === y.c;

		case 3:
		case 4:
		case 8:
			return _Json_equality(x.b, y.b);

		case 6:
			return x.d === y.d && _Json_equality(x.b, y.b);

		case 7:
			return x.e === y.e && _Json_equality(x.b, y.b);

		case 9:
			return x.f === y.f && _Json_listEquality(x.g, y.g);

		case 10:
			return x.h === y.h && _Json_equality(x.b, y.b);

		case 11:
			return _Json_listEquality(x.g, y.g);
	}
}

function _Json_listEquality(aDecoders, bDecoders)
{
	var len = aDecoders.length;
	if (len !== bDecoders.length)
	{
		return false;
	}
	for (var i = 0; i < len; i++)
	{
		if (!_Json_equality(aDecoders[i], bDecoders[i]))
		{
			return false;
		}
	}
	return true;
}


// ENCODE

var _Json_encode = F2(function(indentLevel, value)
{
	return JSON.stringify(_Json_unwrap(value), null, indentLevel) + '';
});

function _Json_wrap(value) { return { $: 0, a: value }; }
function _Json_unwrap(value) { return value.a; }

function _Json_wrap_UNUSED(value) { return value; }
function _Json_unwrap_UNUSED(value) { return value; }

function _Json_emptyArray() { return []; }
function _Json_emptyObject() { return {}; }

var _Json_addField = F3(function(key, value, object)
{
	object[key] = _Json_unwrap(value);
	return object;
});

function _Json_addEntry(func)
{
	return F2(function(entry, array)
	{
		array.push(_Json_unwrap(func(entry)));
		return array;
	});
}

var _Json_encodeNull = _Json_wrap(null);


// COMPATIBILITY

/*
 * We include these to avoid having to change code in other `elm/*` packages.
 *
 * We have to define these as functions rather than variables as the
 * implementations of elm/core:Platform.Scheduler.* functions may come later in
 * the generated javascript file.
 *
 * **IMPORTANT**: these functions return `Process.Task`s and
 * `Process.ProcessId`s rather than `RawScheduler.Task`s and
 * `RawScheduler.ProcessId`s for compatability with `elm/*` package code.
 */

function _Scheduler_succeed(value) {
  return $elm$core$Platform$Scheduler$succeed(value);
}

function _Scheduler_binding(future) {
  return $elm$core$Platform$Scheduler$binding(future);
}

function _Scheduler_rawSpawn(task) {
  return $elm$core$Platform$Scheduler$rawSpawn(task);
}

// SCHEDULER

var _Scheduler_guid = 0;
var _Scheduler_processes = new WeakMap();

function _Scheduler_getGuid() {
  return _Scheduler_guid++;
}

function _Scheduler_getProcessState(id) {
  const procState = _Scheduler_processes.get(id);
  if (procState === undefined) {
    return $elm$core$Maybe$Nothing;
  }
  return $elm$core$Maybe$Just(procState);
}

var _Scheduler_registerNewProcess = F2((procId, procState) => {
  if (_Basics_isDebug && _Scheduler_processes.has(procId)) {
    _Debug_crash(
      12,
      _Debug_runtimeCrashReason("procIdAlreadyRegistered"),
      procId && procId.a && procId.a.id
    );
  }
  _Scheduler_processes.set(procId, procState);
  return procId;
});

const _Scheduler_enqueueWithStepper = (stepper) => {
  let working = false;
  const queue = [];

  return (procId) => (rootTask) => {
    if (_Basics_isDebug && queue.some((p) => p[0].a.id === procId.a.id)) {
      _Debug_crash(
        12,
        _Debug_runtimeCrashReason("procIdAlreadyInQueue"),
        procId && procId.a && procId.a.id
      );
    }
    queue.push([procId, rootTask]);
    if (working) {
      return procId;
    }
    working = true;
    while (true) {
      const next = queue.shift();
      if (next === undefined) {
        working = false;
        return procId;
      }
      const [newProcId, newRootTask] = next;
      _Scheduler_processes.set(newProcId, A2(stepper, newProcId, newRootTask));
    }
  };
};

const _Scheduler_delay = F2((time, value) => ({
  then_: (callback) => {
    let id = setTimeout(() => {
      callback(value);
    }, time);
    return (x) => {
      if (id !== null) {
        clearTimeout(id);
        id = null;
      }
      return x;
    };
  },
}));


// State

var _Platform_outgoingPorts = new Map();
var _Platform_incomingPorts = new Map();

var _Platform_effectsQueue = [];
var _Platform_effectDispatchInProgress = false;

let _Platform_runAfterLoadQueue = [];
const _Platform_runAfterLoad = (f) => {
  if (_Platform_runAfterLoadQueue == null) {
    f();
  } else {
    _Platform_runAfterLoadQueue.push(f);
  }
};

// INITIALIZE A PROGRAM

const _Platform_initialize = F3((flagDecoder, args, impl) => {
  throw(1);
  // Elm.Kernel.Json.wrap : RawJsObject -> Json.Decode.Value
  // Elm.Kernel.Json.run : Json.Decode.Decoder a -> Json.Decode.Value -> Result Json.Decode.Error a
  const flagsResult = A2(_Json_run, flagDecoder, _Json_wrap(args ? args["flags"] : undefined));

  if (!$elm$core$Result$isOk(flagsResult)) {
    if (_Basics_isDebug) {
      _Debug_crash(2, _Json_errorToString(result.a));
    } else {
      _Debug_crash(2);
    }
  }

  let cmdSender;
  const ports = {};

  const dispatch = (model, cmds) => {
    _Platform_effectsQueue.push({
      a: cmds,
      b: impl.subscriptions(model),
    });

    if (_Platform_effectDispatchInProgress) {
      return;
    }

    _Platform_effectDispatchInProgress = true;
    while (true) {
      const fx = _Platform_effectsQueue.shift();
      if (fx === undefined) {
        _Platform_effectDispatchInProgress = false;
        return;
      }
      const tuple = A3(
        $elm$core$Platform$initializeHelperFunctions.dispatchEffects,
        fx.a,
        fx.b,
        cmdSender
      );
      tuple.a(sendToApp);
      $elm$core$Platform$Raw$Scheduler$rawSpawn(tuple.b);
    }
  };

  const sendToApp = F2((msg, viewMetadata) => {
    const updateValue = A2(impl.update, msg, model);
    model = updateValue.a;
    A2(stepper, model, viewMetadata);
    dispatch(model, updateValue.b);
  });

  for (const f of _Platform_runAfterLoadQueue) {
    f();
  }
  _Platform_runAfterLoadQueue = null;

  cmdSender = $elm$core$Platform$initializeHelperFunctions.setupEffectsChannel(sendToApp);

  for (const [key, { port }] of _Platform_outgoingPorts.entries()) {
    ports[key] = port;
  }
  for (const [key, { port }] of _Platform_incomingPorts.entries()) {
    ports[key] = port;
  }

  const initValue = impl.init(flagsResult.a);
  let model = initValue.a;
  const stepper = A2($elm$core$Platform$initializeHelperFunctions.stepperBuilder, sendToApp, model);

  dispatch(model, initValue.b);

  return ports ? { ports } : {};
});

// TRACK PRELOADS
//
// This is used by code in elm/browser and elm/http
// to register any HTTP requests that are triggered by init.
//

var _Platform_preload;

function _Platform_registerPreload(url) {
  _Platform_preload.add(url);
}

// EFFECT MANAGERS

function _Platform_createManager(init, onEffects, onSelfMsg, cmdMap, subMap) {
  _Debug_crash(12, _Debug_runtimeCrashReason("EffectModule"));
}

// BAGS

/* Called by compiler generated js for event managers for the
 * `command` or `subscription` function within an event manager
 */
const _Platform_leaf = (home) => (value) => {
  _Debug_crash(12, _Debug_runtimeCrashReason("PlatformLeaf", home));
};

// PORTS

function _Platform_checkPortName(name) {
  if (_Platform_outgoingPorts.has(name) || _Platform_incomingPorts.has(name)) {
    _Debug_crash(3, name);
  }
}

function _Platform_outgoingPort(name, converter) {
  _Platform_checkPortName(name);
  let subs = [];
  const subscribe = (callback) => {
    subs.push(callback);
  };
  const unsubscribe = (callback) => {
    // copy subs into a new array in case unsubscribe is called within
    // a subscribed callback
    subs = subs.slice();
    var index = subs.indexOf(callback);
    if (index >= 0) {
      subs.splice(index, 1);
    }
  };
  const execSubscribers = (payload) => {
    const value = _Json_unwrap(converter(payload));
    for (const sub of subs) {
      sub(value);
    }
    return _Utils_Tuple0;
  };
  _Platform_outgoingPorts.set(name, {
    port: {
      subscribe,
      unsubscribe,
    },
  });

  return (payload) =>
    _Platform_command(
      $elm$core$Platform$Scheduler$execImpure((_) => {
        execSubscribers(payload);
        return $elm$core$Maybe$Nothing;
      })
    );
}

function _Platform_incomingPort(name, converter) {
  _Platform_checkPortName(name);

  const tuple = _Platform_createSubProcess((_) => _Utils_Tuple0);
  const key = tuple.a;
  const sender = tuple.b;

  function send(incomingValue) {
    var result = A2(_Json_run, converter, _Json_wrap(incomingValue));

    $elm$core$Result$isOk(result) || _Debug_crash(4, name, result.a);

    var value = result.a;
    A2(_Channel_rawSend, sender, value);
  }

  _Platform_incomingPorts.set(name, {
    port: {
      send,
    },
  });

  return _Platform_subscription(key);
}

// Functions exported to elm

const _Platform_subscriptionStates = new Map();
let _Platform_subscriptionProcessIds = 0;

const _Platform_createSubProcess = (onSubUpdate) => {
  const channel = _Channel_rawUnbounded();
  const key = { id: _Platform_subscriptionProcessIds++ };
  const msgHandler = (hcst) =>
    $elm$core$Platform$Raw$Task$execImpure((_) => {
      const subscriptionState = _Platform_subscriptionStates.get(key);
      if (_Basics_isDebug && subscriptionState === undefined) {
        _Debug_crash(12, _Debug_runtimeCrashReason("subscriptionProcessMissing"), key && key.id);
      }
      for (const sendToApp of subscriptionState.listeners) {
        sendToApp(hcst);
      }
      return _Utils_Tuple0;
    });

  const onSubEffects = (_) =>
    A2($elm$core$Platform$Raw$Task$andThen, onSubEffects, A2($elm$core$Platform$Raw$Channel$recv, msgHandler, channel.b));

  _Platform_subscriptionStates.set(key, {
    listeners: [],
    onSubUpdate: onSubUpdate,
  });
  _Platform_runAfterLoad(() => $elm$core$Platform$Raw$Scheduler$rawSpawn(onSubEffects(_Utils_Tuple0)));

  return _Utils_Tuple2(key, channel.a);
};

const _Platform_resetSubscriptions = (newSubs) => {
  for (const subState of _Platform_subscriptionStates.values()) {
    subState.listeners.length = 0;
  }
  for (const tuple of _List_toArray(newSubs)) {
    const key = tuple.a;
    const sendToApp = tuple.b;
    const subState = _Platform_subscriptionStates.get(key);
    if (_Basics_isDebug && subState.listeners === undefined) {
      _Debug_crash(12, _Debug_runtimeCrashReason("subscriptionProcessMissing"), key && key.id);
    }
    subState.listeners.push(sendToApp);
  }
  for (const subState of _Platform_subscriptionStates.values()) {
    subState.onSubUpdate(subState.listeners.length);
  }
  return _Utils_Tuple0;
};

const _Platform_effectManagerNameToString = (name) => name;

const _Platform_wrapTask = (task) => $elm$core$Platform$Task(task);

const _Platform_wrapProcessId = (processId) => $elm$core$Platform$ProcessId(processId);

// command : Platform.Task Never (Maybe msg) -> Cmd msg
const _Platform_command = (task) => {
  const cmdData = _List_Cons(task, _List_Nil);
  if (_Basics_isDebug) {
    return {
      $: "Cmd",
      a: cmdData,
    };
  }
  return cmdData;
};

// subscription : RawSub.Id -> (RawSub.HiddenConvertedSubType -> msg) -> Sub msg
const _Platform_subscription = (id) => (tagger) => {
  const subData = _List_Cons(_Utils_Tuple2(id, tagger), _List_Nil);
  if (_Basics_isDebug) {
    return {
      $: "Sub",
      a: subData,
    };
  }
  return subData;
};

// EXPORT ELM MODULES
//
// Have DEBUG and PROD versions so that we can (1) give nicer errors in
// debug mode and (2) not pay for the bits needed for that in prod mode.
//

function _Platform_export_UNUSED(exports) {
  scope["Elm"] ? _Platform_mergeExportsProd(scope["Elm"], exports) : (scope["Elm"] = exports);
}

function _Platform_mergeExportsProd(obj, exports) {
  for (var name in exports) {
    name in obj
      ? name == "init"
        ? _Debug_crash(6)
        : _Platform_mergeExportsProd(obj[name], exports[name])
      : (obj[name] = exports[name]);
  }
}

function _Platform_export(exports) {
  scope["Elm"]
    ? _Platform_mergeExportsDebug("Elm", scope["Elm"], exports)
    : (scope["Elm"] = exports);
}

function _Platform_mergeExportsDebug(moduleName, obj, exports) {
  for (var name in exports) {
    name in obj
      ? name == "init"
        ? _Debug_crash(6, moduleName)
        : _Platform_mergeExportsDebug(moduleName + "." + name, obj[name], exports[name])
      : (obj[name] = exports[name]);
  }
}
var $elm$core$List$Cons_elm_builtin = F2(
	function (a, b) {
		return {$: 'Cons_elm_builtin', a: a, b: b};
	});
var $elm$core$Basics$EQ = {$: 'EQ'};
var $elm$core$Basics$LT = {$: 'LT'};
var $elm$core$List$Nil_elm_builtin = {$: 'Nil_elm_builtin'};
var $elm$core$List$cons = $elm$core$List$Cons_elm_builtin;
var $elm$core$Elm$JsArray$foldr = _JsArray_foldr;
var $elm$core$Array$foldr = F3(
	function (func, baseCase, _v0) {
		var tree = _v0.c;
		var tail = _v0.d;
		var helper = F2(
			function (node, acc) {
				if (node.$ === 'SubTree') {
					var subTree = node.a;
					return A3($elm$core$Elm$JsArray$foldr, helper, acc, subTree);
				} else {
					var values = node.a;
					return A3($elm$core$Elm$JsArray$foldr, func, acc, values);
				}
			});
		return A3(
			$elm$core$Elm$JsArray$foldr,
			helper,
			A3($elm$core$Elm$JsArray$foldr, func, baseCase, tail),
			tree);
	});
var $elm$core$Array$toList = function (array) {
	return A3($elm$core$Array$foldr, $elm$core$List$cons, _List_Nil, array);
};
var $elm$core$Dict$foldr = F3(
	function (func, acc, t) {
		foldr:
		while (true) {
			if (t.$ === 'RBEmpty_elm_builtin') {
				return acc;
			} else {
				var key = t.b;
				var value = t.c;
				var left = t.d;
				var right = t.e;
				var $temp$func = func,
					$temp$acc = A3(
					func,
					key,
					value,
					A3($elm$core$Dict$foldr, func, acc, right)),
					$temp$t = left;
				func = $temp$func;
				acc = $temp$acc;
				t = $temp$t;
				continue foldr;
			}
		}
	});
var $elm$core$Dict$toList = function (dict) {
	return A3(
		$elm$core$Dict$foldr,
		F3(
			function (key, value, list) {
				return A2(
					$elm$core$List$cons,
					_Utils_Tuple2(key, value),
					list);
			}),
		_List_Nil,
		dict);
};
var $elm$core$Dict$keys = function (dict) {
	return A3(
		$elm$core$Dict$foldr,
		F3(
			function (key, value, keyList) {
				return A2($elm$core$List$cons, key, keyList);
			}),
		_List_Nil,
		dict);
};
var $elm$core$Set$toList = function (_v0) {
	var dict = _v0.a;
	return $elm$core$Dict$keys(dict);
};
var $elm$core$Basics$add = F2(
	function (lhs, rhs) {
		var sum = lhs + rhs;
		return sum;
	});
var $elm$core$List$foldl = F3(
	function (func, acc, list) {
		foldl:
		while (true) {
			if (!list.b) {
				return acc;
			} else {
				var x = list.a;
				var xs = list.b;
				var $temp$func = func,
					$temp$acc = A2(func, x, acc),
					$temp$list = xs;
				func = $temp$func;
				acc = $temp$acc;
				list = $temp$list;
				continue foldl;
			}
		}
	});
var $elm$core$Basics$gt = F2(
	function (lhs, rhs) {
		var lhsLarger = _Utils_cmp(lhs, rhs) > 0;
		return lhsLarger;
	});
var $elm$core$List$reverse = function (list) {
	return A3($elm$core$List$foldl, $elm$core$List$cons, _List_Nil, list);
};
var $elm$core$List$foldrHelper = F4(
	function (fn, acc, ctr, ls) {
		if (!ls.b) {
			return acc;
		} else {
			var a = ls.a;
			var r1 = ls.b;
			if (!r1.b) {
				return A2(fn, a, acc);
			} else {
				var b = r1.a;
				var r2 = r1.b;
				if (!r2.b) {
					return A2(
						fn,
						a,
						A2(fn, b, acc));
				} else {
					var c = r2.a;
					var r3 = r2.b;
					if (!r3.b) {
						return A2(
							fn,
							a,
							A2(
								fn,
								b,
								A2(fn, c, acc)));
					} else {
						var d = r3.a;
						var r4 = r3.b;
						var res = (ctr > 500) ? A3(
							$elm$core$List$foldl,
							fn,
							acc,
							$elm$core$List$reverse(r4)) : A4($elm$core$List$foldrHelper, fn, acc, ctr + 1, r4);
						return A2(
							fn,
							a,
							A2(
								fn,
								b,
								A2(
									fn,
									c,
									A2(fn, d, res))));
					}
				}
			}
		}
	});
var $elm$core$List$foldr = F3(
	function (fn, acc, ls) {
		return A4($elm$core$List$foldrHelper, fn, acc, 0, ls);
	});
var $elm$core$List$append = F2(
	function (xs, ys) {
		if (!ys.b) {
			return xs;
		} else {
			return A3($elm$core$List$foldr, $elm$core$List$cons, ys, xs);
		}
	});
var $elm$core$Basics$identity = function (x) {
	return x;
};
var $elm$core$Platform$Cmd$Cmd = function (a) {
	return {$: 'Cmd', a: a};
};
var $elm$core$Basics$composeR = F3(
	function (f, g, x) {
		return g(
			f(x));
	});
var $elm$core$List$concat = function (lists) {
	return A3($elm$core$List$foldr, $elm$core$List$append, _List_Nil, lists);
};
var $elm$core$List$map = F2(
	function (f, xs) {
		return A3(
			$elm$core$List$foldr,
			F2(
				function (x, acc) {
					return A2(
						$elm$core$List$cons,
						f(x),
						acc);
				}),
			_List_Nil,
			xs);
	});
var $elm$core$Platform$Cmd$batch = A2(
	$elm$core$Basics$composeR,
	$elm$core$List$map(
		function (_v0) {
			var cmd = _v0.a;
			return cmd;
		}),
	A2($elm$core$Basics$composeR, $elm$core$List$concat, $elm$core$Platform$Cmd$Cmd));
var $elm$core$Platform$Cmd$none = $elm$core$Platform$Cmd$batch(_List_Nil);
var $elm$core$Platform$Sub$Sub = function (a) {
	return {$: 'Sub', a: a};
};
var $elm$core$Platform$Sub$batch = A2(
	$elm$core$Basics$composeR,
	$elm$core$List$map(
		function (_v0) {
			var sub = _v0.a;
			return sub;
		}),
	A2($elm$core$Basics$composeR, $elm$core$List$concat, $elm$core$Platform$Sub$Sub));
var $elm$core$Platform$Sub$none = $elm$core$Platform$Sub$batch(_List_Nil);
var $elm$core$Maybe$Just = function (a) {
	return {$: 'Just', a: a};
};
var $elm$core$Maybe$Nothing = {$: 'Nothing'};
var $elm$core$Result$Err = function (a) {
	return {$: 'Err', a: a};
};
var $elm$json$Json$Decode$Failure = F2(
	function (a, b) {
		return {$: 'Failure', a: a, b: b};
	});
var $elm$json$Json$Decode$Field = F2(
	function (a, b) {
		return {$: 'Field', a: a, b: b};
	});
var $elm$json$Json$Decode$Index = F2(
	function (a, b) {
		return {$: 'Index', a: a, b: b};
	});
var $elm$core$Result$Ok = function (a) {
	return {$: 'Ok', a: a};
};
var $elm$json$Json$Decode$OneOf = function (a) {
	return {$: 'OneOf', a: a};
};
var $elm$core$Basics$False = {$: 'False'};
var $elm$core$String$all = _String_all;
var $elm$core$Basics$and = F2(
	function (lhs, rhs) {
		var areBothTrue = lhs && rhs;
		return areBothTrue;
	});
var $elm$core$Basics$append = F2(
	function (lhs, rhs) {
		var appended = _Utils_ap(lhs, rhs);
		return appended;
	});
var $elm$json$Json$Encode$encode = _Json_encode;
var $elm$core$String$fromInt = _String_fromNumber;
var $elm$core$String$join = F2(
	function (sep, chunks) {
		return A2(
			_String_join,
			sep,
			_List_toArray(chunks));
	});
var $elm$core$String$split = F2(
	function (sep, string) {
		return _List_fromArray(
			A2(_String_split, sep, string));
	});
var $elm$json$Json$Decode$indent = function (str) {
	return A2(
		$elm$core$String$join,
		'\n    ',
		A2($elm$core$String$split, '\n', str));
};
var $elm$core$List$length = function (xs) {
	return A3(
		$elm$core$List$foldl,
		F2(
			function (_v0, i) {
				return i + 1;
			}),
		0,
		xs);
};
var $elm$core$List$map2Help = F4(
	function (f, xs1, xs2, ys) {
		map2Help:
		while (true) {
			var _v0 = _Utils_Tuple2(xs1, xs2);
			if (_v0.a.b && _v0.b.b) {
				var _v1 = _v0.a;
				var head1 = _v1.a;
				var rest1 = _v1.b;
				var _v2 = _v0.b;
				var head2 = _v2.a;
				var rest2 = _v2.b;
				var $temp$f = f,
					$temp$xs1 = rest1,
					$temp$xs2 = rest2,
					$temp$ys = A2(
					$elm$core$List$cons,
					A2(f, head1, head2),
					ys);
				f = $temp$f;
				xs1 = $temp$xs1;
				xs2 = $temp$xs2;
				ys = $temp$ys;
				continue map2Help;
			} else {
				return ys;
			}
		}
	});
var $elm$core$List$map2 = F3(
	function (f, xs1, xs2) {
		return $elm$core$List$reverse(
			A4($elm$core$List$map2Help, f, xs1, xs2, _List_Nil));
	});
var $elm$core$Basics$le = F2(
	function (lhs, rhs) {
		var lhsSmallerOrEqual = _Utils_cmp(lhs, rhs) < 1;
		return lhsSmallerOrEqual;
	});
var $elm$core$Basics$sub = F2(
	function (lhs, rhs) {
		var difference = lhs - rhs;
		return difference;
	});
var $elm$core$List$rangeHelp = F3(
	function (lo, hi, list) {
		rangeHelp:
		while (true) {
			if (_Utils_cmp(lo, hi) < 1) {
				var $temp$lo = lo,
					$temp$hi = hi - 1,
					$temp$list = A2($elm$core$List$cons, hi, list);
				lo = $temp$lo;
				hi = $temp$hi;
				list = $temp$list;
				continue rangeHelp;
			} else {
				return list;
			}
		}
	});
var $elm$core$List$range = F2(
	function (lo, hi) {
		return A3($elm$core$List$rangeHelp, lo, hi, _List_Nil);
	});
var $elm$core$List$indexedMap = F2(
	function (f, xs) {
		return A3(
			$elm$core$List$map2,
			f,
			A2(
				$elm$core$List$range,
				0,
				$elm$core$List$length(xs) - 1),
			xs);
	});
var $elm$core$Char$toCode = _Char_toCode;
var $elm$core$Char$isLower = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (97 <= code) && (code <= 122);
};
var $elm$core$Char$isUpper = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (code <= 90) && (65 <= code);
};
var $elm$core$Basics$or = F2(
	function (lhs, rhs) {
		var areEitherTrue = lhs || rhs;
		return areEitherTrue;
	});
var $elm$core$Char$isAlpha = function (_char) {
	return $elm$core$Char$isLower(_char) || $elm$core$Char$isUpper(_char);
};
var $elm$core$Char$isDigit = function (_char) {
	var code = $elm$core$Char$toCode(_char);
	return (code <= 57) && (48 <= code);
};
var $elm$core$Char$isAlphaNum = function (_char) {
	return $elm$core$Char$isLower(_char) || ($elm$core$Char$isUpper(_char) || $elm$core$Char$isDigit(_char));
};
var $elm$core$String$uncons = _String_uncons;
var $elm$json$Json$Decode$errorOneOf = F2(
	function (i, error) {
		return '\n\n(' + ($elm$core$String$fromInt(i + 1) + (') ' + $elm$json$Json$Decode$indent(
			$elm$json$Json$Decode$errorToString(error))));
	});
var $elm$json$Json$Decode$errorToString = function (error) {
	return A2($elm$json$Json$Decode$errorToStringHelp, error, _List_Nil);
};
var $elm$json$Json$Decode$errorToStringHelp = F2(
	function (error, context) {
		errorToStringHelp:
		while (true) {
			switch (error.$) {
				case 'Field':
					var f = error.a;
					var err = error.b;
					var isSimple = function () {
						var _v1 = $elm$core$String$uncons(f);
						if (_v1.$ === 'Nothing') {
							return false;
						} else {
							var _v2 = _v1.a;
							var _char = _v2.a;
							var rest = _v2.b;
							return $elm$core$Char$isAlpha(_char) && A2($elm$core$String$all, $elm$core$Char$isAlphaNum, rest);
						}
					}();
					var fieldName = isSimple ? ('.' + f) : ('[\'' + (f + '\']'));
					var $temp$error = err,
						$temp$context = A2($elm$core$List$cons, fieldName, context);
					error = $temp$error;
					context = $temp$context;
					continue errorToStringHelp;
				case 'Index':
					var i = error.a;
					var err = error.b;
					var indexName = '[' + ($elm$core$String$fromInt(i) + ']');
					var $temp$error = err,
						$temp$context = A2($elm$core$List$cons, indexName, context);
					error = $temp$error;
					context = $temp$context;
					continue errorToStringHelp;
				case 'OneOf':
					var errors = error.a;
					if (!errors.b) {
						return 'Ran into a Json.Decode.oneOf with no possibilities' + function () {
							if (!context.b) {
								return '!';
							} else {
								return ' at json' + A2(
									$elm$core$String$join,
									'',
									$elm$core$List$reverse(context));
							}
						}();
					} else {
						if (!errors.b.b) {
							var err = errors.a;
							var $temp$error = err,
								$temp$context = context;
							error = $temp$error;
							context = $temp$context;
							continue errorToStringHelp;
						} else {
							var starter = function () {
								if (!context.b) {
									return 'Json.Decode.oneOf';
								} else {
									return 'The Json.Decode.oneOf at json' + A2(
										$elm$core$String$join,
										'',
										$elm$core$List$reverse(context));
								}
							}();
							var introduction = starter + (' failed in the following ' + ($elm$core$String$fromInt(
								$elm$core$List$length(errors)) + ' ways:'));
							return A2(
								$elm$core$String$join,
								'\n\n',
								A2(
									$elm$core$List$cons,
									introduction,
									A2($elm$core$List$indexedMap, $elm$json$Json$Decode$errorOneOf, errors)));
						}
					}
				default:
					var msg = error.a;
					var json = error.b;
					var introduction = function () {
						if (!context.b) {
							return 'Problem with the given value:\n\n';
						} else {
							return 'Problem with the value at json' + (A2(
								$elm$core$String$join,
								'',
								$elm$core$List$reverse(context)) + ':\n\n    ');
						}
					}();
					return introduction + ($elm$json$Json$Decode$indent(
						A2($elm$json$Json$Encode$encode, 4, json)) + ('\n\n' + msg));
			}
		}
	});
var $elm$core$Array$branchFactor = 32;
var $elm$core$Array$Array_elm_builtin = F4(
	function (a, b, c, d) {
		return {$: 'Array_elm_builtin', a: a, b: b, c: c, d: d};
	});
var $elm$core$Elm$JsArray$empty = _JsArray_empty;
var $elm$core$Basics$ceiling = _Basics_ceiling;
var $elm$core$Basics$fdiv = F2(
	function (lhs, rhs) {
		var quotient = lhs / rhs;
		return quotient;
	});
var $elm$core$Basics$logBase = F2(
	function (base, number) {
		return _Basics_log(number) / _Basics_log(base);
	});
var $elm$core$Basics$toFloat = function (x) {
	var asFloat = x;
	return asFloat;
};
var $elm$core$Array$shiftStep = $elm$core$Basics$ceiling(
	A2($elm$core$Basics$logBase, 2, $elm$core$Array$branchFactor));
var $elm$core$Array$empty = A4($elm$core$Array$Array_elm_builtin, 0, $elm$core$Array$shiftStep, $elm$core$Elm$JsArray$empty, $elm$core$Elm$JsArray$empty);
var $elm$core$Elm$JsArray$initialize = _JsArray_initialize;
var $elm$core$Array$Leaf = function (a) {
	return {$: 'Leaf', a: a};
};
var $elm$core$Basics$apL = F2(
	function (f, x) {
		var applied = f(x);
		return applied;
	});
var $elm$core$Basics$apR = F2(
	function (x, f) {
		var applied = f(x);
		return applied;
	});
var $elm$core$Basics$eq = F2(
	function (lhs, rhs) {
		var areEqual = _Utils_eq(lhs, rhs);
		return areEqual;
	});
var $elm$core$Basics$floor = _Basics_floor;
var $elm$core$Elm$JsArray$length = _JsArray_length;
var $elm$core$Basics$max = F2(
	function (x, y) {
		return (_Utils_cmp(x, y) > 0) ? x : y;
	});
var $elm$core$Basics$mul = F2(
	function (lhs, rhs) {
		var product = lhs * rhs;
		return product;
	});
var $elm$core$Array$SubTree = function (a) {
	return {$: 'SubTree', a: a};
};
var $elm$core$Elm$JsArray$initializeFromList = _JsArray_initializeFromList;
var $elm$core$Array$compressNodes = F2(
	function (nodes, acc) {
		compressNodes:
		while (true) {
			var _v0 = A2($elm$core$Elm$JsArray$initializeFromList, $elm$core$Array$branchFactor, nodes);
			var node = _v0.a;
			var remainingNodes = _v0.b;
			var newAcc = A2(
				$elm$core$List$cons,
				$elm$core$Array$SubTree(node),
				acc);
			if (!remainingNodes.b) {
				return $elm$core$List$reverse(newAcc);
			} else {
				var $temp$nodes = remainingNodes,
					$temp$acc = newAcc;
				nodes = $temp$nodes;
				acc = $temp$acc;
				continue compressNodes;
			}
		}
	});
var $elm$core$Tuple$first = function (_v0) {
	var x = _v0.a;
	return x;
};
var $elm$core$Array$treeFromBuilder = F2(
	function (nodeList, nodeListSize) {
		treeFromBuilder:
		while (true) {
			var newNodeSize = $elm$core$Basics$ceiling(nodeListSize / $elm$core$Array$branchFactor);
			if (newNodeSize === 1) {
				return A2($elm$core$Elm$JsArray$initializeFromList, $elm$core$Array$branchFactor, nodeList).a;
			} else {
				var $temp$nodeList = A2($elm$core$Array$compressNodes, nodeList, _List_Nil),
					$temp$nodeListSize = newNodeSize;
				nodeList = $temp$nodeList;
				nodeListSize = $temp$nodeListSize;
				continue treeFromBuilder;
			}
		}
	});
var $elm$core$Array$builderToArray = F2(
	function (reverseNodeList, builder) {
		if (!builder.nodeListSize) {
			return A4(
				$elm$core$Array$Array_elm_builtin,
				$elm$core$Elm$JsArray$length(builder.tail),
				$elm$core$Array$shiftStep,
				$elm$core$Elm$JsArray$empty,
				builder.tail);
		} else {
			var treeLen = builder.nodeListSize * $elm$core$Array$branchFactor;
			var depth = $elm$core$Basics$floor(
				A2($elm$core$Basics$logBase, $elm$core$Array$branchFactor, treeLen - 1));
			var correctNodeList = reverseNodeList ? $elm$core$List$reverse(builder.nodeList) : builder.nodeList;
			var tree = A2($elm$core$Array$treeFromBuilder, correctNodeList, builder.nodeListSize);
			return A4(
				$elm$core$Array$Array_elm_builtin,
				$elm$core$Elm$JsArray$length(builder.tail) + treeLen,
				A2($elm$core$Basics$max, 5, depth * $elm$core$Array$shiftStep),
				tree,
				builder.tail);
		}
	});
var $elm$core$Basics$idiv = F2(
	function (lhs, rhs) {
		var quotient = (lhs / rhs) | 0;
		return quotient;
	});
var $elm$core$Basics$lt = F2(
	function (lhs, rhs) {
		var lhsSmaller = _Utils_cmp(lhs, rhs) < 0;
		return lhsSmaller;
	});
var $elm$core$Array$initializeHelp = F5(
	function (fn, fromIndex, len, nodeList, tail) {
		initializeHelp:
		while (true) {
			if (fromIndex < 0) {
				return A2(
					$elm$core$Array$builderToArray,
					false,
					{nodeList: nodeList, nodeListSize: (len / $elm$core$Array$branchFactor) | 0, tail: tail});
			} else {
				var leaf = $elm$core$Array$Leaf(
					A3($elm$core$Elm$JsArray$initialize, $elm$core$Array$branchFactor, fromIndex, fn));
				var $temp$fn = fn,
					$temp$fromIndex = fromIndex - $elm$core$Array$branchFactor,
					$temp$len = len,
					$temp$nodeList = A2($elm$core$List$cons, leaf, nodeList),
					$temp$tail = tail;
				fn = $temp$fn;
				fromIndex = $temp$fromIndex;
				len = $temp$len;
				nodeList = $temp$nodeList;
				tail = $temp$tail;
				continue initializeHelp;
			}
		}
	});
var $elm$core$Basics$remainderBy = F2(
	function (divisor, dividend) {
		var remainder = dividend % divisor;
		return remainder;
	});
var $elm$core$Array$initialize = F2(
	function (len, fn) {
		if (len <= 0) {
			return $elm$core$Array$empty;
		} else {
			var tailLen = len % $elm$core$Array$branchFactor;
			var tail = A3($elm$core$Elm$JsArray$initialize, tailLen, len - tailLen, fn);
			var initialFromIndex = (len - tailLen) - $elm$core$Array$branchFactor;
			return A5($elm$core$Array$initializeHelp, fn, initialFromIndex, len, _List_Nil, tail);
		}
	});
var $elm$core$Basics$True = {$: 'True'};
var $elm$core$Result$isOk = function (result) {
	if (result.$ === 'Ok') {
		return true;
	} else {
		return false;
	}
};
var $elm$core$Platform$ProcessId = function (a) {
	return {$: 'ProcessId', a: a};
};
var $elm$core$Platform$Task = function (a) {
	return {$: 'Task', a: a};
};
var $elm$core$Platform$Raw$Task$AsyncAction = function (a) {
	return {$: 'AsyncAction', a: a};
};
var $elm$core$Platform$Raw$Task$andThen = F2(
	function (func, task) {
		if (task.$ === 'Value') {
			var val = task.a;
			return func(val);
		} else {
			var fut = task.a;
			return $elm$core$Platform$Raw$Task$AsyncAction(
				{
					then_: function (callback) {
						return fut.then_(
							function (newTask) {
								return callback(
									A2($elm$core$Platform$Raw$Task$andThen, func, newTask));
							});
					}
				});
		}
	});
var $elm$core$Platform$Raw$Task$Value = function (a) {
	return {$: 'Value', a: a};
};
var $elm$core$Platform$Raw$Impure$unwrapFunction = _Basics_fudgeType;
var $elm$core$Platform$Raw$Task$execImpure = function (func) {
	return $elm$core$Platform$Raw$Task$AsyncAction(
		{
			then_: function (callback) {
				var _v0 = callback(
					$elm$core$Platform$Raw$Task$Value(
						A2($elm$core$Platform$Raw$Impure$unwrapFunction, func, _Utils_Tuple0)));
				return function (_v1) {
					return _Utils_Tuple0;
				};
			}
		});
};
var $elm$core$Platform$Scheduler$unwrapTask = _Basics_unwrapTypeWrapper;
var $elm$core$Platform$Scheduler$taskFn = F2(
	function (fn, task) {
		return fn(
			$elm$core$Platform$Scheduler$unwrapTask(task));
	});
var $elm$core$Platform$Scheduler$wrapTask = _Platform_wrapTask;
var $elm$core$Platform$Scheduler$binding = function (fut) {
	return $elm$core$Platform$Scheduler$wrapTask(
		$elm$core$Platform$Raw$Task$AsyncAction(
			{
				then_: function (doneCallback) {
					return fut.then_(
						$elm$core$Platform$Scheduler$taskFn(
							function (task) {
								return doneCallback(task);
							}));
				}
			}));
};
var $elm$core$Platform$Scheduler$succeed = function (val) {
	return $elm$core$Platform$Scheduler$wrapTask(
		$elm$core$Platform$Raw$Task$Value(
			$elm$core$Result$Ok(val)));
};
var $elm$core$Platform$Scheduler$execImpure = function (func) {
	return $elm$core$Platform$Scheduler$binding(
		{
			then_: function (doneCallback) {
				var _v0 = doneCallback(
					$elm$core$Platform$Scheduler$succeed(
						func(_Utils_Tuple0)));
				return function (_v1) {
					return _Utils_Tuple0;
				};
			}
		});
};
var $elm$core$Platform$AsyncUpdate = {$: 'AsyncUpdate'};
var $elm$core$Platform$Raw$Impure$function = _Basics_fudgeType;
var $elm$core$Platform$Raw$Impure$propagate = F2(
	function (f, b) {
		return $elm$core$Platform$Raw$Impure$function(
			function (a) {
				return A2(
					$elm$core$Platform$Raw$Impure$unwrapFunction,
					f(a),
					b);
			});
	});
var $elm$core$Platform$resetSubscriptions = _Platform_resetSubscriptions;
var $elm$core$Platform$Raw$Channel$rawSend = _Channel_rawSend;
var $elm$core$Platform$Raw$Channel$send = F2(
	function (channelId, msg) {
		return $elm$core$Platform$Raw$Task$execImpure(
			$elm$core$Platform$Raw$Impure$function(
				function (_v0) {
					return A2($elm$core$Platform$Raw$Channel$rawSend, channelId, msg);
				}));
	});
var $elm$core$Platform$Raw$Impure$andThen = F2(
	function (ip2, ip1) {
		return $elm$core$Platform$Raw$Impure$function(
			function (a) {
				var b = A2($elm$core$Platform$Raw$Impure$unwrapFunction, ip1, a);
				return A2($elm$core$Platform$Raw$Impure$unwrapFunction, ip2, b);
			});
	});
var $elm$core$Platform$Raw$Impure$toThunk = F2(
	function (f, x) {
		return A2(
			$elm$core$Platform$Raw$Impure$andThen,
			f,
			$elm$core$Platform$Raw$Impure$function(
				function (_v0) {
					return x;
				}));
	});
var $elm$core$Platform$unwrapCmd = _Basics_unwrapTypeWrapper;
var $elm$core$Platform$unwrapSub = _Basics_unwrapTypeWrapper;
var $elm$core$Platform$dispatchEffects = F2(
	function (cmdBag, subBag) {
		var subs = $elm$core$Platform$unwrapSub(subBag);
		var cmds = $elm$core$Platform$unwrapCmd(cmdBag);
		return function (channel) {
			var updateSubs = A2(
				$elm$core$Platform$Raw$Impure$propagate,
				function (sendToAppFunc) {
					var thunks = A2(
						$elm$core$List$map,
						function (_v0) {
							var id = _v0.a;
							var tagger = _v0.b;
							return _Utils_Tuple2(
								id,
								A2(
									$elm$core$Platform$Raw$Impure$propagate,
									function (v) {
										return sendToAppFunc(
											tagger(v));
									},
									$elm$core$Platform$AsyncUpdate));
						},
						subs);
					return A2($elm$core$Platform$Raw$Impure$toThunk, $elm$core$Platform$resetSubscriptions, thunks);
				},
				_Utils_Tuple0);
			return _Utils_Tuple2(
				updateSubs,
				A2($elm$core$Platform$Raw$Channel$send, channel, cmds));
		};
	});
var $elm$core$Platform$Raw$Scheduler$ProcessId = function (a) {
	return {$: 'ProcessId', a: a};
};
var $elm$core$Platform$Raw$Scheduler$Ready = function (a) {
	return {$: 'Ready', a: a};
};
var $elm$core$Platform$Raw$Scheduler$Running = function (a) {
	return {$: 'Running', a: a};
};
var $elm$core$Platform$Raw$Scheduler$enqueueWithStepper = _Scheduler_enqueueWithStepper;
var $elm$core$Platform$Raw$Scheduler$stepper = F2(
	function (processId, root) {
		if (root.$ === 'Value') {
			var val = root.a;
			return $elm$core$Platform$Raw$Scheduler$Ready(
				$elm$core$Platform$Raw$Task$Value(val));
		} else {
			var doEffect = root.a;
			return $elm$core$Platform$Raw$Scheduler$Running(
				doEffect.then_(
					function (newRoot) {
						var _v1 = A2(
							$elm$core$Platform$Raw$Scheduler$cyclic$enqueue(),
							processId,
							newRoot);
						return _Utils_Tuple0;
					}));
		}
	});
function $elm$core$Platform$Raw$Scheduler$cyclic$enqueue() {
	return $elm$core$Platform$Raw$Scheduler$enqueueWithStepper($elm$core$Platform$Raw$Scheduler$stepper);
}
try {
	var $elm$core$Platform$Raw$Scheduler$enqueue = $elm$core$Platform$Raw$Scheduler$cyclic$enqueue();
	$elm$core$Platform$Raw$Scheduler$cyclic$enqueue = function () {
		return $elm$core$Platform$Raw$Scheduler$enqueue;
	};
} catch ($) {
	throw 'Some top-level definitions from `Platform.Raw.Scheduler` are causing infinite recursion:\n\n  ┌─────┐\n  │    enqueue\n  │     ↓\n  │    stepper\n  └─────┘\n\nThese errors are very tricky, so read https://elm-lang.org/0.19.1/bad-recursion to learn how to fix it!';}
var $elm$core$Platform$Raw$Scheduler$getGuid = _Scheduler_getGuid;
var $elm$core$Platform$Raw$Scheduler$rawSpawn = function (initTask) {
	return A2(
		$elm$core$Platform$Raw$Scheduler$enqueue,
		$elm$core$Platform$Raw$Scheduler$ProcessId(
			{
				id: $elm$core$Platform$Raw$Scheduler$getGuid(_Utils_Tuple0)
			}),
		initTask);
};
var $elm$core$Platform$Scheduler$wrapProcessId = _Platform_wrapProcessId;
var $elm$core$Platform$Scheduler$rawSpawn = $elm$core$Platform$Scheduler$taskFn(
	function (task) {
		return $elm$core$Platform$Scheduler$wrapProcessId(
			$elm$core$Platform$Raw$Scheduler$rawSpawn(task));
	});
var $elm$core$Platform$Raw$Scheduler$getProcessState = _Scheduler_getProcessState;
var $elm$core$Platform$Raw$Scheduler$rawKill = function (id) {
	var _v0 = $elm$core$Platform$Raw$Scheduler$getProcessState(id);
	if (_v0.$ === 'Just') {
		if (_v0.a.$ === 'Running') {
			var killer = _v0.a.a;
			return killer(_Utils_Tuple0);
		} else {
			return _Utils_Tuple0;
		}
	} else {
		return _Utils_Tuple0;
	}
};
var $elm$core$Platform$Raw$Scheduler$spawn = function (task) {
	return $elm$core$Platform$Raw$Task$execImpure(
		$elm$core$Platform$Raw$Impure$function(
			function (_v0) {
				return $elm$core$Platform$Raw$Scheduler$rawSpawn(task);
			}));
};
var $elm$core$Platform$Raw$Scheduler$batch = function (ids) {
	return $elm$core$Platform$Raw$Scheduler$spawn(
		$elm$core$Platform$Raw$Task$AsyncAction(
			{
				then_: function (doneCallback) {
					var _v0 = doneCallback(
						$elm$core$Platform$Raw$Scheduler$spawn(
							$elm$core$Platform$Raw$Task$Value(_Utils_Tuple0)));
					return function (_v1) {
						return A3(
							$elm$core$List$foldr,
							F2(
								function (id, _v2) {
									return $elm$core$Platform$Raw$Scheduler$rawKill(id);
								}),
							_Utils_Tuple0,
							ids);
					};
				}
			}));
};
var $elm$core$Platform$Raw$Task$map = function (func) {
	return $elm$core$Platform$Raw$Task$andThen(
		function (x) {
			return $elm$core$Platform$Raw$Task$Value(
				func(x));
		});
};
var $elm$core$Basics$never = function (_v0) {
	never:
	while (true) {
		var nvr = _v0.a;
		var $temp$_v0 = nvr;
		_v0 = $temp$_v0;
		continue never;
	}
};
var $elm$core$Platform$Raw$Channel$rawUnbounded = _Channel_rawUnbounded;
var $elm$core$Platform$Raw$Channel$rawRecv = _Channel_rawRecv;
var $elm$core$Platform$Raw$Channel$recv = F2(
	function (tagger, chl) {
		return $elm$core$Platform$Raw$Task$AsyncAction(
			{
				then_: function (doneCallback) {
					return A2(
						$elm$core$Platform$Raw$Channel$rawRecv,
						chl,
						function (msg) {
							return doneCallback(
								tagger(msg));
						});
				}
			});
	});
var $elm$core$Tuple$second = function (_v0) {
	var y = _v0.b;
	return y;
};
var $elm$core$Platform$Raw$Task$delay = _Scheduler_delay;
var $elm$core$Platform$Raw$Task$sleep = function (time) {
	return $elm$core$Platform$Raw$Task$AsyncAction(
		A2(
			$elm$core$Platform$Raw$Task$delay,
			time,
			$elm$core$Platform$Raw$Task$Value(_Utils_Tuple0)));
};
var $elm$core$Platform$setupEffectsChannel = function (sendToApp2) {
	var receiveMsg = function (cmds) {
		var cmdTask = A2(
			$elm$core$Platform$Raw$Task$andThen,
			$elm$core$Platform$Raw$Scheduler$batch,
			A3(
				$elm$core$List$foldr,
				F2(
					function (curr, accTask) {
						return A2(
							$elm$core$Platform$Raw$Task$andThen,
							function (acc) {
								return A2(
									$elm$core$Platform$Raw$Task$map,
									function (id) {
										return A2($elm$core$List$cons, id, acc);
									},
									curr);
							},
							accTask);
					}),
				$elm$core$Platform$Raw$Task$Value(_List_Nil),
				A2(
					$elm$core$List$map,
					$elm$core$Platform$Raw$Scheduler$spawn,
					A2(
						$elm$core$List$map,
						$elm$core$Platform$Raw$Task$map(
							function (r) {
								if (r.$ === 'Ok') {
									if (r.a.$ === 'Just') {
										var msg = r.a.a;
										return A2(
											$elm$core$Platform$Raw$Impure$unwrapFunction,
											sendToApp2(msg),
											$elm$core$Platform$AsyncUpdate);
									} else {
										var _v5 = r.a;
										return _Utils_Tuple0;
									}
								} else {
									var err = r.a;
									return $elm$core$Basics$never(err);
								}
							}),
						A2(
							$elm$core$List$map,
							function (_v3) {
								var t = _v3.a;
								return t;
							},
							cmds)))));
		return A2(
			$elm$core$Platform$Raw$Task$map,
			function (_v2) {
				return _Utils_Tuple0;
			},
			cmdTask);
	};
	var dispatchChannel = $elm$core$Platform$Raw$Channel$rawUnbounded(_Utils_Tuple0);
	var dispatchTask = function (_v0) {
		return A2(
			$elm$core$Platform$Raw$Task$andThen,
			dispatchTask,
			A2($elm$core$Platform$Raw$Channel$recv, receiveMsg, dispatchChannel.b));
	};
	var _v1 = $elm$core$Platform$Raw$Scheduler$rawSpawn(
		A2(
			$elm$core$Platform$Raw$Task$andThen,
			dispatchTask,
			$elm$core$Platform$Raw$Task$sleep(0)));
	return dispatchChannel.a;
};
var $elm$core$Platform$initializeHelperFunctions = {
	dispatchEffects: $elm$core$Platform$dispatchEffects,
	setupEffectsChannel: $elm$core$Platform$setupEffectsChannel,
	stepperBuilder: F2(
		function (_v0, _v1) {
			return F2(
				function (_v2, _v3) {
					return _Utils_Tuple0;
				});
		})
};
var $elm$core$Platform$initialize = _Platform_initialize;
var $elm$core$Platform$makeProgram = _Basics_fudgeType;
var $elm$core$Platform$worker = function (impl) {
	return $elm$core$Platform$makeProgram(
		F3(
			function (flagsDecoder, _v0, args) {
				return A3($elm$core$Platform$initialize, flagsDecoder, args, impl);
			}));
};
var $author$project$Util$Programs$sendCmd = function (cmd) {
	return $elm$core$Platform$worker(
		{
			init: function (_v0) {
				return _Utils_Tuple2(_Utils_Tuple0, cmd);
			},
			subscriptions: function (_v1) {
				return $elm$core$Platform$Sub$none;
			},
			update: F2(
				function (_v2, _v3) {
					return _Utils_Tuple2(_Utils_Tuple0, $elm$core$Platform$Cmd$none);
				})
		});
};
var $elm$json$Json$Encode$string = _Json_wrap;
var $author$project$Util$Cmds$write = _Platform_outgoingPort('write', $elm$json$Json$Encode$string);
var $author$project$Util$Programs$print = function (string) {
	return $author$project$Util$Programs$sendCmd(
		$author$project$Util$Cmds$write(string));
};
var $elm$json$Json$Decode$succeed = _Json_succeed;
var $author$project$Main$main = $author$project$Util$Programs$print('Hello World!');
_Platform_export({'Main':{'init':$author$project$Main$main(
	$elm$json$Json$Decode$succeed(_Utils_Tuple0))(0)}});}(this));