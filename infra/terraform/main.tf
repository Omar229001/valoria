terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "digitalocean" {
  token = var.do_token
}

resource "digitalocean_kubernetes_cluster" "valoria" {
  name    = var.cluster_name
  region  = var.region
  version = "1.35.1-do.2"

  node_pool {
    name       = "valoria-pool"
    size       = var.node_size
    node_count = var.node_count
  }
}