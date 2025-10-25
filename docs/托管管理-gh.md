
## gh - prepare vars | 版本管理 | 准备变量

```powershell
$repo="ymc-github/pscan";$repo_desc="A cross-platform process filtering and analysis tool with window detection support";

$repo_uname=$repo -replace "-","_" -replace "/","_";
$repo_name=$repo  -replace ".*/","";
$repo_user=$repo  -replace "/.*","";

$email=git config user.email;

$repo_user;
$repo_name;
```

## gh - login with token from file

```powershell
# gh auth login --with-token < mytoken.txt
$token=get-content d:/book/secret/github.token.md;
# $token

# gh auth login --with-token $token # fail in powershell

# gh login with token from file in powershell
get-content d:/book/secret/github.token.md | gh auth login --with-token

# sh -c 'gh auth login --with-token < secret/github.token.md'

# gh issue list --label "bug" --label "help wanted"
```

## gh - add github repo - public

## gh - add github repo - private

```powershell
# todo:
# yours gh/repo --name xx --description xx --method create

# create github repo
gh repo create $repo_name --public --description "$repo_desc"

gh repo create $repo_name --private --description "$repo_desc"
```

## gh - create deploy token | 创建部署用密钥

```powershell
# $repo="ymc-github/yours"

# list repo for some repo
gh repo deploy-key list --repo $repo

# std repo name and get email from git
# $repo_uname=$repo -replace "-","_" -replace "/","_";$email=git config user.email;

# mkdir -p ~/.ssh/;

# ssh-keygen -C "$email" -f ~/.ssh/gh_$repo_uname -t ed25519 -N "123" #done

ssh-keygen -C "$email" -f $HOME/.ssh/gh_$repo_uname -t ed25519 -N '""' #done

```

[ssh-keygen-in-windows-powershell-create-a-key-pair-and-avoid-pr](https://superuser.com/questions/1634427/non-interactive-ssh-keygen-in-windows-powershell-create-a-key-pair-and-avoid-pr)

## gh - upload github deploy | 上传部署用密钥

```powershell
# gh repo deploy-key list --repo $repo;
gh repo deploy-key add $HOME/.ssh/gh_${repo_uname}.pub --repo $repo -w --title deploy;
```

## gh - set ssh key to ssh client | 使用部署密钥

```powershell
$txt=@"
Host github.com
    User git
    HostName github.com
    PreferredAuthentications publickey
    IdentityFile ~/.ssh/gh_${repo_uname}
"@

set-content -path $HOME/.ssh/config -value $txt
ssh -T git@github.com
# ssh -Tv git@github.com
```

## gh - set secret - gh.token.auto

```powershell

# gh secret set --repo $repo  -f D:\book\secret\npm.token.auto.md
gh secret set --repo $repo  -f D:\book\secret\gh.token.auto.md
```

## gh - push local repo to github repo - the first time

```powershell
# git remote -v
# git remote remove ghg
git remote add ghg git@github.com:$repo.git
git push -u ghg main
```

## gh - push local repo to github repo - not the first time

```powershell
# git remote -v
# git remote remove ghg
git push ghg main
```

## gh - edit repo info

```powershell
# rename your github repo ? do
# gh repo rename issues --repo ymc-github/some-issues
# gh repo rename utxt --repo ymc-github/nano-utxt

# enable issues and wiki ? do
gh repo edit --enable-issues --enable-wiki --repo $repo

# enable discussions in the repository
gh repo edit --enable-discussions --repo $repo

# put  your github repo to private ? do
gh repo edit $repo --visibility "private"

# put  your github repo to public ? do
gh repo edit $repo --visibility "public"
```