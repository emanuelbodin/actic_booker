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
  name               = "${var.lambda_function_name}_role"
  assume_role_policy = data.aws_iam_policy_document.assume_role.json
}


resource "aws_iam_policy" "cloudwatch_logs" {
  name        = "${var.lambda_function_name}_policy"
  description = "Allows Lambda to write logs to CloudWatch"
  
  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect   = "Allow"
        Action   = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_logs_attach" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.cloudwatch_logs.arn
}

resource "aws_cloudwatch_log_group" "lambda" {
  name              = "/aws/lambda/${var.lambda_function_name}"
  retention_in_days = 14
  lifecycle {
    prevent_destroy = false
  }
}

resource "aws_lambda_function" "app" {
  function_name = var.lambda_function_name
  role          = aws_iam_role.iam_for_lambda.arn
  image_uri     = "${aws_ecr_repository.app.repository_url}:${var.image_tag}"
  package_type  = "Image"
  architectures = ["arm64"]
  timeout       = 10
  depends_on    = [aws_cloudwatch_log_group.lambda]
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
      schedule_expression = "cron(45 17 ? * Fri *)" // Time in UTC, runs every Friday at 18:45 Swedish time (17:45 UTC)
      
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

resource "aws_lambda_permission" "allow_eventbridge" {
  statement_id  = "${var.lambda_function_name}-allow-eventbridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.app.function_name
  principal     = "events.amazonaws.com"
  source_arn    = module.eventbridge.eventbridge_rule_arns["${var.rule_name}"]
}