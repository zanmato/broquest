use super::completion::ScriptContext;
use super::variable_store::VariableStore;
use crate::domain::{KeyValuePair, RequestData, ResponseData, ScriptDialect};
use anyhow::Result;
use rquickjs::{Context, Ctx, Error, Function, Object, Runtime};
use tracing::{debug, error};

// LLRT modules for buffer, crypto, and URL support
use llrt_modules::{buffer, crypto, url};

/// JavaScript prelude that layers Bruno's `bru`/`req`/`res` method API and a
/// minimal `test`/`expect` on top of broquest's native `bro`/`req`/`res`
/// objects, then aliases `bru` to `bro`. Injected before every user script so
/// OpenCollection scripts written against Bruno's API run unchanged.
const BRUNO_COMPAT_PRELUDE: &str = r#"
(function () {
  var G = globalThis;

  function findKey(obj, name) {
    if (!obj) return undefined;
    var target = String(name).toLowerCase();
    return Object.keys(obj).find(function (k) { return k.toLowerCase() === target; });
  }

  if (typeof req !== 'undefined') {
    req.getUrl = function () { return req.url; };
    req.setUrl = function (u) { req.url = u; };
    req.getMethod = function () { return req.method; };
    req.setMethod = function (m) { req.method = m; };
    req.getName = function () { return req.name; };
    req.getBody = function (opts) {
      if (opts && opts.raw) return req.body;
      try { return JSON.parse(req.body); } catch (e) { return req.body; }
    };
    req.setBody = function (b) { req.body = (typeof b === 'string') ? b : JSON.stringify(b); };
    req.getHeaders = function () { return req.headers; };
    req.getHeader = function (n) { var k = findKey(req.headers, n); return k ? req.headers[k] : undefined; };
    req.setHeader = function (n, v) { var k = findKey(req.headers, n); req.headers[k || n] = v; };
    req.setHeaders = function (o) { for (var k in o) req.setHeader(k, o[k]); };
    req.deleteHeader = function (n) { var k = findKey(req.headers, n); if (k) delete req.headers[k]; };
    req.deleteHeaders = function (names) { (names || []).forEach(function (n) { req.deleteHeader(n); }); };
    req.getTimeout = function () { return req.timeout; };
    req.setTimeout = function (t) { req.timeout = t; };
    req.getExecutionMode = function () { return 'standalone'; };
    req.getExecutionPlatform = function () { return 'app'; };
  }

  if (typeof res !== 'undefined') {
    res.getStatus = function () { return res.status; };
    res.getStatusText = function () { return res.statusText; };
    res.getHeaders = function () { return res.headers; };
    res.getHeader = function (n) { var k = findKey(res.headers, n); return k ? res.headers[k] : undefined; };
    res.getBody = function () { return res.body; };
    res.setBody = function (b) { res.body = b; };
    res.getResponseTime = function () { return res.responseTime; };
    res.getUrl = function () { return res.url; };
    res.getSize = function () { return { body: res.size, headers: 0, total: res.size }; };
  }

  if (typeof bro !== 'undefined') {
    // Typed runtime variables over the JSON string boundary.
    bro.getVar = function (n) { var s = bro.__getVar(n); return s === undefined ? undefined : JSON.parse(s); };
    bro.setVar = function (n, v) { bro.__setVar(n, JSON.stringify(v === undefined ? null : v)); };
    bro.getAllVars = function () { return JSON.parse(bro.__getAllVars()); };
    bro.getAllEnvVars = function () { return JSON.parse(bro.__getAllEnvVars()); };

    // Collection-level variables (read-only, declared on the collection).
    bro.getCollectionVar = function (n) {
      var s = bro.__getCollectionVar(n);
      return s === undefined ? undefined : JSON.parse(s);
    };
    bro.hasCollectionVar = function (n) { return bro.__hasCollectionVar(n); };

    // Request-level variables (read-only, declared on the request).
    bro.getRequestVar = function (n) {
      var s = bro.__getRequestVar(n);
      return s === undefined ? undefined : JSON.parse(s);
    };
    bro.hasRequestVar = function (n) { return bro.__hasRequestVar(n); };

    // Folder vars are not modeled by broquest yet. No-op stubs so scripts don't throw.
    bro.getFolderVar = function (_n) { return undefined; };
    bro.hasFolderVar = function (_n) { return false; };

    // Simple {{var}} interpolation (dynamic {{$...}} vars are left untouched).
    // Precedence: runtime > request > collection > environment (matches Bruno).
    bro.interpolate = function (str) {
      return String(str).replace(/\{\{\s*([^}]+?)\s*\}\}/g, function (m, name) {
        if (name[0] === '$') return m;
        var v = bro.getVar(name);
        if (v === undefined || v === null) v = bro.getRequestVar(name);
        if (v === undefined || v === null) v = bro.getCollectionVar(name);
        if (v === undefined || v === null) v = bro.getEnvVar(name);
        return (v === undefined || v === null) ? m : v;
      });
    };

    G.bru = bro;
  }

  // Minimal test() + chai-style expect().
  G.__testResults = G.__testResults || [];
  G.test = function (name, fn) {
    try { fn(); G.__testResults.push({ name: name, status: 'pass' }); }
    catch (e) {
      G.__testResults.push({ name: name, status: 'fail', error: String(e && e.message || e) });
      if (typeof console !== 'undefined') console.error('Test failed: ' + name + ' - ' + (e && e.message || e));
    }
  };

  G.expect = function (actual) {
    var flags = { neg: false, deep: false };
    function fail(msg) { throw new Error((flags.neg ? 'expected not ' : 'expected ') + msg); }
    function deepEq(a, b) { return JSON.stringify(a) === JSON.stringify(b); }
    function check(cond, msg) { if (flags.neg ? cond : !cond) fail(msg); }
    var api = {};
    ['to', 'be', 'been', 'is', 'that', 'and', 'has', 'have', 'with', 'of', 'same', 'which'].forEach(function (w) {
      Object.defineProperty(api, w, { get: function () { return api; } });
    });
    Object.defineProperty(api, 'not', { get: function () { flags.neg = !flags.neg; return api; } });
    Object.defineProperty(api, 'deep', { get: function () { flags.deep = true; return api; } });
    Object.defineProperty(api, 'ok', { get: function () { check(!!actual, 'to be truthy'); return api; } });
    Object.defineProperty(api, 'true', { get: function () { check(actual === true, 'to be true'); return api; } });
    Object.defineProperty(api, 'false', { get: function () { check(actual === false, 'to be false'); return api; } });
    Object.defineProperty(api, 'null', { get: function () { check(actual === null, 'to be null'); return api; } });
    Object.defineProperty(api, 'undefined', { get: function () { check(actual === undefined, 'to be undefined'); return api; } });
    Object.defineProperty(api, 'exist', { get: function () { check(actual !== null && actual !== undefined, 'to exist'); return api; } });
    Object.defineProperty(api, 'empty', {
      get: function () {
        var e = actual == null || actual.length === 0 || (typeof actual === 'object' && Object.keys(actual).length === 0);
        check(e, 'to be empty'); return api;
      }
    });
    api.equal = api.equals = api.eq = function (v) { check(flags.deep ? deepEq(actual, v) : actual === v, 'to equal ' + JSON.stringify(v)); return api; };
    api.eql = function (v) { check(deepEq(actual, v), 'to deeply equal ' + JSON.stringify(v)); return api; };
    api.a = api.an = function (t) { var ty = Array.isArray(actual) ? 'array' : (actual === null ? 'null' : typeof actual); check(ty === t, 'to be a ' + t); return api; };
    api.include = api.includes = api.contain = function (v) {
      var ok = typeof actual === 'string' ? actual.indexOf(v) >= 0
        : Array.isArray(actual) ? actual.indexOf(v) >= 0
          : (actual && Object.prototype.hasOwnProperty.call(actual, v));
      check(ok, 'to include ' + JSON.stringify(v)); return api;
    };
    api.property = function (k, v) {
      var has = actual != null && Object.prototype.hasOwnProperty.call(actual, k);
      check(has, 'to have property ' + k);
      if (arguments.length > 1) check(actual[k] === v, 'property ' + k + ' to equal ' + JSON.stringify(v));
      return api;
    };
    api.above = api.greaterThan = api.gt = function (n) { check(actual > n, 'to be above ' + n); return api; };
    api.below = api.lessThan = api.lt = function (n) { check(actual < n, 'to be below ' + n); return api; };
    api.least = api.gte = function (n) { check(actual >= n, 'to be at least ' + n); return api; };
    api.most = api.lte = function (n) { check(actual <= n, 'to be at most ' + n); return api; };
    api.lengthOf = api.length = function (n) { check(actual != null && actual.length === n, 'to have length ' + n); return api; };
    api.match = function (re) { check(re.test(actual), 'to match ' + re); return api; };
    return api;
  };
})();
"#;

