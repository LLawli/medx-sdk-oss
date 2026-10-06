# Soltando uma versão

SDK, `medx-cli` e `medx-mcp` têm uma versão só, a do `[workspace.package]`
no `Cargo.toml`. Soltar uma versão é um comando:

```sh
bin/release 0.2.0     # ou: just release 0.2.0
```

Ele:

1. Recusa rodar com mudanças no commit do working copy, sem nada em
   `[Unreleased]` nos dois CHANGELOGs, ou se a tag já existe.
2. Põe a versão no `Cargo.toml` e no `Cargo.lock`.
3. Transforma `[Unreleased]` em `[0.2.0] - <hoje>` no
   [CHANGELOG.md](../CHANGELOG.md) e no
   [medx-mcp/CHANGELOG.md](../medx-mcp/CHANGELOG.md) e atualiza os links de
   comparação. A crate sem mudanças ganha a seção com "Sem mudanças nesta
   crate.".
4. Roda `just check`.
5. Commita `chore: release v0.2.0` com jj e aponta o `master` para ele.
6. Empurra o `master`. Os commits são assinados no push: um toque na
   YubiKey por commit que sai.
7. Cria a tag assinada `v0.2.0` no commit empurrado (a assinatura no push
   reescreveu o commit, então a tag vem depois) e empurra a tag.

`--no-push` para depois do commit local; `--no-check` pula os gates. Os dois
existem para experimentar o script.

## O que a tag dispara

[.github/workflows/release.yml](../.github/workflows/release.yml), em toda
tag `v*.*.*` (ou à mão, com uma tag que já existe):

1. Confere que a tag bate com a versão do `Cargo.toml` e que os dois
   CHANGELOGs têm a seção dela.
2. Roda os gates no Linux.
3. Compila uma vez por plataforma, nativamente: Linux x86_64 e aarch64
   (musl, estático), macOS aarch64 e x86_64, Windows x86_64. Cada arquivo
   (`medx-<target>.tar.gz`, `.zip` no Windows) leva o `medx-cli`, o
   `medx-mcp`, os READMEs, os CHANGELOGs e a licença, com um `.sha256` ao
   lado. Binário ausente ou com menos de 1 MB reprova o build.
4. Preenche a fórmula do Homebrew com os checksums
   ([packaging/homebrew/](../packaging/homebrew/)).
5. Cria o GitHub Release com as seções dos dois CHANGELOGs, instruções de
   instalação e os checksums.
6. Leva a fórmula ao tap `LLawli/homebrew-tap`, quando ligado.
7. Monta o pacote do Chocolatey com o checksum do `.zip` de Windows
   publicado, instala o pacote num runner Windows, testa o `medx-cli` e o
   `medx-mcp`, desinstala e, quando ligado, envia ao community.chocolatey.org.

Um PR que mexe no `release.yml`, em `packaging/` ou no `bin/release` roda os
passos 3 e 4 sem publicar nada, e o passo 7 contra o último release
publicado, também sem publicar, para o pipeline não estrear quebrado numa
tag.

## Ligar o Homebrew (uma vez)

O passo 6 fica desligado até o repositório ter:

- a variável `HOMEBREW_TAP=true`;
- o secret `HOMEBREW_TAP_DEPLOY_KEY`, a metade privada de uma chave SSH
  cadastrada como deploy key **de escrita** no `LLawli/homebrew-tap` (o
  mesmo esquema dos outros projetos do tap).

Só faz sentido com o repositório público: a fórmula baixa os arquivos do
release, e os de um repositório privado não abrem sem autenticação. Ao
ligar, acrescente o `medx` à tabela do README do tap.

## Ligar o Chocolatey (uma vez)

O envio do passo 7 fica desligado até o repositório ter:

- a variável `CHOCOLATEY=true`;
- o secret `CHOCOLATEY_API_KEY`, a chave de API da conta `luka_kuuhaku` no
  [community.chocolatey.org](https://community.chocolatey.org/account).

Toda versão enviada passa pela moderação do Chocolatey antes de aparecer
para `choco install`: a primeira, revisada por uma pessoa, costuma levar
dias; as seguintes, quando passam nas verificações automáticas, saem mais
rápido. O pacote (`packaging/chocolatey/`) baixa o `.zip` do GitHub
Release e confere o SHA256; não embute o binário.

Quando a moderação pede correção numa versão ("Waiting for Maintainer"),
corrija `packaging/chocolatey/` num PR e, depois do merge, reenvie a mesma
versão sem soltar outra:

```sh
gh workflow run chocolatey.yml -R LLawli/medx-sdk-oss -f tag=v0.1.1
```

O workflow usa o nuspec do `master` com o `.zip` daquela tag. Uma versão
ainda em moderação aceita o mesmo número de novo e é substituída.

O binário de Windows não é assinado (o SmartScreen avisa na primeira
execução), e os de macOS não são notarizados: arquivos baixados com curl ou
pelo Homebrew não ficam em quarentena, então o Gatekeeper não os bloqueia.
