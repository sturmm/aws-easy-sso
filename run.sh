#!/bin/false

AWS_EASY_SSO_SOURCING_MODE=true aws-easy-sso-cli $@
test $? -eq 0 && . ~/.awseasysso/export_profile || exit 1;