/// JavaScript prelude for scripts written against the JetBrains HTTP Client,
/// as in `.http` files: `client`, `request`, `response` and `$random`, mapped
/// onto broquest's `bro`, `req` and `res`. Variables set with
/// `client.global.set` or `request.variables.set` are session variables.
const JETBRAINS_PRELUDE: &str = r#"
(function () {
  var G = globalThis;

  function log() {
    var text = Array.prototype.map.call(arguments, function (a) {
      return typeof a === 'string' ? a : JSON.stringify(a);
    }).join(' ');
    bro.__log(text);
  }
  if (typeof G.console === 'undefined') G.console = { log: log, info: log, warn: log, error: log };

  function headerValues(headers, name) {
    var target = String(name).toLowerCase();
    return Object.keys(headers || {}).filter(function (k) { return k.toLowerCase() === target; })
      .map(function (k) { return headers[k]; });
  }

  G.client = {
    global: {
      set: function (name, value) { bro.setVar(name, value); },
      get: function (name) { var v = bro.getVar(name); return v === undefined ? null : v; },
      isEmpty: function () { return Object.keys(bro.getAllVars()).length === 0; },
      clear: function (name) { bro.deleteVar(name); },
      clearAll: function () { Object.keys(bro.getAllVars()).forEach(function (k) { bro.deleteVar(k); }); }
    },
    test: function (name, fn) { test(name, fn); },
    assert: function (condition, message) {
      if (!condition) throw new Error(message || 'Assertion failed');
    },
    log: log,
    exit: function () { throw new Error('client.exit() is not supported'); }
  };

  var variables = {
    set: function (name, value) { bro.setVar(name, value); },
    get: function (name) {
      var v = bro.getVar(name);
      if (v === undefined) v = bro.getRequestVar(name);
      return v === undefined ? null : v;
    }
  };
  G.request = {
    method: req.method,
    variables: variables,
    environment: { get: function (name) { var v = bro.getEnvVar(name); return v === undefined ? null : v; } },
    url: {
      getRaw: function () { return req.url; },
      tryGetSubstituted: function () { return bro.interpolate(req.url); }
    },
    body: {
      getRaw: function () { return req.body; },
      tryGetSubstituted: function () { return bro.interpolate(req.body); }
    },
    headers: {
      all: function () {
        return Object.keys(req.headers).map(function (k) {
          return { name: k, getRawValue: function () { return req.headers[k]; },
                   tryGetSubstitutedValue: function () { return bro.interpolate(req.headers[k]); } };
        });
      },
      findByName: function (name) {
        var values = headerValues(req.headers, name);
        if (!values.length) return null;
        return { name: name, getRawValue: function () { return values[0]; },
                 tryGetSubstitutedValue: function () { return bro.interpolate(values[0]); } };
      }
    }
  };

  if (typeof res !== 'undefined') {
    var contentType = (res.getHeader('content-type') || '').split(';');
    var charset = contentType.slice(1).map(function (p) { return p.trim(); })
      .filter(function (p) { return p.toLowerCase().indexOf('charset=') === 0; })
      .map(function (p) { return p.slice(8); })[0] || '';
    var body = res.body;
    if (typeof body === 'string' && /[/+]json\b/i.test(contentType[0])) {
      try { body = JSON.parse(body); } catch (e) {}
    }
    G.response = {
      body: body,
      status: res.status,
      headers: {
        valueOf: function (name) { var v = headerValues(res.headers, name); return v.length ? v[0] : null; },
        valuesOf: function (name) { return headerValues(res.headers, name); }
      },
      contentType: { mimeType: contentType[0].trim(), charset: charset }
    };
  }

  function randomString(length, alphabet) {
    var out = '';
    for (var i = 0; i < length; i++) out += alphabet.charAt(Math.floor(Math.random() * alphabet.length));
    return out;
  }
  var letters = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
  G.$random = {
    integer: function (from, to) {
      from = from === undefined ? 0 : from; to = to === undefined ? 1000 : to;
      return Math.floor(from + Math.random() * (to - from));
    },
    float: function (from, to) {
      from = from === undefined ? 0 : from; to = to === undefined ? 1000 : to;
      return from + Math.random() * (to - from);
    },
    alphabetic: function (length) { return randomString(length, letters); },
    alphanumeric: function (length) { return randomString(length, letters + '0123456789'); },
    hexadecimal: function (length) { return randomString(length, '0123456789abcdef'); }
  };
  Object.defineProperty(G.$random, 'uuid', { get: function () { return crypto.randomUUID(); } });
  Object.defineProperty(G, '$uuid', { get: function () { return crypto.randomUUID(); } });
  Object.defineProperty(G, '$timestamp', { get: function () { return Math.floor(Date.now() / 1000); } });
  Object.defineProperty(G, '$isoTimestamp', { get: function () { return new Date().toISOString(); } });
  Object.defineProperty(G, '$randomInt', { get: function () { return Math.floor(Math.random() * 1001); } });
})();
"#;

