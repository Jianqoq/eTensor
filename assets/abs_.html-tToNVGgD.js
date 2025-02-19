import{_ as n,c as a,d as t,o as p}from"./app-smAh7eZf.js";const e={};function o(c,s){return p(),a("div",null,s[0]||(s[0]=[t(`<h1>abs_</h1><div class="language-rust line-numbers-mode" data-highlighter="prismjs" data-ext="rs" data-title="rs"><pre><code><span class="line"><span class="token function">abs_</span><span class="token punctuation">(</span></span>
<span class="line">    x<span class="token punctuation">:</span> <span class="token operator">&amp;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">,</span> </span>
<span class="line">    out<span class="token punctuation">:</span> <span class="token operator">&amp;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span> <span class="token operator">|</span> <span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span></span>
<span class="line"><span class="token punctuation">)</span> <span class="token punctuation">-&gt;</span> <span class="token class-name">Result</span><span class="token operator">&lt;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token operator">&gt;</span></span>
<span class="line"></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>Calculate absolute of input with out</p><h2>Parameters:</h2><p><code>x</code>: input Tensor <code>out</code>: Tensor to write to</p><h2>Returns:</h2><p>Tensor with type <code>T</code></p><h2>Examples:</h2><div class="language-rust line-numbers-mode" data-highlighter="prismjs" data-ext="rs" data-title="rs"><pre><code><span class="line"><span class="token keyword">use</span> <span class="token namespace">hpt<span class="token punctuation">::</span></span><span class="token punctuation">{</span><span class="token class-name">NormalUaryOps</span><span class="token punctuation">,</span> <span class="token class-name">Tensor</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token punctuation">,</span> <span class="token class-name">TensorInfo</span><span class="token punctuation">}</span><span class="token punctuation">;</span></span>
<span class="line"></span>
<span class="line"><span class="token keyword">fn</span> <span class="token function-definition function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token punctuation">-&gt;</span> <span class="token class-name">Result</span><span class="token operator">&lt;</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token operator">&gt;</span> <span class="token punctuation">{</span></span>
<span class="line">    <span class="token keyword">let</span> a <span class="token operator">=</span> <span class="token class-name">Tensor</span><span class="token punctuation">::</span><span class="token operator">&lt;</span><span class="token keyword">f32</span><span class="token operator">&gt;</span><span class="token punctuation">::</span><span class="token function">new</span><span class="token punctuation">(</span><span class="token punctuation">[</span><span class="token operator">-</span><span class="token number">10.0</span><span class="token punctuation">]</span><span class="token punctuation">)</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token keyword">let</span> b <span class="token operator">=</span> a<span class="token punctuation">.</span><span class="token function">abs_</span><span class="token punctuation">(</span><span class="token operator">&amp;</span><span class="token keyword">mut</span> a<span class="token punctuation">.</span><span class="token function">clone</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span><span class="token operator">?</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token macro property">println!</span><span class="token punctuation">(</span><span class="token string">&quot;{}&quot;</span><span class="token punctuation">,</span> b<span class="token punctuation">)</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token macro property">assert_eq!</span><span class="token punctuation">(</span>a<span class="token punctuation">.</span><span class="token function">ptr</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">.</span>ptr <span class="token keyword">as</span> <span class="token keyword">u64</span><span class="token punctuation">,</span> b<span class="token punctuation">.</span><span class="token function">ptr</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">.</span>ptr <span class="token keyword">as</span> <span class="token keyword">u64</span><span class="token punctuation">)</span><span class="token punctuation">;</span></span>
<span class="line">    <span class="token class-name">Ok</span><span class="token punctuation">(</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span></span>
<span class="line"><span class="token punctuation">}</span></span>
<span class="line"></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2>Backend Support</h2><table><thead><tr><th>Backend</th><th>Supported</th></tr></thead><tbody><tr><td>CPU</td><td>✅</td></tr><tr><td>Cuda</td><td>✅</td></tr></tbody></table>`,11)]))}const u=n(e,[["render",o],["__file","abs_.html.vue"]]),i=JSON.parse('{"path":"/user_guide/unary/abs_.html","title":"abs_","lang":"zh-CN","frontmatter":{},"headers":[{"level":2,"title":"Parameters:","slug":"parameters","link":"#parameters","children":[]},{"level":2,"title":"Returns:","slug":"returns","link":"#returns","children":[]},{"level":2,"title":"Examples:","slug":"examples","link":"#examples","children":[]},{"level":2,"title":"Backend Support","slug":"backend-support","link":"#backend-support","children":[]}],"git":{"updatedTime":1739987937000,"contributors":[{"name":"Jianqoq","username":"Jianqoq","email":"120760306+Jianqoq@users.noreply.github.com","commits":1,"url":"https://github.com/Jianqoq"}]},"filePathRelative":"user_guide/unary/abs_.md"}');export{u as comp,i as data};
