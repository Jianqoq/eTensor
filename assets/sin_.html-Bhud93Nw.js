import{_ as s,o as n,c as a,a as t}from"./app-Bp33fJrM.js";const p={},e=t(`<h1 id="sin" tabindex="-1"><a class="header-anchor" href="#sin" aria-hidden="true">#</a> sin</h1><div class="language-rust line-numbers-mode" data-ext="rs"><pre class="language-rust"><code><span class="token class-name">Tensor</span><span class="token punctuation">::</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">::</span><span class="token function">sin_</span><span class="token punctuation">(</span>x<span class="token punctuation">:</span> <span class="token operator">&amp;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">,</span> out<span class="token punctuation">:</span> <span class="token operator">&amp;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span> <span class="token operator">|</span> <span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">)</span> <span class="token punctuation">-&gt;</span> <span class="token class-name">Result</span><span class="token operator">&lt;</span><span class="token class-name">Tensor</span><span class="token operator">&lt;</span><span class="token class-name">T</span><span class="token operator">&gt;</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token operator">&gt;</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div><p>Trigonometric sine with out</p><h2 id="parameters" tabindex="-1"><a class="header-anchor" href="#parameters" aria-hidden="true">#</a> Parameters:</h2><p><code>x</code>: Angle(radians)</p><h2 id="returns" tabindex="-1"><a class="header-anchor" href="#returns" aria-hidden="true">#</a> Returns:</h2><p>Tensor with type <code>T</code></p><h2 id="examples" tabindex="-1"><a class="header-anchor" href="#examples" aria-hidden="true">#</a> Examples:</h2><div class="language-rust line-numbers-mode" data-ext="rs"><pre class="language-rust"><code><span class="token keyword">use</span> <span class="token namespace">tensor_dyn<span class="token punctuation">::</span></span><span class="token punctuation">{</span><span class="token class-name">FloatUaryOps</span><span class="token punctuation">,</span> <span class="token class-name">Tensor</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token punctuation">}</span><span class="token punctuation">;</span>

<span class="token keyword">fn</span> <span class="token function-definition function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token punctuation">-&gt;</span> <span class="token class-name">Result</span><span class="token operator">&lt;</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">,</span> <span class="token class-name">TensorError</span><span class="token operator">&gt;</span> <span class="token punctuation">{</span>
    <span class="token keyword">let</span> a <span class="token operator">=</span> <span class="token class-name">Tensor</span><span class="token punctuation">::</span><span class="token operator">&lt;</span><span class="token keyword">f32</span><span class="token operator">&gt;</span><span class="token punctuation">::</span><span class="token function">new</span><span class="token punctuation">(</span><span class="token punctuation">[</span><span class="token number">10.0</span><span class="token punctuation">]</span><span class="token punctuation">)</span><span class="token punctuation">;</span>
    <span class="token keyword">let</span> b <span class="token operator">=</span> a<span class="token punctuation">.</span><span class="token function">sin_</span><span class="token punctuation">(</span><span class="token operator">&amp;</span>a<span class="token punctuation">)</span><span class="token operator">?</span><span class="token punctuation">;</span>
    <span class="token macro property">println!</span><span class="token punctuation">(</span><span class="token string">&quot;{}&quot;</span><span class="token punctuation">,</span> b<span class="token punctuation">)</span><span class="token punctuation">;</span>
    <span class="token class-name">Ok</span><span class="token punctuation">(</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span>
<span class="token punctuation">}</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div>`,9),o=[e];function c(l,r){return n(),a("div",null,o)}const u=s(p,[["render",c],["__file","sin_.html.vue"]]);export{u as default};