/// Represents a syntax error found in a script
#[derive(Debug, Clone)]
pub struct ScriptDiagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub is_warning: bool,
}

/// Script execution service using rquickjs for JavaScript execution
#[derive(Clone)]
pub struct ScriptExecutionService {
    runtime: Runtime,
}

impl ScriptExecutionService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            runtime: Runtime::new()?,
        })
    }

    /// Execute a pre-request script
    pub fn execute_pre_request_script(
        &self,
        script: &str,
        request: &mut RequestData,
        variable_store: &VariableStore,
    ) -> Result<()> {
        if script.trim().is_empty() {
            return Ok(());
        }

        let ctx = Context::full(&self.runtime)?;
        ctx.with(|ctx| {
            // Setup global objects
            self.setup_request_object(ctx.clone(), request)?;
            self.setup_bro_object(ctx.clone(), variable_store)?;
            self.setup_nodejs_compatibility(ctx.clone())?;
            self.setup_bruno_compat(ctx.clone())?;
            self.setup_dialect(ctx.clone(), request)?;

            // Execute the script with better error handling
            if let Err(e) = ctx.eval::<(), _>(script) {
                // Try to catch the actual JavaScript error
                let js_error = ctx.catch();
                let detailed_error = format!("JavaScript Exception: {:?}", js_error);
                error!("Caught JavaScript error: {}", detailed_error);

                let error_msg = format!("Pre-request script failed: {} - {}", e, detailed_error);
                error!("Script execution error: {:?}", e); // Debug format for more details
                error!("Script content that failed: {}", script);
                return Err(anyhow::anyhow!(error_msg));
            }

            // Extract modifications from request object
            self.extract_request_modifications(ctx, request)?;

            Ok(())
        })
    }

    /// Execute a post-response script
    pub fn execute_post_response_script(
        &self,
        script: &str,
        request: &RequestData,
        response: &ResponseData,
        variable_store: &VariableStore,
    ) -> Result<()> {
        if script.trim().is_empty() {
            return Ok(());
        }

        let ctx = Context::full(&self.runtime)?;
        ctx.with(|ctx| {
            // Setup global objects
            self.setup_request_object(ctx.clone(), request)?;
            self.setup_response_object(ctx.clone(), response)?;
            self.setup_bro_object(ctx.clone(), variable_store)?;
            self.setup_nodejs_compatibility(ctx.clone())?;
            self.setup_bruno_compat(ctx.clone())?;
            self.setup_dialect(ctx.clone(), request)?;

            // Execute the script with better error handling
            if let Err(e) = ctx.eval::<(), _>(script) {
                // Try to catch the actual JavaScript error
                let js_error = ctx.catch();
                let detailed_error = format!("JavaScript Exception: {:?}", js_error);
                error!("Caught JavaScript error: {}", detailed_error);

                let error_msg = format!("Post-response script failed: {} - {}", e, detailed_error);
                error!("Script execution error: {:?}", e); // Debug format for more details
                error!("Script content that failed: {}", script);
                return Err(anyhow::anyhow!(error_msg));
            }

            Ok(())
        })
    }

    /// Setup the request object for JavaScript access
    fn setup_request_object(&self, ctx: Ctx, request: &RequestData) -> Result<()> {
        let req_obj = Object::new(ctx.clone())?;

        // Convert headers to JavaScript object
        let headers_obj = Object::new(ctx.clone())?;
        for header in &request.headers {
            if header.enabled {
                headers_obj.set(header.key.clone(), header.value.clone())?;
            }
        }
        req_obj.set("headers", headers_obj)?;

        // Set basic request properties
        req_obj.set("name", request.name.clone())?;
        req_obj.set("method", request.method.as_str())?;
        req_obj.set("url", request.url.clone())?;
        req_obj.set("body", request.body.clone())?;

        // Convert query parameters to JavaScript object
        let query_obj = Object::new(ctx.clone())?;
        for param in &request.query_params {
            if param.enabled {
                query_obj.set(param.key.clone(), param.value.clone())?;
            }
        }
        req_obj.set("query", query_obj)?;

        // Set the request object as global
        ctx.globals().set("req", req_obj)?;

        Ok(())
    }

    /// Setup the response object for JavaScript access
    fn setup_response_object(&self, ctx: Ctx, response: &ResponseData) -> Result<()> {
        let res_obj = Object::new(ctx.clone())?;

        // Set response properties
        if let Some(status_code) = response.status_code {
            res_obj.set("status", status_code)?;
        }

        res_obj.set("body", response.body.clone())?;

        // Convert headers to JavaScript object
        let headers_obj = Object::new(ctx.clone())?;
        for header in &response.headers {
            if header.enabled {
                headers_obj.set(header.key.clone(), header.value.clone())?;
            }
        }
        res_obj.set("headers", headers_obj)?;

        // Set additional response properties if available
        if let Some(latency) = response.latency {
            let ms = latency.as_millis() as i64;
            res_obj.set("latency", ms)?;
            res_obj.set("responseTime", ms)?;
        }
        if let Some(size) = response.size {
            res_obj.set("size", size as i64)?;
        }
        if let Some(status_text) = &response.status_text {
            res_obj.set("statusText", status_text.clone())?;
        }
        if let Some(url) = &response.url {
            res_obj.set("url", url.clone())?;
        }

        // Set the response object as global
        ctx.globals().set("res", res_obj)?;

        // Parse response body as JSON if content-type is application/json
        if let Some(_content_type_header) = response
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
            .filter(|h| h.value.to_lowercase().contains("application/json"))
        {
            ctx.eval::<(), _>("res.body = JSON.parse(res.body);")?;
        }

        Ok(())
    }

    /// Setup the `bro` object with variable management functions.
    ///
    /// This exposes the native primitives; the higher-level Bruno-compatible API
    /// (typed `getVar`/`setVar`, `req`/`res` methods, `test`/`expect`, and the
    /// `bru` alias) is layered on top in JS by [`Self::setup_bruno_compat`].
    ///
    /// The Rust <-> JS boundary is kept string-based (values crossing as JSON
    /// strings) to avoid engine-specific value conversions.
    fn setup_bro_object<'js>(&self, ctx: Ctx<'js>, variable_store: &VariableStore) -> Result<()> {
        let bro_obj = Object::new(ctx.clone())?;

        bro_obj.set(
            "__log",
            Function::new(ctx.clone(), |text: String| {
                tracing::info!(target: "script", "{}", text);
            })?,
        )?;

        // --- Environment variables ---
        let store = variable_store.clone();
        bro_obj.set(
            "setEnvVar",
            Function::new(ctx.clone(), move |name: String, value: String| {
                store.set_env_var_str(&name, &value);
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "getEnvVar",
            Function::new(ctx.clone(), move |name: String| -> Option<String> {
                store.get_env_var_str(&name)
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "hasEnvVar",
            Function::new(ctx.clone(), move |name: String| -> bool {
                store.has_env_var(&name)
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "deleteEnvVar",
            Function::new(ctx.clone(), move |name: String| {
                store.delete_env_var(&name);
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "__getAllEnvVars",
            Function::new(ctx.clone(), move || -> String {
                serde_json::to_string(&store.get_all_env_vars())
                    .unwrap_or_else(|_| "{}".to_string())
            })?,
        )?;

        // --- Runtime variables (JSON-encoded across the boundary) ---
        let store = variable_store.clone();
        bro_obj.set(
            "__getVar",
            Function::new(ctx.clone(), move |name: String| -> Option<String> {
                store.get_var(&name).map(|v| v.to_string())
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "__setVar",
            Function::new(ctx.clone(), move |name: String, json: String| {
                let value =
                    serde_json::from_str(&json).unwrap_or(serde_json::Value::String(json.clone()));
                store.set_var(&name, value);
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "hasVar",
            Function::new(ctx.clone(), move |name: String| -> bool {
                store.has_var(&name)
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "deleteVar",
            Function::new(ctx.clone(), move |name: String| {
                store.delete_var(&name);
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "__getAllVars",
            Function::new(ctx.clone(), move || -> String {
                serde_json::to_string(&store.get_all_vars()).unwrap_or_else(|_| "{}".to_string())
            })?,
        )?;

        // --- Collection variables (read-only; Bruno bru.getCollectionVar) ---
        let store = variable_store.clone();
        bro_obj.set(
            "__getCollectionVar",
            Function::new(ctx.clone(), move |name: String| -> Option<String> {
                store.get_collection_var(&name).map(|v| v.to_string())
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "__hasCollectionVar",
            Function::new(ctx.clone(), move |name: String| -> bool {
                store.has_collection_var(&name)
            })?,
        )?;

        // --- Request variables (read-only; Bruno bru.getRequestVar) ---
        let store = variable_store.clone();
        bro_obj.set(
            "__getRequestVar",
            Function::new(ctx.clone(), move |name: String| -> Option<String> {
                store.get_request_var(&name).map(|v| v.to_string())
            })?,
        )?;

        let store = variable_store.clone();
        bro_obj.set(
            "__hasRequestVar",
            Function::new(ctx.clone(), move |name: String| -> bool {
                store.has_request_var(&name)
            })?,
        )?;

        ctx.globals().set("bro", bro_obj)?;

        Ok(())
    }

    /// Layer Bruno's `bru`/`req`/`res` method API on top of the native objects,
    /// plus a minimal `test`/`expect`, entirely in JS. This lets OpenCollection
    /// scripts written against Bruno's `bru` API run unchanged — `bru` is aliased
    /// to broquest's `bro` object.
    fn setup_bruno_compat<'js>(&self, ctx: Ctx<'js>) -> Result<()> {
        ctx.eval::<(), _>(BRUNO_COMPAT_PRELUDE)?;
        Ok(())
    }

    /// Add the JetBrains HTTP Client API for requests whose scripts use it.
    fn setup_dialect<'js>(&self, ctx: Ctx<'js>, request: &RequestData) -> Result<()> {
        if request.script_dialect == ScriptDialect::JetBrains {
            ctx.eval::<(), _>(JETBRAINS_PRELUDE)?;
        }
        Ok(())
    }

    /// Setup Node.js compatibility functions
    fn setup_nodejs_compatibility<'js>(&self, ctx: Ctx<'js>) -> Result<()> {
        buffer::init(&ctx)?;
        crypto::init(&ctx)?;
        url::init(&ctx)?;

        Ok(())
    }

    /// Extract modifications from the JavaScript request object back to RequestData
    fn extract_request_modifications(&self, ctx: Ctx, request: &mut RequestData) -> Result<()> {
        // Get the request object from JavaScript
        let req_obj: Object = ctx.globals().get("req")?;

        // Extract modified headers
        if let Ok(headers_obj) = req_obj.get::<_, Object>("headers") {
            let mut new_headers = Vec::new();
            for key_result in headers_obj.keys::<String>() {
                if let Ok(key) = key_result
                    && let Ok(value) = headers_obj.get::<_, String>(&key)
                {
                    new_headers.push(KeyValuePair {
                        key,
                        value,
                        enabled: true,
                    });
                }
            }
            request.headers = new_headers;
        }

        // Extract modified URL if changed
        if let Ok(url) = req_obj.get::<_, String>("url") {
            request.url = url;
        }

        // Extract modified body if changed
        if let Ok(body) = req_obj.get::<_, String>("body") {
            request.body = body;
        }

        Ok(())
    }

    /// Check script syntax without executing it
    /// Returns Ok(()) if syntax is valid, Err(ScriptDiagnostic) if there's an error
    /// Syntax errors are reported as errors, ReferenceErrors as warnings
    pub fn check_syntax(script: &str, context: ScriptContext) -> Result<(), ScriptDiagnostic> {
        if script.trim().is_empty() {
            return Ok(());
        }

        let runtime = Runtime::new().map_err(|e| ScriptDiagnostic {
            line: 0,
            column: 0,
            message: format!("Failed to create runtime: {}", e),
            is_warning: false,
        })?;

        let ctx = Context::full(&runtime).map_err(|e| ScriptDiagnostic {
            line: 0,
            column: 0,
            message: format!("Failed to create context: {}", e),
            is_warning: false,
        })?;

        ctx.with(|ctx| {
            // Initialize LLRT modules for buffer, crypto, url support
            if let Err(e) = buffer::init(&ctx) {
                error!("Failed to initialize buffer module: {}", e);
            }
            if let Err(e) = crypto::init(&ctx) {
                error!("Failed to initialize crypto module: {}", e);
            }
            if let Err(e) = url::init(&ctx) {
                error!("Failed to initialize url module: {}", e);
            }

            // Set up stub global objects to avoid ReferenceError for req, bro
            // (res is only available in post-response scripts)
            let req_obj = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create req object: {}", e),
                is_warning: false,
            })?;
            let req_headers = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create req headers: {}", e),
                is_warning: false,
            })?;
            req_obj
                .set("headers", req_headers)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set req headers: {}", e),
                    is_warning: false,
                })?;
            req_obj.set("method", "GET").map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to set req method: {}", e),
                is_warning: false,
            })?;
            req_obj.set("url", "").map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to set req url: {}", e),
                is_warning: false,
            })?;
            req_obj.set("body", "").map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to set req body: {}", e),
                is_warning: false,
            })?;
            let req_query = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create req query: {}", e),
                is_warning: false,
            })?;
            req_obj
                .set("query", req_query)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set req query: {}", e),
                    is_warning: false,
                })?;
            ctx.globals()
                .set("req", req_obj)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set req global: {}", e),
                    is_warning: false,
                })?;

            // Set up stub res object only for post-response scripts
            if context == ScriptContext::PostResponse {
                let res_obj = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to create res object: {}", e),
                    is_warning: false,
                })?;
                res_obj.set("status", 200).map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set res status: {}", e),
                    is_warning: false,
                })?;
                res_obj.set("body", "").map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set res body: {}", e),
                    is_warning: false,
                })?;
                let res_headers = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to create res headers: {}", e),
                    is_warning: false,
                })?;
                res_obj
                    .set("headers", res_headers)
                    .map_err(|e| ScriptDiagnostic {
                        line: 0,
                        column: 0,
                        message: format!("Failed to set res headers: {}", e),
                        is_warning: false,
                    })?;
                ctx.globals()
                    .set("res", res_obj)
                    .map_err(|e| ScriptDiagnostic {
                        line: 0,
                        column: 0,
                        message: format!("Failed to set res global: {}", e),
                        is_warning: false,
                    })?;
            }

            // Set up stub bro object
            let bro_obj = Object::new(ctx.clone()).map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create bro object: {}", e),
                is_warning: false,
            })?;
            // Simple stub functions
            let stub_set_fn =
                Function::new(ctx.clone(), |_: String, _: String| {}).map_err(|e| {
                    ScriptDiagnostic {
                        line: 0,
                        column: 0,
                        message: format!("Failed to create stub setEnvVar: {}", e),
                        is_warning: false,
                    }
                })?;
            bro_obj
                .set("setEnvVar", stub_set_fn)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set setEnvVar: {}", e),
                    is_warning: false,
                })?;
            let stub_get_fn = Function::new(ctx.clone(), |_: String| -> rquickjs::Null {
                rquickjs::Null
            })
            .map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create stub getEnvVar: {}", e),
                is_warning: false,
            })?;
            bro_obj
                .set("getEnvVar", stub_get_fn)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set getEnvVar: {}", e),
                    is_warning: false,
                })?;
            // Stub the runtime/collection var methods the prelude exposes, so
            // scripts using bru.getVar/getCollectionVar/etc. don't raise a
            // (fatal) TypeError during the static syntax check.
            let stub_getter = Function::new(ctx.clone(), |_: String| -> rquickjs::Null {
                rquickjs::Null
            })
            .map_err(|e| ScriptDiagnostic {
                line: 0,
                column: 0,
                message: format!("Failed to create stub getter: {}", e),
                is_warning: false,
            })?;
            let stub_setter =
                Function::new(ctx.clone(), |_: String, _: String| {}).map_err(|e| {
                    ScriptDiagnostic {
                        line: 0,
                        column: 0,
                        message: format!("Failed to create stub setter: {}", e),
                        is_warning: false,
                    }
                })?;
            let stub_bool =
                Function::new(ctx.clone(), |_: String| -> bool { false }).map_err(|e| {
                    ScriptDiagnostic {
                        line: 0,
                        column: 0,
                        message: format!("Failed to create stub bool fn: {}", e),
                        is_warning: false,
                    }
                })?;
            for method in [
                "getVar",
                "setVar",
                "getAllVars",
                "getCollectionVar",
                "hasCollectionVar",
                "getRequestVar",
                "getFolderVar",
            ] {
                let f = if method == "setVar" {
                    stub_setter.clone()
                } else if method.starts_with("has") {
                    stub_bool.clone()
                } else {
                    stub_getter.clone()
                };
                bro_obj.set(method, f).map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set stub {}: {}", method, e),
                    is_warning: false,
                })?;
            }
            ctx.globals()
                .set("bro", bro_obj)
                .map_err(|e| ScriptDiagnostic {
                    line: 0,
                    column: 0,
                    message: format!("Failed to set bro global: {}", e),
                    is_warning: false,
                })?;

            // Try to evaluate the script
            if let Err(Error::Exception) = ctx.eval::<(), _>(script) {
                // Catch the exception to get details
                let exception = ctx.catch();

                // Log the full exception for debugging
                debug!("Script evaluation error: {:?}", exception);

                // Extract error message and position
                let (message, line, column, is_warning) =
                    if let Ok(exc_obj) = rquickjs::Exception::from_value(exception.clone()) {
                        let msg = exc_obj
                            .message()
                            .unwrap_or_else(|| "Unknown error".to_string());

                        // Get error name to distinguish SyntaxError from ReferenceError
                        let error_name = exc_obj
                            .get::<_, String>("name")
                            .ok()
                            .unwrap_or_else(|| "Unknown".to_string());

                        debug!(
                            "Error name: {}, message: {}, {:?}",
                            error_name, msg, exc_obj
                        );

                        // ReferenceError is a warning (variable might exist at runtime)
                        // SyntaxError and other errors are actual errors
                        let is_warning = error_name == "ReferenceError";

                        let mut line: u32 = 0;
                        let mut col: u32 = 0;

                        if let Some(stack) = exc_obj.stack() {
                            // Parse stack trace format: "at <eval> (eval_script:1:4)"
                            // or "at <filename>:<line>:<column>"
                            for line_part in stack.lines() {
                                let line_part = line_part.trim();
                                if line_part.contains("at ") && line_part.contains(":") {
                                    // Split by ":" and try to extract numbers
                                    let parts: Vec<&str> = line_part.split(':').collect();
                                    if parts.len() >= 2 {
                                        let mut found = false;
                                        // Try to parse the second-to-last part as line number
                                        if let Some(line_str) = parts.get(parts.len() - 2) {
                                            let line_str = line_str.trim();
                                            if let Ok(parsed_line) = line_str.parse::<u32>()
                                                && parsed_line > 0
                                            {
                                                line = parsed_line.saturating_sub(1);
                                                found = true;
                                            }
                                        }
                                        // Try to parse the last part as column number
                                        // Strip non-digit characters (like trailing ")" or "\n")
                                        if let Some(col_str) = parts.last() {
                                            let col_str = col_str
                                                .trim()
                                                .trim_end_matches(|c: char| !c.is_ascii_digit());
                                            if let Ok(parsed_col) = col_str.parse::<u32>()
                                                && parsed_col > 0
                                            {
                                                col = parsed_col.saturating_sub(1);
                                            }
                                        }
                                        if found {
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        (msg, line, col, is_warning)
                    } else {
                        (format!("{:?}", exception), 0, 0, false)
                    };

                return Err(ScriptDiagnostic {
                    line,
                    column,
                    message,
                    is_warning,
                });
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::collections::HashMap;

    fn service() -> ScriptExecutionService {
        ScriptExecutionService::new().expect("create script service")
    }

    fn vars(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn empty_script_is_a_noop() {
        let svc = service();
        let mut request = RequestData::default();
        let store = VariableStore::new();
        svc.execute_pre_request_script("   ", &mut request, &store)
            .expect("empty pre-request script should succeed");
        svc.execute_post_response_script("", &request, &ResponseData::default(), &store)
            .expect("empty post-response script should succeed");
    }

    #[test]
    fn pre_request_extracts_url_body_and_headers() {
        let svc = service();
        let mut request = RequestData {
            url: "http://old.example".to_string(),
            body: "old".to_string(),
            ..Default::default()
        };
        let store = VariableStore::new();

        svc.execute_pre_request_script(
            r#"
            req.setUrl('http://new.example/path');
            req.setBody('hello world');
            req.setHeader('X-Test', 'abc');
            "#,
            &mut request,
            &store,
        )
        .expect("script should run");

        assert_eq!(request.url, "http://new.example/path");
        assert_eq!(request.body, "hello world");
        assert!(
            request
                .headers
                .iter()
                .any(|h| h.key == "X-Test" && h.value == "abc"),
            "the header set by the script should be extracted back into RequestData"
        );
    }

    #[test]
    fn bro_and_bru_var_apis_write_runtime_and_env_vars() {
        let svc = service();
        let mut request = RequestData::default();
        let store = VariableStore::new();

        svc.execute_pre_request_script(
            r#"
            bro.setVar('num', 42);
            bru.setVar('str', 'two');
            bro.setEnvVar('E', 'ev');
            "#,
            &mut request,
            &store,
        )
        .expect("script should run");

        // `bru` is aliased to `bro`, so both write into the same runtime store.
        assert_eq!(store.get_var("num"), Some(json!(42)));
        assert_eq!(store.get_var("str"), Some(json!("two")));
        assert_eq!(store.get_env_var_str("E"), Some("ev".to_string()));
    }

    #[test]
    fn read_only_collection_and_request_var_scopes() {
        let svc = service();
        let mut request = RequestData::default();
        let store = VariableStore::new();
        store.set_collection_vars(vars(&[("cv", json!("cval"))]));
        store.set_request_vars(vars(&[("rv", json!("rval"))]));

        svc.execute_pre_request_script(
            r#"
            bro.setVar('gotC', bro.getCollectionVar('cv'));
            bro.setVar('gotR', bru.getRequestVar('rv'));
            bro.setVar('hasC', bro.hasCollectionVar('cv'));
            bro.setVar('missing', bro.getRequestVar('nope') === undefined);
            "#,
            &mut request,
            &store,
        )
        .expect("script should run");

        assert_eq!(store.get_var("gotC"), Some(json!("cval")));
        assert_eq!(store.get_var("gotR"), Some(json!("rval")));
        assert_eq!(store.get_var("hasC"), Some(json!(true)));
        assert_eq!(store.get_var("missing"), Some(json!(true)));
    }

    #[test]
    fn post_response_exposes_res_and_collects_passing_tests() {
        let svc = service();
        let request = RequestData::default();
        let store = VariableStore::new();
        let response = ResponseData {
            status_code: Some(200),
            body: r#"{"ok":true}"#.to_string(),
            headers: vec![KeyValuePair {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                enabled: true,
            }],
            ..Default::default()
        };

        svc.execute_post_response_script(
            r#"
            test('status is 200', function () { expect(res.getStatus()).to.equal(200); });
            test('body parsed as json', function () { expect(res.body.ok).to.equal(true); });
            bro.setVar('passed', __testResults.filter(function (t) { return t.status === 'pass'; }).length);
            bro.setVar('failed', __testResults.filter(function (t) { return t.status === 'fail'; }).length);
            "#,
            &request,
            &response,
            &store,
        )
        .expect("script should run");

        assert_eq!(store.get_var("passed"), Some(json!(2)));
        assert_eq!(store.get_var("failed"), Some(json!(0)));
    }

    fn jetbrains_request() -> RequestData {
        RequestData {
            url: "https://example.com/{{id}}".to_string(),
            body: r#"{"id": "{{id}}"}"#.to_string(),
            headers: vec![KeyValuePair {
                key: "X-Client".to_string(),
                value: "{{client}}".to_string(),
                enabled: true,
            }],
            script_dialect: crate::domain::ScriptDialect::JetBrains,
            ..Default::default()
        }
    }

    #[test]
    fn jetbrains_response_handler() {
        let svc = service();
        let store = VariableStore::new();
        let response = ResponseData {
            status_code: Some(201),
            body: r#"{"token":"abc","user":{"id":7}}"#.to_string(),
            headers: vec![
                KeyValuePair {
                    key: "Content-Type".to_string(),
                    value: "application/problem+json; charset=utf-8".to_string(),
                    enabled: true,
                },
                KeyValuePair {
                    key: "Location".to_string(),
                    value: "/users/7".to_string(),
                    enabled: true,
                },
            ],
            ..Default::default()
        };

        svc.execute_post_response_script(
            r#"
            client.global.set("token", response.body.token);
            client.global.set("userId", response.body.user.id);
            client.global.set("location", response.headers.valueOf("location"));
            client.global.set("missing", response.headers.valueOf("x-nope"));
            client.global.set("mime", response.contentType.mimeType);
            client.global.set("charset", response.contentType.charset);
            client.test("created", function () {
                client.assert(response.status === 201, "status should be 201");
            });
            client.test("fails", function () { client.assert(false, "nope"); });
            client.global.set("results", __testResults.map(function (t) { return t.status; }));
            client.global.set("uuid", $random.uuid.length);
            client.global.set("number", typeof $random.integer(1, 3));
            client.log("handled", { status: response.status });
            "#,
            &jetbrains_request(),
            &response,
            &store,
        )
        .expect("script should run");

        assert_eq!(store.get_var("token"), Some(json!("abc")));
        assert_eq!(store.get_var("userId"), Some(json!(7)));
        assert_eq!(store.get_var("location"), Some(json!("/users/7")));
        assert_eq!(store.get_var("missing"), Some(json!(null)));
        assert_eq!(
            store.get_var("mime"),
            Some(json!("application/problem+json"))
        );
        assert_eq!(store.get_var("charset"), Some(json!("utf-8")));
        assert_eq!(store.get_var("results"), Some(json!(["pass", "fail"])));
        assert_eq!(store.get_var("uuid"), Some(json!(36)));
        assert_eq!(store.get_var("number"), Some(json!("number")));
    }

    #[test]
    fn jetbrains_pre_request_script() {
        let svc = service();
        let store = VariableStore::new();
        store.initialize_with_env(
            &HashMap::from([("client".to_string(), "broquest".to_string())]),
            &HashMap::new(),
        );
        let mut request = jetbrains_request();

        svc.execute_pre_request_script(
            r#"
            request.variables.set("id", "42");
            client.global.set("raw", request.url.getRaw());
            client.global.set("header", request.headers.findByName("x-client").tryGetSubstitutedValue());
            client.global.set("env", request.environment.get("client"));
            client.global.set("method", request.method);
            "#,
            &mut request,
            &store,
        )
        .expect("script should run");

        assert_eq!(store.get_var("id"), Some(json!("42")));
        assert_eq!(
            store.get_var("raw"),
            Some(json!("https://example.com/{{id}}"))
        );
        assert_eq!(store.get_var("header"), Some(json!("broquest")));
        assert_eq!(store.get_var("env"), Some(json!("broquest")));
        assert_eq!(store.get_var("method"), Some(json!("GET")));
    }

    #[test]
    fn jetbrains_api_is_only_there_for_its_dialect() {
        let svc = service();
        let store = VariableStore::new();
        let result = svc.execute_pre_request_script(
            "client.global.set('x', 1);",
            &mut RequestData::default(),
            &store,
        );
        assert!(result.is_err());
    }

    #[test]
    fn failing_test_is_caught_and_recorded() {
        let svc = service();
        let request = RequestData::default();
        let store = VariableStore::new();

        // A failing expect inside test() is caught; the script itself still
        // succeeds and the failure is recorded in __testResults.
        svc.execute_post_response_script(
            r#"
            test('this fails', function () { expect(1).to.equal(2); });
            bro.setVar('failed', __testResults.filter(function (t) { return t.status === 'fail'; }).length);
            "#,
            &request,
            &ResponseData::default(),
            &store,
        )
        .expect("script with a failing test should still succeed");

        assert_eq!(store.get_var("failed"), Some(json!(1)));
    }

    #[test]
    fn bare_failing_expect_propagates_as_error() {
        let svc = service();
        let mut request = RequestData::default();
        let store = VariableStore::new();

        // Outside test(), a failed expectation throws and surfaces as a script
        // execution error.
        let result = svc.execute_pre_request_script("expect(1).to.equal(2);", &mut request, &store);
        assert!(result.is_err(), "a bare failing expect should error");
    }

    #[test]
    fn syntax_error_surfaces_as_error() {
        let svc = service();
        let mut request = RequestData::default();
        let store = VariableStore::new();

        let result = svc.execute_pre_request_script("var x = ;", &mut request, &store);
        assert!(result.is_err(), "a syntax error should fail execution");
    }

    #[test]
    fn check_syntax_distinguishes_valid_and_invalid_scripts() {
        assert!(
            ScriptExecutionService::check_syntax("var x = 1 + 1;", ScriptContext::PreRequest)
                .is_ok()
        );
        assert!(
            ScriptExecutionService::check_syntax("var x = ;", ScriptContext::PreRequest).is_err()
        );
    }
}
