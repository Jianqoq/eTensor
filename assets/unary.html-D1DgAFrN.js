import{_ as r,c as i,a as s,b as n,d as t,o as l,r as o}from"./app-BK9MK9Ew.js";const p={};function c(d,e){const a=o("ChartJS");return l(),i("div",null,[e[0]||(e[0]=s("h1",null,"Unary Benchmark",-1)),n(a,{id:"chartjs-3",config:"eJyllMFunDAQhu/7FJZPGwlZGBsMvVW59NRTpUqNcjCLS6x4MbW9VVZV3r0zCLKLkmg3yoWRxz//zDfG/NsQQtNxNPQLoa0ONMNEp5OGBG7CyunWuAjrOxrtQDNCzdOIwfkeQ/wTEsakhweMvXEHeo9Gs1U0aXp9ypDZ9mSNpb+NYHHKzw3cSVbyssiIZEJJjrEqq2kthWwyUjHYVhkpGeeNmmtODq3ePfbBH4bu1jsfsEbo260qM8IbcIDHzXnF1ofOhI9of9ouPYCWzxvPi+Itvh8+7GA4rwlLpoSUSKAkFpQ5oAGZUEzyCvNNUQM5z1muqisIixJcGnDgQlwiPNN+mvBWD50zZLt/dCurBbRhSlUA0jAhZA4lCyCsMUrWQGyYrApopQDi8mrQIocZ1dVVoO9rPwT6vdMh6CPZjjq8ScoFm+rBkQkkhtiUQFjDiRYCh82qHGPOKlVfQSrwG8yRQPJLpBe0r0ineA/PiZn6MVk/4G2dL78e7F5jDlK/tYtmvtfBxBGE9i/+OVI4LPnRHXp7ZoCTM70ZurMMyny0sytNfqSrdubx07jTzqysjmuX1kCtr+mXCX7VxbSZbHLY3EmPZ2Tj6DTarNWoN09puqp2j59xvFmaWtp6aQ+ntXne/AdQXzZk",title:"Unary%20f32%20Performance%20(size%20%3D%201024%20*%202048%20*%208)",type:"json"}),n(a,{id:"chartjs-6",config:"eJyllD1v2zAQhnf/CoGTAxgHiR8S2a3I0qlTgQINMlAWqwihRZWiixhF/nvvBDm2kAR2kEUHHV+9d8+R1L9VlrF0GBz7krHaRrahRGOTxQQt4pu3tfMjvt+xsevZJmPuaaDgQ0th/BMTxWT7B4qt83t2T0az1ejS9PmUyWbbkzWV/jagxSk/N3BXaCi02WRFBcpIjAZKLTBqyEWxybgCpTnGHDTnc83JobbbxzaGfd/cBh8i1Yhtva4UmeAX+Lg5r1iH2Lj4Ee3PrkkPqC3mheej4i2+HyFucTivCTkSKo0EFQhOBUsFFZIV0oDOMa+BG4xILkpzBSFX6GJoZkJcIjzTfprw1vaNd9l69+gXVkdQUQGX5SYTBqREIJULqHArpSlA4NbiupS0tbmC60F5jpa6vAr0fe2HQL83NkZ7yNaDjW+SqgJ0QUQaKjo+UoASWF/kYIhQCRB0aCWH3JRXkAo6gzkRyOIS6QXtK9Ip3uNzYmZhSF3o6bbOl9/23c5SDlO/rR/dfK+jGwcUdn/pz5Hi/pgf/L7tzgxocq51fXOWIVkYu9mVpTCwRTvz+Nm4td4trA5Ll9phra/pl4th0cW0mLrkqbmTnvaoGwdvyWapJr17StNV7XZ0jMebY1PHtl7ao2mtnlf/AUahNmY=",title:"Unary%20f32%20Performance%20(size%20%3D%204096%20*%202048%20*%208)",type:"json"}),e[1]||(e[1]=t(`<h1>Error precision (lower is better)</h1><ul><li><p><code>Hpt</code>: 10 ulps</p></li><li><p><code>Torch</code>: 10 ulps</p></li><li><p><code>Candle (mkl)</code>: 1 ulps</p></li><li><p><code>Ndarray (par)</code>: 2 ulps</p></li></ul><h1>Compilation config</h1><div class="language-cargo line-numbers-mode" data-highlighter="prismjs" data-ext="cargo" data-title="cargo"><pre><code><span class="line">[profile.release]</span>
<span class="line">opt-level = 3</span>
<span class="line">incremental = true</span>
<span class="line">debug = true</span>
<span class="line">lto = &quot;fat&quot;</span>
<span class="line">codegen-units = 1</span>
<span class="line"></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h1>Running Threads</h1><p><code>10</code></p><h1>Device specification</h1><p><code>CPU</code>: i5-12600k</p><p><code>RAM</code>: G.SKILL Trident Z Royal Series (Intel XMP) DDR4 64GB</p>`,9))])}const u=r(p,[["render",c],["__file","unary.html.vue"]]),h=JSON.parse('{"path":"/benchmarks/unary.html","title":"Unary Benchmark","lang":"zh-CN","frontmatter":{},"headers":[],"git":{"updatedTime":1739329222000,"contributors":[{"name":"Jianqoq","username":"Jianqoq","email":"120760306+Jianqoq@users.noreply.github.com","commits":1,"url":"https://github.com/Jianqoq"}]},"filePathRelative":"benchmarks/unary.md"}');export{u as comp,h as data};
