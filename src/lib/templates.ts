export interface Template {
  id: string;
  label: string;
  content: string;
}

export const defaultTemplates: Template[] = [
  { id: 'goal', label: '① 锁定目标', content: '## ① 锁定目标\n\n目标清晰！\n\n' },
  { id: 'review', label: '② 复盘过程', content: '## ② 复盘过程\n\n做了什么，哪些结果达成了，哪些没做到\n\n' },
  { id: 'analyze', label: '③ 拆解问题', content: '## ③ 拆解问题\n\n连续问自己为什么？\n\n' },
  { id: 'action', label: '④ 优化行动', content: '## ④ 优化行动\n\n把方案变成具体行动 5w1h：做什么，为什么做，什么时候，在哪，谁来做，怎么做\n\n' },
  { id: 'routine', label: '⑤ 形成套路', content: '## ⑤ 形成套路\n\n经验提炼成可以反复使用的方法论，把经验变成反复使用的方法论。\n\n' },
];
