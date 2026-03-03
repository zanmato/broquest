use super::completion::ScriptContext;
use super::variable_store::VariableStore;
use crate::domain::{KeyValuePair, RequestData, ResponseData};
use anyhow::Result;
use rquickjs::{Context, Ctx, Error, Function, Object, Runtime};
use tracing::{debug, error, info};

// LLRT modules for buffer, crypto, and URL support
use llrt_modules::{buffer, crypto, url};

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
            res_obj.set("latency", latency.as_millis() as i64)?;
        }
        if let Some(size) = response.size {
            res_obj.set("size", size as i64)?;
        }
        if let Some(status_text) = &response.status_text {
            res_obj.set("statusText", status_text.clone())?;
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

    /// Setup the bro object with variable management functions
    fn setup_bro_object<'js>(&self, ctx: Ctx<'js>, variable_store: &VariableStore) -> Result<()> {
        let bro_obj = Object::new(ctx.clone())?;

        // Create closures that capture the variable store
        let variable_store_clone1 = variable_store.clone();
        let variable_store_clone2 = variable_store.clone();

        // bro.setEnvVar function
        let set_env_var_fn = Function::new(
            ctx.clone(),
            move |name: String, value: String| -> Result<(), rquickjs::Error> {
                info!("Setting environment variable: {} = {}", name, value);
                variable_store_clone1.set_env_var_str(&name, &value);
                Ok(())
            },
        )?;
        bro_obj.set("setEnvVar", set_env_var_fn)?;

        // bro.getEnvVar function
        let get_env_var_fn = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>, name: String| -> Result<rquickjs::Value<'js>, rquickjs::Error> {
                if let Some(value) = variable_store_clone2.get_env_var_str(&name) {
                    info!("Getting environment variable: {} = {}", name, value);
                    Ok(rquickjs::String::from_str(ctx, &value)?.into_value())
                } else {
                    info!("Environment variable not found: {}", name);
                    Ok(rquickjs::Value::new_undefined(ctx))
                }
            },
        )?;
        bro_obj.set("getEnvVar", get_env_var_fn)?;

        // Set the bro object as global
        ctx.globals().set("bro", bro_obj)?;

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
            let _ = buffer::init(&ctx);
            let _ = crypto::init(&ctx);
            let _ = url::init(&ctx);

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
                                            if let Ok(parsed_line) = line_str.parse::<u32>() {
                                                if parsed_line > 0 {
                                                    line = parsed_line.saturating_sub(1);
                                                    found = true;
                                                }
                                            }
                                        }
                                        // Try to parse the last part as column number
                                        // Strip non-digit characters (like trailing ")" or "\n")
                                        if let Some(col_str) = parts.last() {
                                            let col_str = col_str
                                                .trim()
                                                .trim_end_matches(|c: char| !c.is_ascii_digit());
                                            if let Ok(parsed_col) = col_str.parse::<u32>() {
                                                if parsed_col > 0 {
                                                    col = parsed_col.saturating_sub(1);
                                                }
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
