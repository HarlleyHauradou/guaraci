# Volume III — Interface Gráfica

:::{note} Em desenvolvimento
A interface gráfica ainda não existe. Esta página descreve o que ela vai
fazer e como este volume será organizado. Tudo o que a interface faz
também pode ser feito pelo terminal, com os comandos do
[Volume II](../volume-2/index.md).
:::

A interface é um programa desktop escrito em Rust (egui), inspirado no Flair
do FLUKA. Ela trabalha **sobre os mesmos arquivos `.gci`**: nada fica escondido
na interface, e o arquivo de entrada continua sendo o caderno de laboratório
da simulação.

## Capítulos previstos

1. **Visão geral:** janelas, projetos e arquivos.
2. **Editor:**
   - destaque de sintaxe do `.gci`;
   - validação enquanto se digita, com as mesmas mensagens do `guaraci check`;
   - completar nomes de materiais, volumes e nuclídeos.
3. **Geometria:**
   - cortes 2D (axial, coronal, sagital) com as cores dos materiais;
   - vista 3D;
   - seleção de um volume pelo clique, que leva à seção correspondente no editor.
4. **Materiais e nuclídeos:** navegador do banco de materiais e dos dados de
   decaimento (linhas, meia-vida, energias médias).
5. **Execução:** rodar, acompanhar o progresso, ver o registro e interromper
   uma simulação.
6. **Resultados:**
   - mapa de dose sobreposto à geometria, com escala de cores;
   - perfis de dose;
   - histograma dose-volume (DVH);
   - mapa de incerteza;
   - espectros.
7. **Exportação:** figuras, tabelas e relatórios.
