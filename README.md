# aws-easy-sso - AWS SSO on CLI made easy

<img src="doc/demo.gif" width="500">

`aws-easy-sso` is a simple tool to login to your AWS accounts using SSO. So of course it just works if you have [configured SSO in you organization](https://aws.amazon.com/de/iam/identity-center/). 

## Features

The tool uses the SSO and OIDC SDK to fetch available accounts and roles for your login and appends the [session and profile configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html) to your `~/.aws/config`. It also places the authentication information it used itself in the `~/.aws/sso/cache` directory. These cached sessions are used by the newer credential provider implementations of AWS SDKs. The tool also supports usage of multiple sso sessions (combinations of `sso_start_url` and `sso_region`). For personal learning, the tool is written in Rust.

## Installation

### Cargo: 
``` bash
$ cargo install aws-easy-sso
```

### Setup alias:
#### Zsh:
``` bash
# To allow the program to export variables to the 'outer' env, 
# the actual scripts need to be sourced
$ echo "alias aws-easy-sso="source _aws-easy-sso"" >> ~/.zshrc
```

#### bash:
``` bash
# To allow the program to export variables to the 'outer' env, 
# the actual scripts need to be sourced
$ echo "alias aws-easy-sso="source _aws-easy-sso"" >> ~/.bashrc
```

### Limitations
Windows is no supported currently.

## See also
There are plenty of alternatives that worked for me in the past:

* https://github.com/johnnyopao/awsp
* https://github.com/ohmyzsh/ohmyzsh/blob/master/plugins/aws/aws.plugin.zsh
* https://github.com/benkehoe/aws-sso-util
