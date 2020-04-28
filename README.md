# Hooked-rs

A utility for accepting GitHub webhooks and running a set of commands, configured via a yaml.

## YAML Format
See example.yml

## HTTP status codes
 - 200: Status hook received and added to queue for a known target and branch
 - 202: Status hook received for a known target, but the state of the hook was not "success", so no action was taken
 - 204: Status hook received for a known target, but a branch did not match, so no action was taken
 - 404: Status hook received but it did not match any target 
 - 422: Unhandled webhook. All webhook types except "status" will return this. To prevent this, configure your webhook to only send status hooks.

As target jobs are run asyncronously, the HTTP status code returned to GH will not reflect the success of the underlying job.

## GH Webhook events supported
 - `status`

 ## GH Webhook references
  - https://developer.github.com/webhooks/#events
  - https://developer.github.com/webhooks/testing/
  - https://developer.github.com/webhooks/securing/
