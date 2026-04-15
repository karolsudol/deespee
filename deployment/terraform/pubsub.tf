# Pub/Sub Schema for Deespee Messages
resource "google_pubsub_schema" "deespee_schema" {
  for_each = local.deploy_project_ids

  name    = "deespee-schema"
  type    = "PROTOCOL_BUFFER"
  project = each.value
  definition = file("${path.module}/../../shared/schemas/messages.proto")
}

# Agent Requests Topic
resource "google_pubsub_topic" "agent_requests" {
  for_each = local.deploy_project_ids

  name    = "agent-requests"
  project = each.value

  schema_settings {
    schema   = google_pubsub_schema.deespee_schema[each.key].id
    encoding = "BINARY"
  }
}

# DSP Actions Topic
resource "google_pubsub_topic" "dsp_actions" {
  for_each = local.deploy_project_ids

  name    = "dsp-actions"
  project = each.value

  schema_settings {
    schema   = google_pubsub_schema.deespee_schema[each.key].id
    encoding = "BINARY"
  }
}

# DSP Push Subscription (Triggers DSP Service)
resource "google_pubsub_subscription" "dsp_push" {
  for_each = local.deploy_project_ids

  name    = "dsp-push-subscription"
  topic   = google_pubsub_topic.agent_requests[each.key].name
  project = each.value

  push_config {
    # This URL will be dynamic based on the Cloud Run service URL
    # For now using a placeholder that matches the Cloud Run naming convention
    push_endpoint = "https://dsp-${data.google_project.project[each.key].number}.${var.region}.run.app/pubsub/push"
    
    oidc_token {
      service_account_email = google_service_account.app_sa[each.key].email
    }
  }

  retry_policy {
    minimum_backoff = "10s"
    maximum_backoff = "600s"
  }
}
