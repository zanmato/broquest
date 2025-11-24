use crate::request_editor::{KeyValuePair, RequestData, ResponseData};
use crate::variable_store::VariableStore;
use anyhow::Result;
use rquickjs::{Context, Ctx, Function, Object, Runtime};
use tracing::{error, info};

// Base64 engine for compatibility
use base64::{Engine as _, engine::general_purpose};

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
        // btoa function (base64 encoding) - simplified for now
        let btoa_fn = Function::new(
            ctx.clone(),
            |data: String| -> Result<String, rquickjs::Error> {
                Ok(general_purpose::STANDARD.encode(data.as_bytes()))
            },
        )?;
        ctx.globals().set("btoa", btoa_fn)?;

        // atob function (base64 decoding) - simplified for now
        let atob_fn = Function::new(
            ctx.clone(),
            |data: String| -> Result<String, rquickjs::Error> {
                match general_purpose::STANDARD.decode(data.as_bytes()) {
                    Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).to_string()),
                    Err(_) => Err(rquickjs::Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Base64 decode error"
                    ))),
                }
            },
        )?;
        ctx.globals().set("atob", atob_fn)?;

        // Buffer.from implementation (simplified version for common use cases)
        let buffer_from_fn = Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>, input: rquickjs::Value<'js>, _encoding: Option<String>| -> Result<rquickjs::Value<'js>, rquickjs::Error> {
                match input {
                    value if value.is_string() => {
                        let s = rquickjs::String::from_value(value)?;
                        let string = s.to_string()?;
                        let bytes = string.as_bytes().to_vec();

                        // Create a Uint8Array to represent the Buffer
                        let array = rquickjs::Array::new(ctx)?;
                        for (i, byte) in bytes.iter().enumerate() {
                            array.set(i, *byte as i32)?;
                        }
                        Ok(array.into_value())
                    },
                    value if value.is_array() => {
                        // If input is an array, treat it as byte values
                        let arr = rquickjs::Array::from_value(value)?;
                        let len = arr.len();
                        let mut bytes = Vec::with_capacity(len);
                        for i in 0..len {
                            let value: i32 = arr.get(i)?;
                            bytes.push(value as u8);
                        }

                        // Create a Uint8Array
                        let array = rquickjs::Array::new(ctx)?;
                        for (i, byte) in bytes.iter().enumerate() {
                            array.set(i, *byte as i32)?;
                        }
                        Ok(array.into_value())
                    },
                    _ => Err(rquickjs::Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unsupported input type for Buffer.from"
                    ))),
                }
            },
        )?;

        // Create Buffer object
        let buffer_obj = Object::new(ctx.clone())?;
        buffer_obj.set("from", buffer_from_fn)?;
        ctx.globals().set("Buffer", buffer_obj)?;

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
                    && let Ok(value) = headers_obj.get::<_, String>(&key) {
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
}
