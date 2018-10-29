module "docker_worker_packet" {
  source = "github.com/servo/taskcluster-infrastructure//modules/docker-worker?ref=424ea4ff13de34df70e5242706fe1e26864cc383"

  packet_project_id = "e3d0d8be-9e4c-4d39-90af-38660eb70544"
  packet_instance_type = "t1.small.x86"
  number_of_machines = "1"
  concurrency = "1"

  provisioner_id = "proj-servo"
  worker_type = "docker-worker-kvm"
  worker_group_prefix = "servo-packet"

  taskcluster_client_id    = "${var.taskcluster_client_id}"
  taskcluster_access_token = "${var.taskcluster_access_token}"
  ssl_certificate          = "${var.ssl_certificate}"
  cert_key                 = "${var.cert_key}"
  ssh_pub_key              = "${var.ssh_pub_key}"
  ssh_priv_key             = "${var.ssh_priv_key}"
  private_key              = " "
  relengapi_token          = " "
  stateless_hostname       = " "
}

variable "taskcluster_client_id" {}
variable "taskcluster_access_token" {}
variable "ssl_certificate" {}
variable "cert_key" {}
variable "ssh_pub_key" {}
variable "ssh_priv_key" {}