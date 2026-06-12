import { Mark, mergeAttributes } from "@tiptap/core";
import { Plugin } from "@tiptap/pm/state";
import { Node as ProseMirrorNode } from "@tiptap/pm/model";

function textOffsetToPos(doc: ProseMirrorNode, offset: number): number | null {
  let current = 0;
  let result: number | null = null;

  doc.descendants((node, pos) => {
    if (result !== null) return false;

    if (node.isText && node.text) {
      const nodeStart = current;
      const nodeEnd = current + node.text.length;

      if (offset >= nodeStart && offset <= nodeEnd) {
        result = pos + (offset - nodeStart);
        return false;
      }

      current = nodeEnd;
    }

    return true;
  });

  return result;
}

export const TaskLinkMark = Mark.create({
  name: "taskLink",

  addAttributes() {
    return {
      title: { default: null },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-task-link]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(
        { "data-task-link": "", class: "task-link-pill" },
        HTMLAttributes
      ),
      0,
    ];
  },

  addProseMirrorPlugins() {
    const markType = this.editor.schema.marks.taskLink;

    return [
      new Plugin({
        appendTransaction: (_transactions, _oldState, newState) => {
          const text = newState.doc.textContent;
          const regex = /\[\[([^\]]+)\]\]/g;

          let modified = false;
          const tr = newState.tr;

          let match;
          while ((match = regex.exec(text)) !== null) {
            const [full, title] = match;
            const startOffset = match.index;
            const endOffset = startOffset + full.length;

            const start = textOffsetToPos(newState.doc, startOffset);
            const end = textOffsetToPos(newState.doc, endOffset);

            if (start === null || end === null) continue;

            if (!newState.doc.rangeHasMark(start, end, markType)) {
              tr.addMark(start, end, markType.create({ title }));
              modified = true;
            }
          }

          return modified ? tr : null;
        },
      }),
    ];
  },
});
