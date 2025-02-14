variable "aws_region" {
  description = "AWS region"
  default = "eu-north-1"
}
variable "ecr_repo_name" {
  description = "Name of the ECR repository"
}

variable "ecr_repo_url" {
  description = "URL of the ECR repository"
}

variable "lambda_function_name" {
  description = "Name of the Lambda function"
}

variable "image_tag" {
  description = "Tag of the Docker image"
}

variable "username" {
  description = "actic username"
}

variable "password" {
  description = "actic password"
}

variable "rule_name" {
  description = "Name of the EventBridge rule"
}