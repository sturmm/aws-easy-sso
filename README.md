# aws-easy-sso - AWS SSO on CLI made easy

<img src="doc/demo.gif" width="500">

`aws-easy-sso` is a simple tool to login to your AWS accounts using SSO. So of course it just works if you have [configured SSO in you organization](https://aws.amazon.com/de/iam/identity-center/). For personal learning, the tool is written in Rust.

## Installation

### Build: 
```
$ git clone https://github.com/sturmm/aws-easy-sso.git
$ cd aws-easy-sso
$ cargo build --release
```

### Export to PATH:
#### Zsh:
```
$ echo "path+=('$(pwd)/target/release')" >> ~/.zshrc
```

#### bash:
```
$ echo "PATH=$PATH:$(pwd)/target/release" >> ~/.bash_profile
```

## See also
There are plenty of alternatives that worked for me in the past:

* https://github.com/johnnyopao/awsp
* https://github.com/ohmyzsh/ohmyzsh/blob/master/plugins/aws/aws.plugin.zsh
* https://github.com/benkehoe/aws-sso-util
