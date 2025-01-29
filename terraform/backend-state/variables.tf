variable "bucket_name" {
  description = "The name of the S3 bucket to create to store the Terraform state file"
  type        = string
}

variable "table_name" {
  description = "The name of the DynamoDB table to create to store the Terraform state lock"
  type        = string
}

variable "aws_region" {
  description = "AWS region"
  default     = "eu-north-1"
}