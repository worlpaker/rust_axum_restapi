use std::env::var;
/// Configures tracing for the application.
///
/// This function sets up the tracing subsystem for the application, including
/// the filtering, formatting, and exporting of trace events. It initializes
/// an OpenTelemetry exporter to send trace data to a specified endpoint.
pub fn tracing() {
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::prelude::*;

    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer =
        tracing_subscriber::fmt::layer().event_format(tracing_subscriber::fmt::format().pretty());

    let otel_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&var("JAEGER_URL").expect("JAEGER_URL must be in environment"));

    let otel_resource = opentelemetry::sdk::Resource::new([opentelemetry::KeyValue::new(
        "service.name",
        "backend",
    )]);

    let otel_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otel_exporter)
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(otel_resource)
                .with_sampler(opentelemetry::sdk::trace::Sampler::AlwaysOn),
        )
        .install_simple()
        .unwrap();

    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    tracing_subscriber::Registry::default()
        .with(filter_layer)
        .with(fmt_layer)
        .with(otel_trace_layer)
        .init();
}
