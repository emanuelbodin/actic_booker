variable "aws_region" {
  description = "AWS region"
  default = "eu-north-1"
}

variable "lambda_arn" {
  description = "ARN of the Lambda function"
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