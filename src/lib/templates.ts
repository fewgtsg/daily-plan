export interface Template {
  id: string;
  label: string;
  content: string;
}

export const defaultTemplates: Template[] = [
  { id: 'goal', label: '① 锁定目标', content: '<h2>① 锁定目标</h2><p>目标清晰！</p>' },
  { id: 'review', label: '② 复盘过程', content: '<h2>② 复盘过程</h2><p>做了什么，哪些结果达成了，哪些没做到</p>' },
  { id: 'analyze', label: '③ 拆解问题', content: '<h2>③ 拆解问题</h2><p>连续问自己为什么？</p>' },
  { id: 'action', label: '④ 优化行动', content: '<h2>④ 优化行动</h2><p>把方案变成具体行动 5w1h：做什么，为什么做，什么时候，在哪，谁来做，怎么做</p>' },
  { id: 'routine', label: '⑤ 形成套路', content: '<h2>⑤ 形成套路</h2><p>经验提炼成可以反复使用的方法论，把经验变成反复使用的方法论。</p>' },
];
