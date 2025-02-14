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

resource "aws_ecr_repository" "app" {
  name                 = var.ecr_repo_name
  image_scanning_configuration {
    scan_on_push = true
  }
}

// Lambda
data "aws_iam_policy_document" "assume_role" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "iam_for_lambda" {
  name               = "iam_for_lambda"
  assume_role_policy = data.aws_iam_policy_document.assume_role.json
}


resource "aws_lambda_function" "app" {
  function_name = var.lambda_function_name
  role          = aws_iam_role.iam_for_lambda.arn
  image_uri     = "${aws_ecr_repository.app.repository_url}:${var.image_tag}"
  package_type  = "Image"
  architectures = ["arm64"]
  timeout       = 10
  environment {
    variables = {
      USERNAME = var.username
      PASSWORD = var.password
    }
  }
}

// Eventbridge 

module "eventbridge" {
  source = "terraform-aws-modules/eventbridge/aws"

  create_bus = false
  rules = {
    "${var.rule_name}" = {
      description         = "Trigger for a Lambda"
      schedule_expression = "cron(45 18 ? * Fri *)"
    }
  }

  targets = {
    "${var.rule_name}" = [
      {
        name  = "spinning monday 18:45"
        arn   = aws_lambda_function.app.arn
        input = jsonencode({"center_id": 110, "name": "Spinning", "day": "Mon", "start_time": "18:45", "latest": "true"})
      }
    ]
  }
}