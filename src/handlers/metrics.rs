use crate::observability::metrics::MetricsExporter;
use actix_web::{HttpResponse, get, web};

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Observability",
    responses(
        (status = 200, description = "Prometheus metrics in text format", content_type = "text/plain")
    )
)]
#[get("/metrics")]
pub async fn metrics(exporter: web::Data<MetricsExporter>) -> HttpResponse {
    tracing::debug!("Collecting Prometheus metrics");
    let metrics = exporter.collect_metrics();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let exporter = MetricsExporter::default();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(exporter))
                .service(metrics),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "text/plain; version=0.0.4"
        );
    }
}
