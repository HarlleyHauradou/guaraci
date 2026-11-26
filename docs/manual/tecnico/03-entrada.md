# 3. Leitura da entrada

Crate `guaraci-input`.

## Parser

É um parser descendente recursivo escrito à mão (`parser.rs`), orientado a
linhas. Cada nó da árvore guarda um `Span` (arquivo, início, fim), usado em
todas as mensagens.

Regras léxicas relevantes:

- Um número pode ser seguido, na mesma linha, por uma unidade, com ou sem
  espaço. Uma unidade começa por letra, `µ`, `°` ou `%`.
- O expoente `e` só faz parte do número se for seguido de dígitos.
- Palavras sem aspas aceitam letras, dígitos e `_ - . / +`, o que cobre nomes
  como `Tc-99m` e `dose.mhd`.
- `131I` e `99mTc` são lidos pelo parser como número + unidade. A chave
  `nuclide` reconstrói o texto antes de interpretar o nome.

## `include`

A resolução está em `source.rs`:

- Se o nome não tem extensão, tenta-se primeiro `nome.gci`, depois `nome`.
- O caminho é relativo ao arquivo que contém o `include`.
- Arquivos já lidos são ignorados (inclusão em diamante).
- Uma pilha de inclusão detecta ciclos.

Arquivos embutidos (o banco de materiais) são carregados antes do arquivo do
usuário e marcados como *built-in*. Definições do usuário com o mesmo nome os
substituem.

## Da árvore à configuração

O leitor de seção (`reader.rs`) oferece leituras tipadas (`quantity`, `vec3`,
`choice`, `text_list`…) e:

- registra cada chave lida; as que sobram viram "chave desconhecida", com
  sugestão por similaridade de Jaro-Winkler;
- registra cada valor padrão aplicado (`DefaultNote`);
- acumula erros em vez de parar no primeiro.

A montagem da configuração (`config.rs`) também:

- resolve as cadeias de `same_as` dos materiais, detectando ciclos;
- verifica todas as referências cruzadas (volumes, materiais, pais) e os ciclos de parentesco;
- não repete erros em cascata: se a seção de um volume tem erro, as
  referências a esse volume não são reportadas de novo.
