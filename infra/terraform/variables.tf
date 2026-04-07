variable "do_token" {
  description = "DigitalOcean API Token"
  type        = string
  sensitive   = true
}

variable "region" {
  description = "DigitalOcean region"
  type        = string
  default     = "fra1"
}

variable "cluster_name" {
  description = "Kubernetes cluster name"
  type        = string
  default     = "valoria-prod"
}

variable "node_size" {
  description = "Node droplet size"
  type        = string
  default     = "s-2vcpu-4gb"
}

variable "node_count" {
  description = "Number of nodes in the pool"
  type        = number
  default     = 2
}