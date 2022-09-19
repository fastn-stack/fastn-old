#[derive(Default)]
pub struct HostExports {}

impl fpm_utils::host::host::Host for HostExports {
    fn http(
        &mut self,
        request: fpm_utils::host::host::Httprequest<'_>,
    ) -> fpm_utils::host::host::Httpresponse {
        return fpm_utils::host::host::Httpresponse {
            data: String::from("Hello WASM"),
        };
    }
}

pub struct Context<I, E> {
    // wasi: wasmtime_wasi::WasiCtx,
    pub imports: I,
    pub exports: E,
}
