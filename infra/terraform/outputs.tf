output "cluster_id" {
  description = "ID du cluster Kubernetes"
  value       = digitalocean_kubernetes_cluster.valoria.id
}

output "cluster_endpoint" {
  description = "Endpoint du cluster"
  value       = digitalocean_kubernetes_cluster.valoria.endpoint
  sensitive   = true
}

output "cluster_status" {
  description = "Status du cluster"
  value       = digitalocean_kubernetes_cluster.valoria.status
}