terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {}
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      project = "actic-booker"
    }
  }
}

module "eventbridge" {
  source = "terraform-aws-modules/eventbridge/aws"

  create_bus = false

  rules = {
    crons = {
      description         = "Trigger for a Lambda"
      schedule_expression = "cron(45 18 * * ? *)"
    }
  }

  targets = {
    crons = [
      {
        name  = "lambda-loves-cron"
        arn   = var.lambda_arn
        input = jsonencode({"name": "cron-by-rate"})
      }
    ]
  }
}