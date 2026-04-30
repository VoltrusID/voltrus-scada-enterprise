var Mr=globalThis,wr=Mr.ShadowRoot&&(Mr.ShadyCSS===void 0||Mr.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,Vo=Symbol(),zc=new WeakMap,ws=class{constructor(t,e,n){if(this._$cssResult$=!0,n!==Vo)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=t,this.t=e}get styleSheet(){let t=this.o,e=this.t;if(wr&&t===void 0){let n=e!==void 0&&e.length===1;n&&(t=zc.get(e)),t===void 0&&((this.o=t=new CSSStyleSheet).replaceSync(this.cssText),n&&zc.set(e,t))}return t}toString(){return this.cssText}},Vc=i=>new ws(typeof i=="string"?i:i+"",void 0,Vo),qt=(i,...t)=>{let e=i.length===1?i[0]:t.reduce((n,s,r)=>n+(a=>{if(a._$cssResult$===!0)return a.cssText;if(typeof a=="number")return a;throw Error("Value passed to 'css' function must be a 'css' function result: "+a+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(s)+i[r+1],i[0]);return new ws(e,i,Vo)},Gc=(i,t)=>{if(wr)i.adoptedStyleSheets=t.map(e=>e instanceof CSSStyleSheet?e:e.styleSheet);else for(let e of t){let n=document.createElement("style"),s=Mr.litNonce;s!==void 0&&n.setAttribute("nonce",s),n.textContent=e.cssText,i.appendChild(n)}},Go=wr?i=>i:i=>i instanceof CSSStyleSheet?(t=>{let e="";for(let n of t.cssRules)e+=n.cssText;return Vc(e)})(i):i;var{is:eu,defineProperty:nu,getOwnPropertyDescriptor:iu,getOwnPropertyNames:su,getOwnPropertySymbols:ru,getPrototypeOf:au}=Object,Ar=globalThis,Hc=Ar.trustedTypes,ou=Hc?Hc.emptyScript:"",lu=Ar.reactiveElementPolyfillSupport,As=(i,t)=>i,Es={toAttribute(i,t){switch(t){case Boolean:i=i?ou:null;break;case Object:case Array:i=i==null?i:JSON.stringify(i)}return i},fromAttribute(i,t){let e=i;switch(t){case Boolean:e=i!==null;break;case Number:e=i===null?null:Number(i);break;case Object:case Array:try{e=JSON.parse(i)}catch{e=null}}return e}},Er=(i,t)=>!eu(i,t),Wc={attribute:!0,type:String,converter:Es,reflect:!1,useDefault:!1,hasChanged:Er};Symbol.metadata??=Symbol("metadata"),Ar.litPropertyMetadata??=new WeakMap;var Gn=class extends HTMLElement{static addInitializer(t){this._$Ei(),(this.l??=[]).push(t)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(t,e=Wc){if(e.state&&(e.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(t)&&((e=Object.create(e)).wrapped=!0),this.elementProperties.set(t,e),!e.noAccessor){let n=Symbol(),s=this.getPropertyDescriptor(t,n,e);s!==void 0&&nu(this.prototype,t,s)}}static getPropertyDescriptor(t,e,n){let{get:s,set:r}=iu(this.prototype,t)??{get(){return this[e]},set(a){this[e]=a}};return{get:s,set(a){let o=s?.call(this);r?.call(this,a),this.requestUpdate(t,o,n)},configurable:!0,enumerable:!0}}static getPropertyOptions(t){return this.elementProperties.get(t)??Wc}static _$Ei(){if(this.hasOwnProperty(As("elementProperties")))return;let t=au(this);t.finalize(),t.l!==void 0&&(this.l=[...t.l]),this.elementProperties=new Map(t.elementProperties)}static finalize(){if(this.hasOwnProperty(As("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(As("properties"))){let e=this.properties,n=[...su(e),...ru(e)];for(let s of n)this.createProperty(s,e[s])}let t=this[Symbol.metadata];if(t!==null){let e=litPropertyMetadata.get(t);if(e!==void 0)for(let[n,s]of e)this.elementProperties.set(n,s)}this._$Eh=new Map;for(let[e,n]of this.elementProperties){let s=this._$Eu(e,n);s!==void 0&&this._$Eh.set(s,e)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(t){let e=[];if(Array.isArray(t)){let n=new Set(t.flat(1/0).reverse());for(let s of n)e.unshift(Go(s))}else t!==void 0&&e.push(Go(t));return e}static _$Eu(t,e){let n=e.attribute;return n===!1?void 0:typeof n=="string"?n:typeof t=="string"?t.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(t=>this.enableUpdating=t),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(t=>t(this))}addController(t){(this._$EO??=new Set).add(t),this.renderRoot!==void 0&&this.isConnected&&t.hostConnected?.()}removeController(t){this._$EO?.delete(t)}_$E_(){let t=new Map,e=this.constructor.elementProperties;for(let n of e.keys())this.hasOwnProperty(n)&&(t.set(n,this[n]),delete this[n]);t.size>0&&(this._$Ep=t)}createRenderRoot(){let t=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return Gc(t,this.constructor.elementStyles),t}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(t=>t.hostConnected?.())}enableUpdating(t){}disconnectedCallback(){this._$EO?.forEach(t=>t.hostDisconnected?.())}attributeChangedCallback(t,e,n){this._$AK(t,n)}_$ET(t,e){let n=this.constructor.elementProperties.get(t),s=this.constructor._$Eu(t,n);if(s!==void 0&&n.reflect===!0){let r=(n.converter?.toAttribute!==void 0?n.converter:Es).toAttribute(e,n.type);this._$Em=t,r==null?this.removeAttribute(s):this.setAttribute(s,r),this._$Em=null}}_$AK(t,e){let n=this.constructor,s=n._$Eh.get(t);if(s!==void 0&&this._$Em!==s){let r=n.getPropertyOptions(s),a=typeof r.converter=="function"?{fromAttribute:r.converter}:r.converter?.fromAttribute!==void 0?r.converter:Es;this._$Em=s;let o=a.fromAttribute(e,r.type);this[s]=o??this._$Ej?.get(s)??o,this._$Em=null}}requestUpdate(t,e,n,s=!1,r){if(t!==void 0){let a=this.constructor;if(s===!1&&(r=this[t]),n??=a.getPropertyOptions(t),!((n.hasChanged??Er)(r,e)||n.useDefault&&n.reflect&&r===this._$Ej?.get(t)&&!this.hasAttribute(a._$Eu(t,n))))return;this.C(t,e,n)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(t,e,{useDefault:n,reflect:s,wrapped:r},a){n&&!(this._$Ej??=new Map).has(t)&&(this._$Ej.set(t,a??e??this[t]),r!==!0||a!==void 0)||(this._$AL.has(t)||(this.hasUpdated||n||(e=void 0),this._$AL.set(t,e)),s===!0&&this._$Em!==t&&(this._$Eq??=new Set).add(t))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(e){Promise.reject(e)}let t=this.scheduleUpdate();return t!=null&&await t,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(let[s,r]of this._$Ep)this[s]=r;this._$Ep=void 0}let n=this.constructor.elementProperties;if(n.size>0)for(let[s,r]of n){let{wrapped:a}=r,o=this[s];a!==!0||this._$AL.has(s)||o===void 0||this.C(s,void 0,r,o)}}let t=!1,e=this._$AL;try{t=this.shouldUpdate(e),t?(this.willUpdate(e),this._$EO?.forEach(n=>n.hostUpdate?.()),this.update(e)):this._$EM()}catch(n){throw t=!1,this._$EM(),n}t&&this._$AE(e)}willUpdate(t){}_$AE(t){this._$EO?.forEach(e=>e.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(t)),this.updated(t)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(t){return!0}update(t){this._$Eq&&=this._$Eq.forEach(e=>this._$ET(e,this[e])),this._$EM()}updated(t){}firstUpdated(t){}};Gn.elementStyles=[],Gn.shadowRootOptions={mode:"open"},Gn[As("elementProperties")]=new Map,Gn[As("finalized")]=new Map,lu?.({ReactiveElement:Gn}),(Ar.reactiveElementVersions??=[]).push("2.1.2");var Zo=globalThis,$c=i=>i,Tr=Zo.trustedTypes,Xc=Tr?Tr.createPolicy("lit-html",{createHTML:i=>i}):void 0,jc="$lit$",ii=`lit$${Math.random().toFixed(9).slice(2)}$`,Qc="?"+ii,cu=`<${Qc}>`,Ei=document,Cs=()=>Ei.createComment(""),Rs=i=>i===null||typeof i!="object"&&typeof i!="function",Jo=Array.isArray,hu=i=>Jo(i)||typeof i?.[Symbol.iterator]=="function",Ho=`[ 	
\f\r]`,Ts=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,qc=/-->/g,Yc=/>/g,wi=RegExp(`>|${Ho}(?:([^\\s"'>=/]+)(${Ho}*=${Ho}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),Zc=/'/g,Jc=/"/g,th=/^(?:script|style|textarea|title)$/i,Ko=i=>(t,...e)=>({_$litType$:i,strings:t,values:e}),Z=Ko(1),X_=Ko(2),q_=Ko(3),Ti=Symbol.for("lit-noChange"),Te=Symbol.for("lit-nothing"),Kc=new WeakMap,Ai=Ei.createTreeWalker(Ei,129);function eh(i,t){if(!Jo(i)||!i.hasOwnProperty("raw"))throw Error("invalid template strings array");return Xc!==void 0?Xc.createHTML(t):t}var du=(i,t)=>{let e=i.length-1,n=[],s,r=t===2?"<svg>":t===3?"<math>":"",a=Ts;for(let o=0;o<e;o++){let l=i[o],c,d,u=-1,h=0;for(;h<l.length&&(a.lastIndex=h,d=a.exec(l),d!==null);)h=a.lastIndex,a===Ts?d[1]==="!--"?a=qc:d[1]!==void 0?a=Yc:d[2]!==void 0?(th.test(d[2])&&(s=RegExp("</"+d[2],"g")),a=wi):d[3]!==void 0&&(a=wi):a===wi?d[0]===">"?(a=s??Ts,u=-1):d[1]===void 0?u=-2:(u=a.lastIndex-d[2].length,c=d[1],a=d[3]===void 0?wi:d[3]==='"'?Jc:Zc):a===Jc||a===Zc?a=wi:a===qc||a===Yc?a=Ts:(a=wi,s=void 0);let p=a===wi&&i[o+1].startsWith("/>")?" ":"";r+=a===Ts?l+cu:u>=0?(n.push(c),l.slice(0,u)+jc+l.slice(u)+ii+p):l+ii+(u===-2?o:p)}return[eh(i,r+(i[e]||"<?>")+(t===2?"</svg>":t===3?"</math>":"")),n]},Is=class i{constructor({strings:t,_$litType$:e},n){let s;this.parts=[];let r=0,a=0,o=t.length-1,l=this.parts,[c,d]=du(t,e);if(this.el=i.createElement(c,n),Ai.currentNode=this.el.content,e===2||e===3){let u=this.el.content.firstChild;u.replaceWith(...u.childNodes)}for(;(s=Ai.nextNode())!==null&&l.length<o;){if(s.nodeType===1){if(s.hasAttributes())for(let u of s.getAttributeNames())if(u.endsWith(jc)){let h=d[a++],p=s.getAttribute(u).split(ii),_=/([.?@])?(.*)/.exec(h);l.push({type:1,index:r,name:_[2],strings:p,ctor:_[1]==="."?$o:_[1]==="?"?Xo:_[1]==="@"?qo:Zi}),s.removeAttribute(u)}else u.startsWith(ii)&&(l.push({type:6,index:r}),s.removeAttribute(u));if(th.test(s.tagName)){let u=s.textContent.split(ii),h=u.length-1;if(h>0){s.textContent=Tr?Tr.emptyScript:"";for(let p=0;p<h;p++)s.append(u[p],Cs()),Ai.nextNode(),l.push({type:2,index:++r});s.append(u[h],Cs())}}}else if(s.nodeType===8)if(s.data===Qc)l.push({type:2,index:r});else{let u=-1;for(;(u=s.data.indexOf(ii,u+1))!==-1;)l.push({type:7,index:r}),u+=ii.length-1}r++}}static createElement(t,e){let n=Ei.createElement("template");return n.innerHTML=t,n}};function Yi(i,t,e=i,n){if(t===Ti)return t;let s=n!==void 0?e._$Co?.[n]:e._$Cl,r=Rs(t)?void 0:t._$litDirective$;return s?.constructor!==r&&(s?._$AO?.(!1),r===void 0?s=void 0:(s=new r(i),s._$AT(i,e,n)),n!==void 0?(e._$Co??=[])[n]=s:e._$Cl=s),s!==void 0&&(t=Yi(i,s._$AS(i,t.values),s,n)),t}var Wo=class{constructor(t,e){this._$AV=[],this._$AN=void 0,this._$AD=t,this._$AM=e}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(t){let{el:{content:e},parts:n}=this._$AD,s=(t?.creationScope??Ei).importNode(e,!0);Ai.currentNode=s;let r=Ai.nextNode(),a=0,o=0,l=n[0];for(;l!==void 0;){if(a===l.index){let c;l.type===2?c=new Ps(r,r.nextSibling,this,t):l.type===1?c=new l.ctor(r,l.name,l.strings,this,t):l.type===6&&(c=new Yo(r,this,t)),this._$AV.push(c),l=n[++o]}a!==l?.index&&(r=Ai.nextNode(),a++)}return Ai.currentNode=Ei,s}p(t){let e=0;for(let n of this._$AV)n!==void 0&&(n.strings!==void 0?(n._$AI(t,n,e),e+=n.strings.length-2):n._$AI(t[e])),e++}},Ps=class i{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(t,e,n,s){this.type=2,this._$AH=Te,this._$AN=void 0,this._$AA=t,this._$AB=e,this._$AM=n,this.options=s,this._$Cv=s?.isConnected??!0}get parentNode(){let t=this._$AA.parentNode,e=this._$AM;return e!==void 0&&t?.nodeType===11&&(t=e.parentNode),t}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(t,e=this){t=Yi(this,t,e),Rs(t)?t===Te||t==null||t===""?(this._$AH!==Te&&this._$AR(),this._$AH=Te):t!==this._$AH&&t!==Ti&&this._(t):t._$litType$!==void 0?this.$(t):t.nodeType!==void 0?this.T(t):hu(t)?this.k(t):this._(t)}O(t){return this._$AA.parentNode.insertBefore(t,this._$AB)}T(t){this._$AH!==t&&(this._$AR(),this._$AH=this.O(t))}_(t){this._$AH!==Te&&Rs(this._$AH)?this._$AA.nextSibling.data=t:this.T(Ei.createTextNode(t)),this._$AH=t}$(t){let{values:e,_$litType$:n}=t,s=typeof n=="number"?this._$AC(t):(n.el===void 0&&(n.el=Is.createElement(eh(n.h,n.h[0]),this.options)),n);if(this._$AH?._$AD===s)this._$AH.p(e);else{let r=new Wo(s,this),a=r.u(this.options);r.p(e),this.T(a),this._$AH=r}}_$AC(t){let e=Kc.get(t.strings);return e===void 0&&Kc.set(t.strings,e=new Is(t)),e}k(t){Jo(this._$AH)||(this._$AH=[],this._$AR());let e=this._$AH,n,s=0;for(let r of t)s===e.length?e.push(n=new i(this.O(Cs()),this.O(Cs()),this,this.options)):n=e[s],n._$AI(r),s++;s<e.length&&(this._$AR(n&&n._$AB.nextSibling,s),e.length=s)}_$AR(t=this._$AA.nextSibling,e){for(this._$AP?.(!1,!0,e);t!==this._$AB;){let n=$c(t).nextSibling;$c(t).remove(),t=n}}setConnected(t){this._$AM===void 0&&(this._$Cv=t,this._$AP?.(t))}},Zi=class{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(t,e,n,s,r){this.type=1,this._$AH=Te,this._$AN=void 0,this.element=t,this.name=e,this._$AM=s,this.options=r,n.length>2||n[0]!==""||n[1]!==""?(this._$AH=Array(n.length-1).fill(new String),this.strings=n):this._$AH=Te}_$AI(t,e=this,n,s){let r=this.strings,a=!1;if(r===void 0)t=Yi(this,t,e,0),a=!Rs(t)||t!==this._$AH&&t!==Ti,a&&(this._$AH=t);else{let o=t,l,c;for(t=r[0],l=0;l<r.length-1;l++)c=Yi(this,o[n+l],e,l),c===Ti&&(c=this._$AH[l]),a||=!Rs(c)||c!==this._$AH[l],c===Te?t=Te:t!==Te&&(t+=(c??"")+r[l+1]),this._$AH[l]=c}a&&!s&&this.j(t)}j(t){t===Te?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,t??"")}},$o=class extends Zi{constructor(){super(...arguments),this.type=3}j(t){this.element[this.name]=t===Te?void 0:t}},Xo=class extends Zi{constructor(){super(...arguments),this.type=4}j(t){this.element.toggleAttribute(this.name,!!t&&t!==Te)}},qo=class extends Zi{constructor(t,e,n,s,r){super(t,e,n,s,r),this.type=5}_$AI(t,e=this){if((t=Yi(this,t,e,0)??Te)===Ti)return;let n=this._$AH,s=t===Te&&n!==Te||t.capture!==n.capture||t.once!==n.once||t.passive!==n.passive,r=t!==Te&&(n===Te||s);s&&this.element.removeEventListener(this.name,this,n),r&&this.element.addEventListener(this.name,this,t),this._$AH=t}handleEvent(t){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,t):this._$AH.handleEvent(t)}},Yo=class{constructor(t,e,n){this.element=t,this.type=6,this._$AN=void 0,this._$AM=e,this.options=n}get _$AU(){return this._$AM._$AU}_$AI(t){Yi(this,t)}};var uu=Zo.litHtmlPolyfillSupport;uu?.(Is,Ps),(Zo.litHtmlVersions??=[]).push("3.3.2");var nh=(i,t,e)=>{let n=e?.renderBefore??t,s=n._$litPart$;if(s===void 0){let r=e?.renderBefore??null;n._$litPart$=s=new Ps(t.insertBefore(Cs(),r),r,void 0,e??{})}return s._$AI(i),s};var jo=globalThis,kt=class extends Gn{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){let t=super.createRenderRoot();return this.renderOptions.renderBefore??=t.firstChild,t}update(t){let e=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(t),this._$Do=nh(e,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return Ti}};kt._$litElement$=!0,kt.finalized=!0,jo.litElementHydrateSupport?.({LitElement:kt});var fu=jo.litElementPolyfillSupport;fu?.({LitElement:kt});(jo.litElementVersions??=[]).push("4.2.2");var Zt=i=>(t,e)=>{e!==void 0?e.addInitializer(()=>{customElements.define(i,t)}):customElements.define(i,t)};var pu={attribute:!0,type:String,converter:Es,reflect:!1,hasChanged:Er},mu=(i=pu,t,e)=>{let{kind:n,metadata:s}=e,r=globalThis.litPropertyMetadata.get(s);if(r===void 0&&globalThis.litPropertyMetadata.set(s,r=new Map),n==="setter"&&((i=Object.create(i)).wrapped=!0),r.set(e.name,i),n==="accessor"){let{name:a}=e;return{set(o){let l=t.get.call(this);t.set.call(this,o),this.requestUpdate(a,l,i,!0,o)},init(o){return o!==void 0&&this.C(a,void 0,i,o),o}}}if(n==="setter"){let{name:a}=e;return function(o){let l=this[a];t.call(this,o),this.requestUpdate(a,l,i,!0,o)}}throw Error("Unsupported decorator location: "+n)};function le(i){return(t,e)=>typeof e=="object"?mu(i,t,e):((n,s,r)=>{let a=s.hasOwnProperty(r);return s.constructor.createProperty(r,n),a?Object.getOwnPropertyDescriptor(s,r):void 0})(i,t,e)}function et(i){return le({...i,state:!0,attribute:!1})}var Ci=(i,t,e)=>(e.configurable=!0,e.enumerable=!0,Reflect.decorate&&typeof t!="object"&&Object.defineProperty(i,t,e),e);function Ji(i,t){return(e,n,s)=>{let r=a=>a.renderRoot?.querySelector(i)??null;if(t){let{get:a,set:o}=typeof n=="object"?e:s??(()=>{let l=Symbol();return{get(){return this[l]},set(c){this[l]=c}}})();return Ci(e,n,{get(){let l=a.call(this);return l===void 0&&(l=r(this),(l!==null||this.hasUpdated)&&o.call(this,l)),l}})}return Ci(e,n,{get(){return r(this)}})}}var Cr=window.location.origin,Qo=class extends EventTarget{constructor(){super(...arguments),this._state={authed:!1,role:"user",loading:!0,error:""}}get state(){return{...this._state}}_set(t){this._state={...this._state,...t},this.dispatchEvent(new CustomEvent("change",{detail:this._state}))}async check(){try{let e=await(await fetch(`${Cr}/api/auth/me`,{credentials:"include"})).json();this._set({authed:e.authenticated===!0,role:e.role||"user",loading:!1,error:""})}catch{this._set({authed:!1,role:"user",loading:!1,error:""})}}async login(t,e){this._set({error:""});try{let n=await fetch(`${Cr}/api/auth/login`,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify({username:t,password:e})});if(n.ok){let s=await n.json();this._set({authed:!0,role:s.role||"user",error:""}),fetch(`${Cr}/api/seed`,{method:"POST",credentials:"include"}).catch(()=>{})}else{let s=await n.json().catch(()=>({}));this._set({error:s.error||"Login failed"})}}catch{this._set({error:"Network error"})}}async logout(){try{await fetch(`${Cr}/api/auth/logout`,{method:"POST",credentials:"include"})}catch{}this._set({authed:!1,role:"user",error:""})}},An=new Qo;var el=window.location.origin,tl=class extends EventTarget{constructor(){super(...arguments),this._es=null,this._url=`${el}/api/stream`}connect(){this._es||(this._es=new EventSource(this._url,{withCredentials:!0}),this._es.onmessage=t=>{try{let e=JSON.parse(t.data),n=Array.isArray(e)?e:e.sensors??[];this.dispatchEvent(new CustomEvent("data",{detail:n}))}catch{}},this._es.onerror=()=>{})}disconnect(){this._es?.close(),this._es=null}},en=new tl;async function Kt(i,t){let e=await fetch(`${el}/api${i}`,{credentials:"include",headers:{"Content-Type":"application/json",...t?.headers??{}},...t});if(!e.ok)throw new Error(`API ${i} failed: ${e.status}`);return e.json()}async function Ie(i,t){let e=await fetch(`${el}${i}`,{credentials:"include",...t});if(!e.ok){let n=await e.json().catch(()=>({}));throw new Error(n.error||`Request failed: ${e.status}`)}return e}var nl=class{constructor(){this._audio=null,this._muted=!1,this._playing=!1,typeof Audio<"u"&&(this._audio=new Audio("alarm.wav"),this._audio.loop=!0,this._audio.volume=.5)}start(){!this._audio||this._muted||this._playing||(this._audio.play().catch(()=>{}),this._playing=!0)}stop(){this._audio&&(this._audio.pause(),this._audio.currentTime=0,this._playing=!1)}toggleMute(){return this._muted=!this._muted,this._muted&&this.stop(),this._muted}get isMuted(){return this._muted}get isPlaying(){return this._playing}},Ri=new nl;var il=class extends EventTarget{constructor(){super(...arguments),this._branding={title:"Voltrus Lite",subtitle:"Industrial monitoring",logo:""},this._shortcuts={},this._navShortcuts={}}get branding(){return{...this._branding}}get shortcuts(){return{...this._shortcuts}}get navShortcuts(){return{...this._navShortcuts}}setBranding(t){this._branding={...t},this.dispatchEvent(new CustomEvent("branding",{detail:this._branding}))}setShortcuts(t){this._shortcuts={...t},this.dispatchEvent(new CustomEvent("shortcuts",{detail:this._shortcuts}))}setNavShortcuts(t){this._navShortcuts={...t},this.dispatchEvent(new CustomEvent("navShortcuts",{detail:this._navShortcuts}))}},En=new il;var Rr=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Ls=class extends kt{constructor(){super(...arguments),this._user="",this._pass="",this._error="",this._onAuthChange=t=>{this._error=t.detail.error||""}}static{this.styles=qt`
    :host {
      display: flex; align-items: center; justify-content: center;
      height: 100vh; background: var(--bg);
    }
    form {
      background: var(--card); border: 1px solid var(--border); border-radius: 12px;
      padding: 32px; width: 340px; display: flex; flex-direction: column; gap: 12px;
    }
    .header {
      display: flex; align-items: center; gap: 10; margin-bottom: 12px;
    }
    .header img { width: 36px; height: 36px; border-radius: 8px; object-fit: contain; }
    .header span { font-size: 18px; font-weight: 700; color: var(--fg); }
    label { display: flex; flex-direction: column; gap: 4px; }
    .label-text {
      font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em;
    }
    input {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
      padding: 6px 8px; color: var(--fg); font-size: 13px; font-family: inherit;
      outline: none; width: 100%; box-sizing: border-box;
    }
    input:focus { border-color: var(--border-light); }
    .error { color: var(--crit); font-size: 12px; }
    button[type="submit"] {
      justify-content: center; width: 100%; margin-top: 4px;
      background: var(--cyan); color: #000; border-color: var(--cyan);
      font-weight: 600; cursor: pointer;
      font-family: inherit; font-size: 13px; padding: 7px 14px; border-radius: 6px;
      border: 1px solid var(--cyan);
    }
  `}connectedCallback(){super.connectedCallback(),An.addEventListener("change",this._onAuthChange)}disconnectedCallback(){super.disconnectedCallback(),An.removeEventListener("change",this._onAuthChange)}_submit(t){t.preventDefault(),An.login(this._user,this._pass)}render(){return Z`
      <form @submit="${this._submit}">
        <div class="header">
          <img src="favicon-32.png" alt="Voltrus Lite" />
          <span>Voltrus Lite</span>
        </div>
        <label>
          <span class="label-text">Username</span>
          <input .value="${this._user}" @input="${t=>this._user=t.target.value}" autocomplete="username" autofocus />
        </label>
        <label>
          <span class="label-text">Password</span>
          <input type="password" .value="${this._pass}" @input="${t=>this._pass=t.target.value}" autocomplete="current-password" />
        </label>
        ${this._error?Z`<div class="error">${this._error}</div>`:""}
        <button type="submit">Sign In</button>
      </form>
    `}};Rr([et()],Ls.prototype,"_user",void 0);Rr([et()],Ls.prototype,"_pass",void 0);Rr([et()],Ls.prototype,"_error",void 0);Ls=Rr([Zt("vlt-login-screen")],Ls);var si={modbus:{label:"Modbus",tag:"REAL",color:"var(--ok)"},random_walk:{label:"Random Walk",tag:"SIM",color:"#f59e0b"},sine:{label:"Sine Wave",tag:"SIM",color:"#f59e0b"},step:{label:"Step / Spike",tag:"SIM",color:"#f59e0b"},sawtooth:{label:"Sawtooth",tag:"SIM",color:"#f59e0b"},constant:{label:"Constant",tag:"SIM",color:"#f59e0b"},noise:{label:"Noise",tag:"SIM",color:"#f59e0b"},api:{label:"API",tag:"API",color:"#a78bfa"},none:{label:"None (static)",tag:"NONE",color:"var(--muted)"}},ih=["sparkline","bignumber","gauge","image","text"],sh=["random_walk","sine","step","sawtooth","constant","noise"];function Pr(i,t,e){return i>=e?"critical":i>=t?"warning":"normal"}function Ii(i){return i==="critical"?"var(--crit)":i==="warning"?"var(--warn)":i==="offline"?"var(--muted)":"var(--ok)"}function Pi(i){if(!i.length)return[0,0];let t=1/0,e=-1/0;for(let n of i)n<t&&(t=n),n>e&&(e=n);return[t,e]}function Ds(i){return new Date(i).toLocaleTimeString("en-US",{hour12:!1,hour:"2-digit",minute:"2-digit",second:"2-digit"})}var Ir=typeof document<"u"?document.createElement("canvas").getContext("2d"):null;function Tn(i){return Ir?(Ir.fillStyle="#000000",Ir.fillStyle=i,Ir.fillStyle):i}var Lr=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},gu=["all","active","acknowledged"],Ns=class extends kt{constructor(){super(...arguments),this._filter="active",this._alarms=[],this._loading=!1}static{this.styles=qt`
    :host {
      position: fixed; inset: 0; z-index: 1000;
      display: flex; justify-content: flex-end;
    }
    .backdrop { flex: 1; background: rgba(0,0,0,.5); }
    .panel {
      width: 360px; height: 100%; background: var(--bg);
      border-left: 1px solid var(--border); display: flex; flex-direction: column; overflow: hidden;
    }
    .header {
      padding: 16px 16px 12px; border-bottom: 1px solid var(--border);
      display: flex; justify-content: space-between; align-items: center;
    }
    .header span { font-size: 15px; font-weight: 700; color: var(--fg); }
    .header button {
      padding: 4px 12px; border-radius: 6px; border: 1px solid var(--warn);
      background: transparent; color: var(--warn); cursor: pointer; font-size: 11px; font-weight: 600;
    }
    .header .close {
      background: none; border: none; color: var(--muted); font-size: 18px; padding: 0 4px; line-height: 1;
    }
    .filters {
      display: flex; border-bottom: 1px solid var(--border);
    }
    .filters button {
      flex: 1; padding: 8px 0; font-size: 11px; font-weight: 600; text-transform: uppercase;
      letter-spacing: 0.04em; cursor: pointer; border: none; background: transparent;
      border-bottom: 2px solid transparent; color: var(--muted);
    }
    .filters button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .list { flex: 1; overflow: auto; padding: 8px; }
    .alarm {
      padding: 12px; margin-bottom: 8px; border-radius: 8px;
      border: 1px solid var(--border); background: var(--card);
    }
    .alarm.active-critical { border-color: var(--crit); background: rgba(239,68,68,.15); }
    .alarm.active-warning { border-color: var(--warn); background: rgba(234,179,8,.15); }
    .alarm.resolved { opacity: 0.6; }
    .alarm-header {
      display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;
    }
    .alarm-title { font-size: 13px; font-weight: 600; color: var(--fg); margin-bottom: 4px; }
    .alarm-msg { font-size: 12px; color: var(--muted); margin-bottom: 8px; }
    .pulse { width: 8px; height: 8px; border-radius: 50%; background: var(--crit); animation: pulse 1.5s infinite; }
    .sev { padding: 2px 8px; border-radius: 4px; font-size: 10px; font-weight: 700; text-transform: uppercase; }
    .ack-btn {
      padding: 4px 12px; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer;
      background: transparent; border: 1px solid currentColor;
    }
    @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
  `}connectedCallback(){super.connectedCallback(),this._loadAlarms();let t=setInterval(()=>this._loadAlarms(),5e3);this._cleanup=()=>clearInterval(t)}disconnectedCallback(){super.disconnectedCallback(),this._cleanup?.()}async _loadAlarms(){this._loading=!0;try{this._alarms=await Kt(`/alarms?status=${this._filter}&limit=50`)}catch{this._alarms=[]}this._loading=!1}async _ack(t){try{await fetch(`http://localhost:3100/api/alarms/${t}/acknowledge`,{method:"POST",credentials:"include"}),this._loadAlarms()}catch{}}async _ackAll(){try{await fetch("http://localhost:3100/api/alarms",{method:"POST",credentials:"include"}),this._loadAlarms()}catch{}}_setFilter(t){this._filter=t,this._loadAlarms()}_close(){this.dispatchEvent(new CustomEvent("close"))}render(){return Z`
      <div class="backdrop" @click="${this._close}"></div>
      <div class="panel">
        <div class="header">
          <span>Alarms</span>
          <div style="display:flex;gap:8px">
            <button @click="${this._ackAll}">Ack All</button>
            <button class="close" @click="${this._close}">&times;</button>
          </div>
        </div>
        <div class="filters">
          ${gu.map(t=>Z`
            <button class="${this._filter===t?"active":""}" @click="${()=>this._setFilter(t)}">${t}</button>
          `)}
        </div>
        <div class="list">
          ${this._loading&&this._alarms.length===0?Z`<div style="padding:20px;color:var(--muted);text-align:center">Loading...</div>`:""}
          ${!this._loading&&this._alarms.length===0?Z`<div style="padding:20px;color:var(--muted);text-align:center">No alarms</div>`:""}
          ${this._alarms.map(t=>{let e=t.severity==="critical",n=!t.acknowledged_at,s=!!t.resolved_at,r=e?"var(--crit)":"var(--warn)";return Z`
              <div class="alarm ${n?e?"active-critical":"active-warning":s?"resolved":""}">
                <div class="alarm-header">
                  <div style="display:flex;align-items:center;gap:6px">
                    ${n?Z`<div class="pulse" style="background:${r}"></div>`:""}
                    <span class="sev" style="background:${r}18;color:${r}">${t.severity}</span>
                  </div>
                  <span style="font-size:11px;color:var(--muted)">${Ds(t.triggered_at*1e3)}</span>
                </div>
                <div class="alarm-title">${t.source_name}</div>
                <div class="alarm-msg">${t.message}</div>
                ${n?Z`
                  <button class="ack-btn" style="color:${r};border-color:${r}" @click="${()=>this._ack(t.id)}">Acknowledge</button>
                `:Z`
                  <span style="font-size:11px;color:var(--ok)">&#10003; Acknowledged${t.acknowledged_by?` by ${t.acknowledged_by}`:""}</span>
                `}
              </div>
            `})}
        </div>
      </div>
    `}};Lr([et()],Ns.prototype,"_filter",void 0);Lr([et()],Ns.prototype,"_alarms",void 0);Lr([et()],Ns.prototype,"_loading",void 0);Ns=Lr([Zt("vlt-alarm-panel")],Ns);var dn=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},nn=class extends kt{constructor(){super(...arguments),this.data=[],this.color="#22c55e",this.warn=0,this.crit=0,this.unit="",this.overlays=[],this.pointWidth=0,this._area={pL:60,pR:20,pT:20,pB:44,cW:0,cH:0,yMin:0,yMax:0,dLen:0},this._hover=null}static{this.styles=qt`
    :host { display: block; width: 100%; height: 100%; position: relative; }
    .scroll { width: 100%; height: 100%; overflow-x: auto; overflow-y: hidden; position: relative; }
    .wrapper { position: relative; height: 100%; }
    canvas { display: block; width: 100%; height: 100%; cursor: crosshair; }
    .legend {
      position: absolute; top: 8px; right: 24px; display: flex; gap: 10; pointer-events: none; z-index: 1;
    }
    .legend-item {
      display: flex; align-items: center; gap: 4; font-size: 11px;
      font-family: 'JetBrains Mono', monospace; opacity: 0.85;
      background: rgba(17,17,19,0.8); padding: 2px 6px; border-radius: 4;
    }
    .tooltip {
      position: fixed; background: rgba(17,17,19,0.95); border: 1px solid var(--border);
      border-radius: 8px; padding: 8px 12px; pointer-events: none; z-index: 9999;
      font-family: 'JetBrains Mono', monospace; font-size: 12px; white-space: nowrap;
    }
    .tooltip-line {
      position: fixed; width: 1px; background: rgba(255,255,255,0.12);
      pointer-events: none; z-index: 9999;
    }
    .tooltip-dot {
      position: fixed; width: 10px; height: 10px; border-radius: 50%; border: 2px solid #fff;
      pointer-events: none; z-index: 9999;
    }
  `}updated(t){(t.has("data")||t.has("color")||t.has("overlays")||t.has("pointWidth"))&&this._draw()}_draw(){let t=this.canvas,e=this.wrapperEl;if(!t||!e||this.data.length<2)return;let n=t.getContext("2d"),s=devicePixelRatio||1,r=e.getBoundingClientRect(),a=r.width,o=r.height;if(a<=0||o<=0)return;let l=this.pointWidth>0?Math.max(this.pointWidth*this.data.length+80,a):a,c=o;e.style.width=l+"px",t.width=l*s,t.height=c*s,n.setTransform(s,0,0,s,0,0);let d=60,u=20,h=20,p=44,_=l-d-u,y=c-h-p;if(_<=0||y<=0)return;let m=this.data.map(L=>L.value),[f,S]=Pi(m),M=f-2,E=S+2;E-M<4&&(M--,E++),this._area={pL:d,pR:u,pT:h,pB:p,cW:_,cH:y,yMin:M,yMax:E,dLen:this.data.length};let R=(L,B)=>d+L/(B-1)*_,w=L=>h+y-(L-M)/(E-M)*y;n.clearRect(0,0,l,c),n.strokeStyle="#1a1a1e",n.lineWidth=1;for(let L=0;L<=5;L++){let B=h+y/5*L;n.beginPath(),n.moveTo(d,B),n.lineTo(l-u,B),n.stroke()}n.fillStyle="#52525b",n.font="11px JetBrains Mono,monospace",n.textAlign="right";for(let L=0;L<=5;L++){let B=M+(E-M)/5*(5-L);n.fillText(B.toFixed(1),d-8,h+y/5*L+4)}n.textAlign="center";let I=Math.max(60,_/12),v=_/(this.data.length-1),A=Math.max(1,Math.ceil(I/v));for(let L=0;L<this.data.length;L+=A)n.fillText(Ds(this.data[L].ts),R(L,this.data.length),c-p+16);n.fillText(Ds(this.data[this.data.length-1].ts),R(this.data.length-1,this.data.length),c-p+16),n.setLineDash([6,4]),this.warn>=M&&this.warn<=E&&(n.strokeStyle="rgba(234,179,8,0.4)",n.lineWidth=1,n.beginPath(),n.moveTo(d,w(this.warn)),n.lineTo(l-u,w(this.warn)),n.stroke()),this.crit>=M&&this.crit<=E&&(n.strokeStyle="rgba(239,68,68,0.4)",n.lineWidth=1,n.beginPath(),n.moveTo(d,w(this.crit)),n.lineTo(l-u,w(this.crit)),n.stroke()),n.setLineDash([]);let D=Tn(this.color),T=this.overlays.map(L=>({...L,data:L.data.filter(B=>B.value!=null&&!Number.isNaN(B.value)),color:Tn(L.color)})).filter(L=>L.data.length>=2);for(let L of T){let B=L.data.map(vt=>vt.value),[K,tt]=Pi(B),ct=Math.max(tt-K,.001),bt=vt=>h+y-(vt-K)/ct*y;n.beginPath();for(let vt=0;vt<L.data.length;vt++){let Gt=R(vt,L.data.length),Qt=bt(L.data[vt].value);vt===0?n.moveTo(Gt,Qt):n.lineTo(Gt,Qt)}n.strokeStyle=L.color,n.lineWidth=1.5,n.lineJoin="round",n.lineCap="round",n.stroke()}n.beginPath();for(let L=0;L<this.data.length;L++){let B=R(L,this.data.length),K=w(this.data[L].value);L===0?n.moveTo(B,K):n.lineTo(B,K)}n.strokeStyle=D,n.lineWidth=2,n.lineJoin="round",n.lineCap="round",n.stroke();let O=R(this.data.length-1,this.data.length);n.lineTo(O,h+y),n.lineTo(R(0,this.data.length),h+y),n.closePath();let H=n.createLinearGradient(0,h,0,h+y);H.addColorStop(0,D+"25"),H.addColorStop(1,D+"00"),n.fillStyle=H,n.fill();let $=this.data[this.data.length-1].value,U=w($);n.beginPath(),n.arc(O,U,5,0,Math.PI*2),n.fillStyle=D,n.fill(),n.beginPath(),n.arc(O,U,9,0,Math.PI*2),n.strokeStyle=D+"40",n.lineWidth=2,n.stroke(),n.fillStyle=D,n.font="bold 13px JetBrains Mono,monospace",n.textAlign="right",n.fillText($.toFixed(1)+(this.unit?" "+this.unit:""),l-u-2,U-12)}_onMove(t){let e=this.canvas;if(!e||this.data.length<2){this._hover=null;return}let n=e.getBoundingClientRect(),s=t.clientX-n.left,{pL:r,cW:a,dLen:o}=this._area,l=Math.round((s-r)/a*(o-1));if(l<0||l>=o){this._hover=null;return}this._hover={idx:l,clientX:t.clientX,clientY:t.clientY},this.requestUpdate()}_onLeave(){this._hover=null,this.requestUpdate()}render(){let t=this._hover,e=this.data,n=this.overlays.filter(o=>o.data.length>=2),s=Z``,r=Z``,a=Z``;if(t&&e[t.idx]){let o=[];o.push({name:"Primary",val:e[t.idx].value.toFixed(2)+(this.unit?" "+this.unit:""),color:Tn(this.color)});for(let u of n){let h=Math.min(Math.round(t.idx/(e.length-1)*(u.data.length-1)),u.data.length-1);u.data[h]&&o.push({name:u.name,val:u.data[h].value.toFixed(2)+(u.unit?" "+u.unit:""),color:Tn(u.color)})}let l=190,c=t.clientX+16+l>window.innerWidth?t.clientX-l-12:t.clientX+16,d=Math.max(4,t.clientY-60);s=Z`
        <div class="tooltip" style="left:${c}px;top:${d}px">
          ${o.map((u,h)=>Z`
            <div style="color:${u.color};font-weight:${h===0?700:500};font-size:${h===0?14:11}px;margin-top:${h>0?2:0}px">
              ${o.length>1?Z`<span style="font-size:10px;opacity:0.7">${u.name}: </span>`:""}${u.val}
            </div>
          `)}
          <div style="color:var(--muted);font-size:11px;margin-top:2px">${new Date(e[t.idx].ts).toLocaleString("en-US",{hour12:!1,month:"short",day:"numeric",hour:"2-digit",minute:"2-digit",second:"2-digit"})}</div>
        </div>
      `,r=Z`<div class="tooltip-line" style="left:${t.clientX}px;top:0;height:100%"></div>`,a=Z`<div class="tooltip-dot" style="left:${t.clientX-5}px;top:${t.clientY-5}px;background:${Tn(this.color)}"></div>`}return Z`
      <div class="scroll">
        <div class="wrapper">
          <canvas @mousemove="${this._onMove}" @mouseleave="${this._onLeave}"></canvas>
          ${n.length>0?Z`
            <div class="legend">
              ${n.map(o=>Z`
                <div class="legend-item" style="color:${Tn(o.color)}">
                  <div style="width:12px;height:2px;background:${Tn(o.color)};border-radius:1px"></div>
                  ${o.name} (${o.unit})
                </div>
              `)}
            </div>
          `:""}
        </div>
      </div>
      ${r}
      ${a}
      ${s}
    `}};dn([le({type:Array})],nn.prototype,"data",void 0);dn([le({type:String})],nn.prototype,"color",void 0);dn([le({type:Number})],nn.prototype,"warn",void 0);dn([le({type:Number})],nn.prototype,"crit",void 0);dn([le({type:String})],nn.prototype,"unit",void 0);dn([le({type:Array})],nn.prototype,"overlays",void 0);dn([le({type:Number})],nn.prototype,"pointWidth",void 0);dn([Ji("canvas")],nn.prototype,"canvas",void 0);dn([Ji(".scroll")],nn.prototype,"scrollEl",void 0);dn([Ji(".wrapper")],nn.prototype,"wrapperEl",void 0);nn=dn([Zt("vlt-detail-chart")],nn);var un=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},_u=["30m","1h","6h","24h"],rn=class extends kt{constructor(){super(...arguments),this.allSensors=[],this._range="30m",this._historyData=[],this._loading=!1,this._overlayIds=[],this._overlayData={},this._liveHistory=[],this._onKey=t=>{t.key==="Escape"&&this._back()},this._onLiveData=t=>{let e=t.detail;if(!e?.length)return;let n=[...this._liveHistory];for(let s of e)if(s.id===this.sensor.id&&(this._liveValue=s,s.value!=null&&!Number.isNaN(s.value)&&n.push({value:s.value,ts:Date.now()})),this._overlayIds.includes(s.id)&&s.value!=null&&!Number.isNaN(s.value)){let r=[...this._overlayData[s.id]||[]];r.push({value:s.value,ts:Date.now()}),this._overlayData={...this._overlayData,[s.id]:r.length>180?r.slice(-180):r}}this._liveHistory=n.length>180?n.slice(-180):n}}static{this.styles=qt`
    :host {
      position: fixed; inset: 0; background: var(--bg); z-index: 50;
      display: flex; flex-direction: column; padding: 16px;
    }
    .container {
      max-width: 1600px; width: 100%; margin: 0 auto;
      display: flex; flex-direction: column; height: 100%; gap: 12px;
    }
    .header {
      display: flex; align-items: center; justify-content: space-between; flex-shrink: 0;
    }
    .back-btn {
      display: flex; align-items: center; gap: 6px;
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    h1 { font-size: 20px; font-weight: 700; color: var(--fg); margin: 0; }
    .status-dot { width: 10px; height: 10px; border-radius: 50%; }
    .stats {
      display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; flex-shrink: 0;
    }
    .stat-card {
      background: var(--card); border: 1px solid var(--border); border-radius: 8px;
      padding: 12px 16px;
    }
    .stat-label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
    .stat-value { font-family: 'JetBrains Mono', monospace; font-size: 18px; font-weight: 700; color: var(--fg); }
    .toolbar {
      display: flex; align-items: center; gap: 8px; flex-shrink: 0;
    }
    .toolbar button {
      padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 12px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .chart-wrap { flex: 1; min-height: 0; background: var(--card); border: 1px solid var(--border); border-radius: 10px; overflow: hidden; }
  `}connectedCallback(){super.connectedCallback(),this._liveValue=this.currentValue,this._loadHistory(),window.addEventListener("keydown",this._onKey),en.addEventListener("data",this._onLiveData),en.connect()}disconnectedCallback(){super.disconnectedCallback(),window.removeEventListener("keydown",this._onKey),en.removeEventListener("data",this._onLiveData)}async _loadHistory(){this._loading=!0;try{let t=await Kt(`/history?range=${this._range}`);if(!t.data){this._historyData=[],this._overlayData={};return}let e=this.sensor.key,n=t.data[e];if(n&&Array.isArray(n.values)){let s=[];for(let r=0;r<n.values.length;r++)s.push({ts:n.start+r*n.interval,value:n.values[r]});this._historyData=s}else Array.isArray(n)?this._historyData=n:this._historyData=[];await this._loadOverlayHistories(t.data)}catch{this._historyData=[],this._overlayData={}}this._loading=!1}async _loadOverlayHistories(t){if(this._overlayIds.length===0){this._overlayData={};return}let e={};for(let n of this._overlayIds){let s=this.allSensors.find(r=>r.id===n);if(s)if(t&&t[s.key]){let r=t[s.key];if(r&&Array.isArray(r.values)){let a=[];for(let o=0;o<r.values.length;o++)a.push({ts:r.start+o*r.interval,value:r.values[o]});e[n]=a}else Array.isArray(r)&&(e[n]=r)}else try{let a=(await Kt(`/history?range=${this._range}`)).data?.[s.key];if(a&&Array.isArray(a.values)){let o=[];for(let l=0;l<a.values.length;l++)o.push({ts:a.start+l*a.interval,value:a.values[l]});e[n]=o}else Array.isArray(a)&&(e[n]=a)}catch{}}this._overlayData=e}_back(){this.dispatchEvent(new CustomEvent("back"))}_setRange(t){this._range=t,this._loadHistory()}async _toggleOverlay(t){let e=this._overlayIds.includes(t)?this._overlayIds.filter(n=>n!==t):[...this._overlayIds,t];this._overlayIds=e,await this._loadOverlayHistories()}_removeOverlay(t){this._overlayIds=this._overlayIds.filter(n=>n!==t);let e={...this._overlayData};delete e[t],this._overlayData=e}_overlays(){return this._overlayIds.map(t=>{let e=this.allSensors.find(s=>s.id===t);return e?{data:this._overlayData[t]||[],color:e.color,name:e.name,unit:e.unit}:null}).filter(t=>t!=null)}_chartData(){return this._liveHistory.length>0?[...this._historyData,...this._liveHistory]:this._historyData}render(){let t=this._liveValue&&(this._liveValue.value==null||Number.isNaN(this._liveValue.value)),e=t?"offline":this._liveValue?Pr(this._liveValue.value,this._liveValue.warn,this._liveValue.crit):"normal",n=this._chartData(),s=n.map(d=>d.value).filter(d=>d!=null&&!Number.isNaN(d)),[r,a]=Pi(s),o=s.length?s.reduce((d,u)=>d+u,0)/s.length:0,l=t?null:this._liveValue?this._liveValue.value:this.sensor.base_val,c=this.allSensors.filter(d=>d.id!==this.sensor.id&&!this._overlayIds.includes(d.id));return Z`
      <div class="container">
        <div class="header">
          <div style="display:flex;align-items:center;gap:12px">
            <button class="back-btn" @click="${this._back}">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
              Back
            </button>
            <h1>${this.sensor.name}</h1>
            <div class="status-dot" style="background:${Ii(e)}"></div>
          </div>
          <div style="font-family:'JetBrains Mono',monospace;font-size:28px;font-weight:700;color:${e==="normal"?this.sensor.color:Ii(e)}">
            ${l!=null?l.toFixed(this.sensor.decimals):"OFFLINE"}
            <span style="font-size:14px;color:var(--muted);font-weight:400">${this.sensor.unit}</span>
          </div>
        </div>

        <div class="stats">
          <div class="stat-card">
            <div class="stat-label">Minimum</div>
            <div class="stat-value" style="color:var(--cyan)">${r.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Maximum</div>
            <div class="stat-value" style="color:var(--warn)">${a.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Average</div>
            <div class="stat-value">${o.toFixed(this.sensor.decimals)}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Points</div>
            <div class="stat-value">${s.length}</div>
          </div>
        </div>

        <div class="toolbar">
          ${_u.map(d=>Z`
            <button class="${this._range===d?"active":""}" @click="${()=>this._setRange(d)}">${d}</button>
          `)}
          ${c.length>0?Z`
            <select @change="${d=>this._toggleOverlay(Number(d.target.value))}" style="background:var(--bg);border:1px solid var(--border);color:var(--fg);border-radius:6px;padding:6px 10px;font-family:inherit;font-size:12px">
              <option value="">+ Overlay sensor...</option>
              ${c.map(d=>Z`<option value="${d.id}">${d.name}</option>`)}
            </select>
          `:""}
          ${this._overlayIds.length>0?Z`
            <div style="display:flex;gap:4px;flex-wrap:wrap">
              ${this._overlayIds.map(d=>{let u=this.allSensors.find(h=>h.id===d);return u?Z`<span style="font-size:11px;padding:2px 8px;border-radius:4px;background:${u.color}18;color:${u.color};display:flex;align-items:center;gap:4px;cursor:pointer" @click="${()=>this._removeOverlay(d)}" title="Remove">${u.name} <span style="opacity:.6">×</span></span>`:""})}
            </div>
          `:""}
        </div>

        <div class="chart-wrap">
          ${this._loading&&this._historyData.length===0?Z`<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--muted)">Loading history...</div>`:Z`<vlt-detail-chart .data="${n}" .color="${this.sensor.color}" .warn="${this.sensor.warn_val}" .crit="${this.sensor.crit_val}" .unit="${this.sensor.unit}" .overlays="${this._overlays()}"></vlt-detail-chart>`}
        </div>
      </div>
    `}};un([le({type:Object})],rn.prototype,"sensor",void 0);un([le({type:Object})],rn.prototype,"currentValue",void 0);un([le({type:Array})],rn.prototype,"allSensors",void 0);un([et()],rn.prototype,"_range",void 0);un([et()],rn.prototype,"_historyData",void 0);un([et()],rn.prototype,"_loading",void 0);un([et()],rn.prototype,"_overlayIds",void 0);un([et()],rn.prototype,"_overlayData",void 0);un([et()],rn.prototype,"_liveValue",void 0);un([et()],rn.prototype,"_liveHistory",void 0);rn=un([Zt("vlt-sensor-detail")],rn);var fn=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},an=class extends kt{constructor(){super(...arguments),this._tab="general",this._gap=12,this._refresh=1e3,this._agg="sampling",this._muted=!1,this._branding={title:"Voltrus Lite",subtitle:"Industrial monitoring",logo:""},this._sensors=[],this._shortcuts={},this._navShortcuts={},this._saving=!1,this._navActions=[{key:"grid",label:"Grid View"},{key:"canvas",label:"Canvas View"},{key:"scada",label:"2D SCADA"},{key:"twin",label:"3D Twin"},{key:"system",label:"System"},{key:"alarms",label:"Alarms"},{key:"settings",label:"Settings"},{key:"data",label:"Data Manager"}]}static{this.styles=qt`
    :host {
      position: fixed; inset: 0; background: rgba(0,0,0,.6);
      display: flex; align-items: center; justify-content: center; z-index: 100;
    }
    .modal {
      background: var(--card); border: 1px solid var(--border); border-radius: 12px;
      padding: 0; min-width: 480px; max-width: 600px; max-height: 80vh; display: flex; flex-direction: column;
    }
    .header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid var(--border); }
    .title { font-size: 15px; font-weight: 600; color: var(--fg); }
    .tabs { display: flex; gap: 0; border-bottom: 1px solid var(--border); }
    .tabs button {
      flex: 1; padding: 10px; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
      background: transparent; border: none; border-bottom: 2px solid transparent; color: var(--muted); cursor: pointer; font-family: inherit;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .body { padding: 20px; overflow-y: auto; flex: 1; }
    .row { display: flex; flex-direction: column; gap: 8px; margin-bottom: 14px; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    input[type="text"], input[type="url"] {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px;
      color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box;
    }
    input[type="range"] { width: 100%; }
    .value { font-size: 11px; font-family: 'JetBrains Mono', monospace; }
    .agg-btns { display: flex; gap: 4px; margin-top: 4px; }
    .agg-btns button {
      flex: 1; padding: 8px 6px; font-size: 11px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit;
    }
    .agg-btns button.active { border-color: var(--warn); color: var(--warn); background: rgba(234,179,8,.08); font-weight: 600; }
    .desc { font-size: 11px; color: var(--muted); margin-top: 6px; line-height: 1.4; }
    .mute-btn {
      padding: 6px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
      display: flex; align-items: center; gap: 6px; width: fit-content;
    }
    .mute-btn.muted { border-color: var(--warn); background: rgba(234,179,8,.1); color: var(--warn); }
    .close-btn {
      margin: 0 20px 20px; padding: 8px 16px; border-radius: 6px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .shortcut-row {
      display: grid; grid-template-columns: 1fr 80px; gap: 8px; align-items: center; margin-bottom: 8px;
    }
    .shortcut-row input {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 5px 8px;
      color: var(--fg); font-size: 13px; font-family: inherit; outline: none; text-align: center; text-transform: uppercase;
    }
    .shortcut-row span { font-size: 13px; color: var(--fg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .save-btn {
      margin-top: 12px; padding: 8px 16px; border-radius: 6px; border: 1px solid var(--cyan);
      background: transparent; color: var(--cyan); cursor: pointer; font-family: inherit; font-size: 13px; font-weight: 600;
    }
    .section-title { font-size: 12px; font-weight: 600; color: var(--muted); margin: 16px 0 8px; text-transform: uppercase; letter-spacing: 0.04em; }
  `}connectedCallback(){super.connectedCallback(),this._muted=Ri.isMuted,this._load()}async _load(){try{this._branding=await Kt("/settings/branding"),this._sensors=await Kt("/data-sources"),this._shortcuts=await Kt("/shortcuts"),this._navShortcuts=await Kt("/nav-shortcuts")}catch{}}_setAgg(t){this._agg=t}_toggleMute(){this._muted=Ri.toggleMute()}async _saveBranding(){try{await Ie("/api/settings/branding",{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(this._branding)}),En.setBranding(this._branding)}catch{}}async _saveShortcuts(){this._saving=!0;try{await Ie("/api/shortcuts",{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(this._shortcuts)}),await Ie("/api/nav-shortcuts",{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(this._navShortcuts)}),En.setShortcuts(this._shortcuts),En.setNavShortcuts(this._navShortcuts)}catch{}this._saving=!1}_updateShortcut(t,e){let n=e.trim().toLowerCase();if(n)this._shortcuts={...this._shortcuts,[t]:n};else{let s={...this._shortcuts};delete s[t],this._shortcuts=s}}_updateNavShortcut(t,e){let n=e.trim().toLowerCase();if(n)this._navShortcuts={...this._navShortcuts,[t]:n};else{let s={...this._navShortcuts};delete s[t],this._navShortcuts=s}}_close(){this.dispatchEvent(new CustomEvent("close"))}render(){let t=[{value:"sampling",label:"Sampling",desc:"Pick evenly-spaced points. Fastest, but skips data between intervals."},{value:"avg",label:"Average",desc:"Average all values within each interval bucket. Smooth curve, hides peaks."},{value:"minmax",label:"Min/Max/Avg",desc:"Shows average line with min/max band. Best overview \u2014 no data is lost."}];return Z`
      <div @click="${this._close}">
        <div class="modal" @click="${e=>e.stopPropagation()}">
          <div class="header">
            <div class="title">Dashboard Settings</div>
            <button style="background:transparent;border:none;color:var(--muted);cursor:pointer;font-size:18px;line-height:1" @click="${this._close}">×</button>
          </div>
          <div class="tabs">
            <button class="${this._tab==="general"?"active":""}" @click="${()=>this._tab="general"}">General</button>
            <button class="${this._tab==="branding"?"active":""}" @click="${()=>this._tab="branding"}">Branding</button>
            <button class="${this._tab==="shortcuts"?"active":""}" @click="${()=>this._tab="shortcuts"}">Shortcuts</button>
          </div>
          <div class="body">
            ${this._tab==="general"?Z`
              <div class="row">
                <span class="label">Grid Gap (px)</span>
                <input type="range" min="4" max="24" .value="${this._gap}" @input="${e=>this._gap=parseInt(e.target.value)}">
                <span class="value">${this._gap}px</span>
              </div>
              <div class="row">
                <span class="label">History Auto Mode</span>
                <div class="agg-btns">
                  ${t.map(e=>Z`
                    <button class="${this._agg===e.value?"active":""}" @click="${()=>this._setAgg(e.value)}">${e.label}</button>
                  `)}
                </div>
                <div class="desc">${t.find(e=>e.value===this._agg)?.desc}</div>
              </div>
              <div class="row">
                <span class="label">Mute Alarms</span>
                <button class="mute-btn ${this._muted?"muted":""}" @click="${this._toggleMute}">
                  ${this._muted?"Muted":"Active"}
                </button>
              </div>
            `:""}
            ${this._tab==="branding"?Z`
              <div class="row">
                <span class="label">Title</span>
                <input type="text" .value="${this._branding.title}" @input="${e=>this._branding={...this._branding,title:e.target.value}}" />
              </div>
              <div class="row">
                <span class="label">Subtitle</span>
                <input type="text" .value="${this._branding.subtitle}" @input="${e=>this._branding={...this._branding,subtitle:e.target.value}}" />
              </div>
              <div class="row">
                <span class="label">Logo URL</span>
                <input type="url" .value="${this._branding.logo}" @input="${e=>this._branding={...this._branding,logo:e.target.value}}" placeholder="https://..." />
              </div>
              <button class="save-btn" @click="${this._saveBranding}">Save Branding</button>
            `:""}
            ${this._tab==="shortcuts"?Z`
              <div class="section-title">Navigation Shortcuts</div>
              ${this._navActions.map(e=>Z`
                <div class="shortcut-row">
                  <span title="${e.label}">${e.label}</span>
                  <input type="text" maxlength="1" .value="${this._navShortcuts[e.key]||""}" @input="${n=>this._updateNavShortcut(e.key,n.target.value)}" />
                </div>
              `)}
              <div class="section-title">Sensor Shortcuts</div>
              ${this._sensors.slice(0,20).map(e=>Z`
                <div class="shortcut-row">
                  <span title="${e.name}">${e.name}</span>
                  <input type="text" maxlength="1" .value="${this._shortcuts[e.id]||""}" @input="${n=>this._updateShortcut(e.id,n.target.value)}" />
                </div>
              `)}
              ${this._sensors.length>20?Z`<div style="font-size:11px;color:var(--muted)">Showing first 20 sensors.</div>`:""}
              <button class="save-btn" @click="${this._saveShortcuts}" .disabled="${this._saving}">${this._saving?"Saving...":"Save Shortcuts"}</button>
            `:""}
          </div>
          <button class="close-btn" @click="${this._close}">Close</button>
        </div>
      </div>
    `}};fn([et()],an.prototype,"_tab",void 0);fn([et()],an.prototype,"_gap",void 0);fn([et()],an.prototype,"_refresh",void 0);fn([et()],an.prototype,"_agg",void 0);fn([et()],an.prototype,"_muted",void 0);fn([et()],an.prototype,"_branding",void 0);fn([et()],an.prototype,"_sensors",void 0);fn([et()],an.prototype,"_shortcuts",void 0);fn([et()],an.prototype,"_navShortcuts",void 0);fn([et()],an.prototype,"_saving",void 0);an=fn([Zt("vlt-dashboard-settings")],an);var rl=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},sl=class extends kt{constructor(){super(...arguments),this._value="",this.title="Prompt",this.label="Value",this.placeholder="",this.initialValue="",this.okLabel="OK"}static{this.styles=qt`
    :host { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 60; }
    .modal { background: var(--card); border: 1px solid var(--border); border-radius: 10px; width: 380px; padding: 20px; }
    .title { font-size: 15px; font-weight: 600; margin-bottom: 16px; color: var(--fg); }
    label { display: flex; flex-direction: column; gap: 4px; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    input { background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 8px; color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box; }
    .footer { display: flex; gap: 8; justify-content: flex-end; margin-top: 16px; }
    .footer button { padding: 7px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit; }
    .ok { border: 1px solid var(--cyan); background: transparent; color: var(--cyan); font-weight: 600; }
    .cancel { border: 1px solid var(--border); background: transparent; color: var(--fg); }
  `}connectedCallback(){super.connectedCallback(),this._value=this.initialValue||""}_submit(){let t=this._value.trim();t&&this.dispatchEvent(new CustomEvent("submit",{detail:t}))}_close(){this.dispatchEvent(new CustomEvent("close"))}render(){return Z`
      <div @click="${this._close}">
        <div class="modal" @click="${t=>t.stopPropagation()}">
          <div class="title">${this.title}</div>
          <label>
            <span class="label">${this.label}</span>
            <input .value="${this._value}" @input="${t=>this._value=t.target.value}" placeholder="${this.placeholder}" autofocus @keydown="${t=>t.key==="Enter"&&this._submit()}" />
          </label>
          <div class="footer">
            <button class="cancel" @click="${this._close}">Cancel</button>
            <button class="ok" @click="${this._submit}">${this.okLabel}</button>
          </div>
        </div>
      </div>
    `}};rl([et()],sl.prototype,"_value",void 0);sl=rl([Zt("vlt-prompt-dialog")],sl);var rh=class extends kt{constructor(){super(...arguments),this.title="Confirm",this.message="Are you sure?",this.confirmLabel="Delete"}static{this.styles=qt`
    :host { position: fixed; inset: 0; background: rgba(0,0,0,.6); display: flex; align-items: center; justify-content: center; z-index: 60; }
    .modal { background: var(--card); border: 1px solid var(--border); border-radius: 10px; width: 380px; padding: 20px; }
    .title { font-size: 15px; font-weight: 600; margin-bottom: 12px; color: var(--fg); }
    .msg { font-size: 13px; color: var(--muted); margin-bottom: 20px; }
    .footer { display: flex; gap: 8; justify-content: flex-end; }
    .footer button { padding: 7px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit; }
    .confirm { border: 1px solid var(--crit); background: var(--crit); color: #fff; font-weight: 600; }
    .cancel { border: 1px solid var(--border); background: transparent; color: var(--fg); }
  `}_confirm(){this.dispatchEvent(new CustomEvent("confirm"))}_close(){this.dispatchEvent(new CustomEvent("close"))}render(){return Z`
      <div @click="${this._close}">
        <div class="modal" @click="${t=>t.stopPropagation()}">
          <div class="title">${this.title}</div>
          <div class="msg">${this.message}</div>
          <div class="footer">
            <button class="cancel" @click="${this._close}">Cancel</button>
            <button class="confirm" @click="${this._confirm}">${this.confirmLabel}</button>
          </div>
        </div>
      </div>
    `}};rh=rl([Zt("vlt-confirm-dialog")],rh);var Ki=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Li=class extends kt{constructor(){super(...arguments),this.data=[],this.color="#22c55e",this.warn=0,this.crit=0}static{this.styles=qt`
    :host { display: block; width: 100%; height: 100%; min-height: 0; }
    canvas { display: block; width: 100%; height: 100%; min-height: 0; }
  `}updated(t){(t.has("data")||t.has("color")||t.has("warn")||t.has("crit"))&&this._draw()}_draw(){let t=this.canvas;if(!t||this.data.length<2)return;let e=t.getContext("2d"),n=devicePixelRatio||1,s=t.getBoundingClientRect();t.width=s.width*n,t.height=s.height*n,e.scale(n,n);let r=s.width,a=s.height,o=40,l=10,c=10,d=24,u=r-o-l,h=a-c-d;e.clearRect(0,0,r,a),e.strokeStyle="#1a1a1e",e.lineWidth=1;for(let A=0;A<=4;A++){let D=c+h/4*A;e.beginPath(),e.moveTo(o,D),e.lineTo(r-l,D),e.stroke()}let[p,_]=Pi(this.data),y=p-2,m=_+2;m-y<4&&(y--,m++);let f=A=>o+A/(this.data.length-1)*u,S=A=>c+h-(A-y)/(m-y)*h;e.fillStyle="#52525b",e.font="10px JetBrains Mono,monospace",e.textAlign="right";for(let A=0;A<=4;A++){let D=y+(m-y)/4*(4-A);e.fillText(D.toFixed(1),o-6,c+h/4*A+3)}e.setLineDash([4,4]),this.warn>=y&&this.warn<=m&&(e.strokeStyle="rgba(234,179,8,0.3)",e.beginPath(),e.moveTo(o,S(this.warn)),e.lineTo(r-l,S(this.warn)),e.stroke()),this.crit>=y&&this.crit<=m&&(e.strokeStyle="rgba(239,68,68,0.3)",e.beginPath(),e.moveTo(o,S(this.crit)),e.lineTo(r-l,S(this.crit)),e.stroke()),e.setLineDash([]);let M=Tn(this.color.replace(/"/g,""));e.beginPath();for(let A=0;A<this.data.length;A++){let D=f(A),T=S(this.data[A]);A===0?e.moveTo(D,T):e.lineTo(D,T)}e.strokeStyle=M,e.lineWidth=2,e.lineJoin="round",e.lineCap="round",e.stroke();let E=f(this.data.length-1),R=f(0);e.lineTo(E,c+h),e.lineTo(R,c+h),e.closePath();let w=e.createLinearGradient(0,c,0,c+h);w.addColorStop(0,M+"30"),w.addColorStop(1,M+"00"),e.fillStyle=w,e.fill();let I=this.data[this.data.length-1],v=S(I);e.beginPath(),e.arc(E,v,4,0,Math.PI*2),e.fillStyle=M,e.fill(),e.beginPath(),e.arc(E,v,7,0,Math.PI*2),e.strokeStyle=M+"40",e.lineWidth=2,e.stroke(),e.fillStyle=M,e.font="bold 11px JetBrains Mono,monospace",e.textAlign="right",e.fillText(I.toFixed(1),r-l-2,v-8)}render(){return Z`<canvas></canvas>`}};Ki([le({type:Array})],Li.prototype,"data",void 0);Ki([le({type:String})],Li.prototype,"color",void 0);Ki([le({type:Number})],Li.prototype,"warn",void 0);Ki([le({type:Number})],Li.prototype,"crit",void 0);Ki([Ji("canvas")],Li.prototype,"canvas",void 0);Li=Ki([Zt("vlt-chart-sparkline")],Li);var al=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Dr=class extends kt{constructor(){super(...arguments),this.selected=!1}static{this.styles=qt`
    :host { display: block; height: 100%; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      overflow: hidden; height: 100%; display: flex; flex-direction: column;
      box-sizing: border-box;
    }
    .card.selected { border-color: var(--cyan); box-shadow: 0 0 0 2px var(--cyan); }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 14px 16px 10px;
    }
    .title-row {
      display: flex; align-items: center; gap: 8px;
      font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;
    }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; }
    .source-tag {
      font-size: 9px; padding: 1px 5px; border-radius: 3px; font-weight: 700;
      letter-spacing: 0.03em; text-transform: none;
    }
    .right {
      display: flex; align-items: center; gap: 6px;
    }
    .shortcut {
      font-family: 'JetBrains Mono', monospace; font-size: 10px; font-weight: 600;
      padding: 2px 6px; border-radius: 3px; background: var(--bg); border: 1px solid var(--border);
      color: var(--muted); line-height: 1;
    }
    .status-badge {
      font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em;
      padding: 2px 8px; border-radius: 4px;
    }
    .value {
      font-family: 'JetBrains Mono', monospace; font-size: 22px; font-weight: 700;
      padding: 0 16px;
    }
    .unit { font-size: 13px; color: var(--muted); font-weight: 400; margin-left: 4px; }
    .chart-wrap { flex: 1; min-height: 0; height: 140px; }
  `}_click(){this.dispatchEvent(new CustomEvent("sensor-click",{detail:this.sensor.config.id,bubbles:!0,composed:!0}))}render(){let t=this.sensor.config,e=this.sensor.data,n=e&&(e.value==null||Number.isNaN(e.value)),s=n?"offline":e?Pr(e.value,e.warn,e.crit):"normal",r=t.chart_type||"sparkline",a=r==="image"||r==="text",o=si[t.source_type]?.tag??t.source_type,l=si[t.source_type]?.color??"var(--muted)",c=n?"OFFLINE":e?e.value.toFixed(t.decimals):t.base_val.toFixed(t.decimals),d=s==="normal"?t.color:Ii(s);return Z`
      <div class="card ${this.selected?"selected":""}" @click="${this._click}" style="cursor:pointer">
        ${a?"":Z`
          <div class="header">
            <div class="title-row">
              <div class="status-dot" style="background:${Ii(s)}"></div>
              ${t.name}
              <span class="source-tag" style="background:${l}18;color:${l}">${o}</span>
            </div>
            <div class="right">
              <div class="status-badge" style="background:${s==="critical"?"rgba(239,68,68,.15)":s==="warning"?"rgba(234,179,8,.15)":s==="offline"?"rgba(100,100,100,.15)":"rgba(34,197,94,.15)"};color:${Ii(s)}">${s}</div>
            </div>
          </div>
          <div class="value" style="color:${d}">${c}<span class="unit">${t.unit}</span></div>
        `}
        <div class="chart-wrap">
          ${r==="sparkline"?Z`<vlt-chart-sparkline .data="${this.sensor.chartData}" .color="${t.color}" .warn="${t.warn_val}" .crit="${t.crit_val}"></vlt-chart-sparkline>`:Z`<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--muted);font-size:12px">${r} chart</div>`}
        </div>
      </div>
    `}};al([le({type:Object})],Dr.prototype,"sensor",void 0);al([le({type:Boolean})],Dr.prototype,"selected",void 0);Dr=al([Zt("vlt-sensor-card")],Dr);var on=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},ah={lg:1200,md:768,sm:0},vu={lg:24,md:12,sm:4};var Xe=class extends kt{constructor(){super(...arguments),this._sensors=[],this._dataMap={},this._currentValues={},this._screenId="all",this._loading=!0,this._layouts={lg:[],md:[],sm:[]},this._editMode=!1,this._draggingId=null,this._overId=null,this._bp="lg",this._containerWidth=1200,this._onData=t=>{let e=t.detail;if(!e?.length)return;let n={...this._dataMap},s={...this._currentValues};for(let r of e){let a=String(r.id);if(r.value==null||Number.isNaN(r.value)){s[a]=r;continue}let l=n[a]?[...n[a]]:[];l.push({value:r.value,ts:Date.now()}),n[a]=l.length>180?l.slice(-180):l,s[a]=r}this._dataMap=n,this._currentValues=s}}static{this.styles=qt`
    :host { display: block; padding: 0 20px 20px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    .grid {
      display: grid;
      gap: 12px;
      grid-auto-rows: 50px;
      align-items: stretch;
    }
    .grid.lg { grid-template-columns: repeat(24, 1fr); }
    .grid.md { grid-template-columns: repeat(12, 1fr); }
    .grid.sm { grid-template-columns: repeat(4, 1fr); }
    .card-wrap {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      overflow: hidden; display: flex; flex-direction: column;
      min-height: 0;
    }
    .card-wrap.edit { cursor: grab; }
    .card-wrap.edit.dragging { opacity: 0.5; cursor: grabbing; }
    .card-wrap.edit.over { outline: 2px dashed var(--cyan); outline-offset: 2px; }
    .drag-handle {
      height: 6px; background: var(--cyan); opacity: 0; cursor: grab; border-radius: 10px 10px 0 0; flex-shrink: 0;
    }
    .card-wrap.edit .drag-handle { opacity: 0.4; }
    .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--muted); }
  `}set screenId(t){this._screenId!==t&&(this._screenId=t,this._loadSensors(),this._loadLayouts())}get screenId(){return this._screenId}set editMode(t){this._editMode=t}get editMode(){return this._editMode}connectedCallback(){super.connectedCallback(),en.addEventListener("data",this._onData),en.connect(),this._loadSensors(),this._loadLayouts(),this._seedHistory(),this._resizeObs=new ResizeObserver(t=>{let e=t[0]?.contentRect.width??1200;this._containerWidth=e,this._bp=e>=ah.lg?"lg":e>=ah.md?"md":"sm"}),this._resizeObs.observe(this)}disconnectedCallback(){super.disconnectedCallback(),en.removeEventListener("data",this._onData),this._resizeObs?.disconnect()}async _loadSensors(){this._loading=!0;try{let t=this._screenId==="all"?"/data-sources":`/screens/${this._screenId}/data-sources`;this._sensors=await Kt(t)}catch{this._sensors=[]}this._loading=!1}async _loadLayouts(){try{let t=this._screenId==="all"?"__overview":this._screenId;this._layouts=await Kt(`/screens/${t}/layouts`)}catch{this._layouts={lg:[],md:[],sm:[]}}}async saveLayouts(){try{let t=this._screenId==="all"?"__overview":this._screenId;await Ie(`/api/screens/${t}/layouts`,{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(this._layouts)})}catch{}}async _seedHistory(){try{let t=this._screenId==="all"?"":`&screen=${this._screenId}`,e=await Kt(`/history?range=30m${t}`);if(!e.data)return;let n={};for(let r of Object.keys(e.data)){let a=e.data[r];if(a&&Array.isArray(a.values)){let o=[];for(let l=0;l<a.values.length;l++)o.push({ts:a.start+l*a.interval,value:a.values[l]});n[r]=o}else Array.isArray(a)&&(n[r]=a)}let s={};for(let r of this._sensors){let a=n[r.key];a&&(s[String(r.id)]=a.length>180?a.slice(-180):a)}this._dataMap=s}catch{}}_cardSensors(){return this._sensors.map(t=>({config:t,data:this._currentValues[String(t.id)],chartData:(this._dataMap[String(t.id)]||[]).map(e=>e.value).filter(e=>e!=null&&!Number.isNaN(e))}))}_layoutFor(t){return(this._layouts[this._bp]||[]).find(n=>n.i===String(t))}_gridStyleFor(t){if(!t)return"";let e=vu[this._bp],n=Math.min(t.x,e-1),s=Math.min(t.w,e-n);return`grid-column:${n+1} / span ${s};grid-row:${t.y+1} / span ${t.h};`}_onSensorClick(t){t.stopPropagation(),this.dispatchEvent(new CustomEvent("sensor-click",{detail:t.detail}))}_onDragStart(t,e){if(!this._editMode){e.preventDefault();return}this._draggingId=t,e.dataTransfer.effectAllowed="move",e.dataTransfer.setData("text/plain",String(t))}_onDragOver(t,e){!this._editMode||this._draggingId===null||this._draggingId===t||(e.preventDefault(),e.dataTransfer.dropEffect="move",this._overId=t)}_onDragLeave(){this._overId=null}_onDrop(t,e){if(!this._editMode||this._draggingId===null)return;e.preventDefault();let n=this._draggingId,s=t;if(this._draggingId=null,this._overId=null,n===s)return;let r=a=>{let o=a.find(c=>c.i===String(n)),l=a.find(c=>c.i===String(s));return!o||!l?a:a.map(c=>c.i===String(n)?{...c,x:l.x,y:l.y}:c.i===String(s)?{...c,x:o.x,y:o.y}:c)};this._layouts={lg:r(this._layouts.lg||[]),md:r(this._layouts.md||[]),sm:r(this._layouts.sm||[])}}render(){if(this._loading&&this._sensors.length===0)return Z`<div class="empty">Loading sensors...</div>`;let t=this._cardSensors();return t.length?Z`
      <div class="grid ${this._bp} ${this._editMode?"edit":""}">
        ${t.map(e=>{let n=this._layoutFor(e.config.id),s=this._gridStyleFor(n);return Z`
            <div
              class="card-wrap ${this._editMode?"edit":""} ${this._draggingId===e.config.id?"dragging":""} ${this._overId===e.config.id?"over":""}"
              style="${s}"
              draggable="${this._editMode}"
              @dragstart="${r=>this._onDragStart(e.config.id,r)}"
              @dragover="${r=>this._onDragOver(e.config.id,r)}"
              @dragleave="${this._onDragLeave}"
              @drop="${r=>this._onDrop(e.config.id,r)}"
            >
              ${this._editMode?Z`<div class="drag-handle"></div>`:""}
              <vlt-sensor-card .sensor="${e}" @sensor-click="${this._onSensorClick}"></vlt-sensor-card>
            </div>
          `})}
      </div>
    `:Z`<div class="empty">No sensors on this screen</div>`}};on([et()],Xe.prototype,"_sensors",void 0);on([et()],Xe.prototype,"_dataMap",void 0);on([et()],Xe.prototype,"_currentValues",void 0);on([et()],Xe.prototype,"_screenId",void 0);on([et()],Xe.prototype,"_loading",void 0);on([et()],Xe.prototype,"_layouts",void 0);on([et()],Xe.prototype,"_editMode",void 0);on([et()],Xe.prototype,"_draggingId",void 0);on([et()],Xe.prototype,"_overId",void 0);on([et()],Xe.prototype,"_bp",void 0);on([et()],Xe.prototype,"_containerWidth",void 0);Xe=on([Zt("vlt-grid-view")],Xe);var Cn=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},pn=class extends kt{constructor(){super(...arguments),this._sensors=[],this._layouts=[],this._dataMap={},this._currentValues={},this._selectedId=null,this._dragging=null,this._resizing=null,this._loading=!0,this.screenId="all",this._onData=t=>{let e=t.detail;if(!e?.length)return;let n={...this._dataMap},s={...this._currentValues};for(let r of e){let a=String(r.id),o=n[a]?[...n[a]]:[];o.push({value:r.value,ts:Date.now()}),n[a]=o.length>180?o.slice(-180):o,s[a]=r}this._dataMap=n,this._currentValues=s},this._onMouseMove=t=>{if(this._dragging){let e=this._dragging;this._updateLayout(e.id,{x:e.origX+t.clientX-e.startX,y:e.origY+t.clientY-e.startY})}if(this._resizing){let e=this._resizing;this._updateLayout(e.id,{width:Math.max(200,e.origW+t.clientX-e.startX),height:Math.max(150,e.origH+t.clientY-e.startY)})}},this._onMouseUp=()=>{this._dragging=null,this._resizing=null,window.removeEventListener("mousemove",this._onMouseMove),window.removeEventListener("mouseup",this._onMouseUp)}}static{this.styles=qt`
    :host { display: block; height: 100%; overflow: auto; position: relative; }
    .container { position: relative; min-height: 600px; padding: 20px; }
    .card-wrap {
      position: absolute; background: var(--card); border: 1px solid var(--border);
      border-radius: 10px; overflow: hidden; display: flex; flex-direction: column;
      user-select: none;
    }
    .card-wrap.selected { border-color: var(--cyan); box-shadow: 0 0 0 2px var(--cyan); }
    .card-wrap .drag-bar {
      height: 6px; background: var(--cyan); opacity: 0; cursor: grab; border-radius: 10px 10px 0 0;
    }
    .card-wrap:hover .drag-bar, .card-wrap.selected .drag-bar { opacity: 0.6; }
    .resize-handle {
      position: absolute; width: 12px; height: 12px; bottom: 0; right: 0; cursor: nwse-resize;
      background: var(--cyan); border-radius: 6px 0 6px 0; opacity: 0;
    }
    .card-wrap:hover .resize-handle, .card-wrap.selected .resize-handle { opacity: 0.7; }
  `}connectedCallback(){super.connectedCallback(),en.addEventListener("data",this._onData),en.connect(),this._load()}disconnectedCallback(){super.disconnectedCallback(),en.removeEventListener("data",this._onData)}updated(t){t.has("screenId")&&this._load()}async _load(){this._loading=!0;try{let t=this.screenId==="all"?"/data-sources":`/screens/${this.screenId}/data-sources`;this._sensors=await Kt(t);let e=this.screenId==="all"?"__overview":this.screenId;this._layouts=await Kt(`/screens/${e}/canvas-layouts`)}catch{}this._loading=!1}_layoutFor(t){let e=this._layouts.find(r=>r.data_source_id===t),n=this._sensors.findIndex(r=>r.id===t),s=Math.max(1,Math.floor(1200/320));return e||{data_source_id:t,x:n%s*320,y:Math.floor(n/s)*220,width:300,height:200}}_onMouseDown(t,e){e.target?.classList?.contains("resize-handle")||(this._selectedId=t,this._dragging={id:t,startX:e.clientX,startY:e.clientY,origX:this._layoutFor(t).x,origY:this._layoutFor(t).y},window.addEventListener("mousemove",this._onMouseMove),window.addEventListener("mouseup",this._onMouseUp))}_onResizeDown(t,e){e.stopPropagation();let n=this._layoutFor(t);this._resizing={id:t,startX:e.clientX,startY:e.clientY,origW:n.width,origH:n.height},window.addEventListener("mousemove",this._onMouseMove),window.addEventListener("mouseup",this._onMouseUp)}_updateLayout(t,e){this._layouts=this._layouts.map(n=>n.data_source_id===t?{...n,...e}:n)}async saveLayouts(){try{let t=this.screenId==="all"?"__overview":this.screenId;await Ie(`/api/screens/${t}/canvas-layouts`,{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(this._layouts)})}catch{}}render(){return this._loading&&this._sensors.length===0?Z`<div style="display:flex;align-items:center;justify-content:center;height:200px;color:var(--muted)">Loading sensors...</div>`:Z`
      <div class="container">
        ${this._sensors.map(t=>{let e=this._layoutFor(t.id),n=(this._dataMap[String(t.id)]||[]).map(s=>s.value).filter(s=>s!=null&&!Number.isNaN(s));return Z`
            <div class="card-wrap ${this._selectedId===t.id?"selected":""}"
              style="left:${e.x}px;top:${e.y}px;width:${e.width}px;height:${e.height}px"
              @mousedown="${s=>this._onMouseDown(t.id,s)}"
            >
              <div class="drag-bar"></div>
              <vlt-sensor-card .sensor="${{config:t,data:this._currentValues[String(t.id)],chartData:n}}"></vlt-sensor-card>
              <div class="resize-handle" @mousedown="${s=>this._onResizeDown(t.id,s)}"></div>
            </div>
          `})}
      </div>
    `}};Cn([et()],pn.prototype,"_sensors",void 0);Cn([et()],pn.prototype,"_layouts",void 0);Cn([et()],pn.prototype,"_dataMap",void 0);Cn([et()],pn.prototype,"_currentValues",void 0);Cn([et()],pn.prototype,"_selectedId",void 0);Cn([et()],pn.prototype,"_dragging",void 0);Cn([et()],pn.prototype,"_resizing",void 0);Cn([et()],pn.prototype,"_loading",void 0);Cn([le({type:String})],pn.prototype,"screenId",void 0);pn=Cn([Zt("vlt-canvas-view")],pn);var ol=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Nr=class extends kt{constructor(){super(...arguments),this._editMode=!1,this._tool="select"}static{this.styles=qt`
    :host { display: block; height: 100%; padding: 20px; overflow: auto; }
    .toolbar {
      display: flex; align-items: center; gap: 8px; margin-bottom: 12px;
    }
    .toolbar button {
      padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 12px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .canvas {
      position: relative; width: 100%; height: calc(100% - 60px);
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
    }
    .placeholder {
      display: flex; align-items: center; justify-content: center;
      height: 100%; color: var(--muted); font-size: 14px;
    }
  `}render(){return Z`
      <div class="toolbar">
        <button class="${this._tool==="select"?"active":""}" @click="${()=>this._tool="select"}">Select</button>
        <button class="${this._tool==="connect"?"active":""}" @click="${()=>this._tool="connect"}">Connect</button>
        <button class="${this._tool==="delete"?"active":""}" @click="${()=>this._tool="delete"}">Delete</button>
        <button @click="${()=>this._editMode=!this._editMode}">${this._editMode?"Done":"Edit"}</button>
      </div>
      <div class="canvas">
        <div class="placeholder">
          2D SCADA Diagram Editor — equipment palette, drag/drop, and connections coming soon.
          <br>Use the React client for full SCADA editing.
        </div>
      </div>
    `}};ol([et()],Nr.prototype,"_editMode",void 0);ol([et()],Nr.prototype,"_tool",void 0);Nr=ol([Zt("vlt-scada-view")],Nr);var Ph=0,Hl=1,Lh=2;var or=1,Dh=2,vs=3,Zn=0,ze=1,Un=2,Fn=0,ki=1,Wl=2,$l=3,Xl=4,Nh=5;var ui=100,Uh=101,Fh=102,Oh=103,Bh=104,kh=200,zh=201,Vh=202,Gh=203,sa=204,ra=205,Hh=206,Wh=207,$h=208,Xh=209,qh=210,Yh=211,Zh=212,Jh=213,Kh=214,aa=0,oa=1,la=2,zi=3,ca=4,ha=5,da=6,ua=7,ql=0,jh=1,Qh=2,xn=0,Yl=1,Zl=2,Jl=3,Kl=4,jl=5,Ql=6,tc=7;var ec=300,vi=301,Hi=302,Ga=303,Ha=304,lr=306,fa=1e3,Pn=1001,pa=1002,Re=1003,td=1004;var cr=1005;var Pe=1006,Wa=1007;var xi=1008;var He=1009,nc=1010,ic=1011,xs=1012,$a=1013,yn=1014,bn=1015,On=1016,Xa=1017,qa=1018,ys=1020,sc=35902,rc=35899,ac=1021,oc=1022,hn=1023,Ln=1026,yi=1027,lc=1028,Ya=1029,bi=1030,Za=1031;var Ja=1033,hr=33776,dr=33777,ur=33778,fr=33779,Ka=35840,ja=35841,Qa=35842,to=35843,eo=36196,no=37492,io=37496,so=37488,ro=37489,pr=37490,ao=37491,oo=37808,lo=37809,co=37810,ho=37811,uo=37812,fo=37813,po=37814,mo=37815,go=37816,_o=37817,vo=37818,xo=37819,yo=37820,bo=37821,So=36492,Mo=36494,wo=36495,Ao=36283,Eo=36284,mr=36285,To=36286;var Vs=2300,ma=2301,ia=2302,Dl=2303,Nl=2400,Ul=2401,Fl=2402;var ed=3200;var Co=0,nd=1,Qn="",Ze="srgb",Gs="srgb-linear",Hs="linear",te="srgb";var Oi=7680;var Ol=519,id=512,sd=513,rd=514,Ro=515,ad=516,od=517,Io=518,ld=519,Bl=35044;var cc="300 es",vn=2e3,ds=2001;function xu(i){for(let t=i.length-1;t>=0;--t)if(i[t]>=65535)return!0;return!1}function yu(i){return ArrayBuffer.isView(i)&&!(i instanceof DataView)}function Ws(i){return document.createElementNS("http://www.w3.org/1999/xhtml",i)}function cd(){let i=Ws("canvas");return i.style.display="block",i}var oh={},us=null;function hc(...i){let t="THREE."+i.shift();us?us("log",t,...i):console.log(t,...i)}function hd(i){let t=i[0];if(typeof t=="string"&&t.startsWith("TSL:")){let e=i[1];e&&e.isStackTrace?i[0]+=" "+e.getLocation():i[1]='Stack trace not available. Enable "THREE.Node.captureStackTrace" to capture stack traces.'}return i}function Et(...i){i=hd(i);let t="THREE."+i.shift();if(us)us("warn",t,...i);else{let e=i[0];e&&e.isStackTrace?console.warn(e.getError(t)):console.warn(t,...i)}}function Ct(...i){i=hd(i);let t="THREE."+i.shift();if(us)us("error",t,...i);else{let e=i[0];e&&e.isStackTrace?console.error(e.getError(t)):console.error(t,...i)}}function ga(...i){let t=i.join(" ");t in oh||(oh[t]=!0,Et(...i))}function dd(i,t,e){return new Promise(function(n,s){function r(){switch(i.clientWaitSync(t,i.SYNC_FLUSH_COMMANDS_BIT,0)){case i.WAIT_FAILED:s();break;case i.TIMEOUT_EXPIRED:setTimeout(r,e);break;default:n()}}setTimeout(r,e)})}var ud={[aa]:oa,[la]:da,[ca]:ua,[zi]:ha,[oa]:aa,[da]:la,[ua]:ca,[ha]:zi},Dn=class{addEventListener(t,e){this._listeners===void 0&&(this._listeners={});let n=this._listeners;n[t]===void 0&&(n[t]=[]),n[t].indexOf(e)===-1&&n[t].push(e)}hasEventListener(t,e){let n=this._listeners;return n===void 0?!1:n[t]!==void 0&&n[t].indexOf(e)!==-1}removeEventListener(t,e){let n=this._listeners;if(n===void 0)return;let s=n[t];if(s!==void 0){let r=s.indexOf(e);r!==-1&&s.splice(r,1)}}dispatchEvent(t){let e=this._listeners;if(e===void 0)return;let n=e[t.type];if(n!==void 0){t.target=this;let s=n.slice(0);for(let r=0,a=s.length;r<a;r++)s[r].call(this,t);t.target=null}}},De=["00","01","02","03","04","05","06","07","08","09","0a","0b","0c","0d","0e","0f","10","11","12","13","14","15","16","17","18","19","1a","1b","1c","1d","1e","1f","20","21","22","23","24","25","26","27","28","29","2a","2b","2c","2d","2e","2f","30","31","32","33","34","35","36","37","38","39","3a","3b","3c","3d","3e","3f","40","41","42","43","44","45","46","47","48","49","4a","4b","4c","4d","4e","4f","50","51","52","53","54","55","56","57","58","59","5a","5b","5c","5d","5e","5f","60","61","62","63","64","65","66","67","68","69","6a","6b","6c","6d","6e","6f","70","71","72","73","74","75","76","77","78","79","7a","7b","7c","7d","7e","7f","80","81","82","83","84","85","86","87","88","89","8a","8b","8c","8d","8e","8f","90","91","92","93","94","95","96","97","98","99","9a","9b","9c","9d","9e","9f","a0","a1","a2","a3","a4","a5","a6","a7","a8","a9","aa","ab","ac","ad","ae","af","b0","b1","b2","b3","b4","b5","b6","b7","b8","b9","ba","bb","bc","bd","be","bf","c0","c1","c2","c3","c4","c5","c6","c7","c8","c9","ca","cb","cc","cd","ce","cf","d0","d1","d2","d3","d4","d5","d6","d7","d8","d9","da","db","dc","dd","de","df","e0","e1","e2","e3","e4","e5","e6","e7","e8","e9","ea","eb","ec","ed","ee","ef","f0","f1","f2","f3","f4","f5","f6","f7","f8","f9","fa","fb","fc","fd","fe","ff"];var ll=Math.PI/180,_a=180/Math.PI;function gr(){let i=Math.random()*4294967295|0,t=Math.random()*4294967295|0,e=Math.random()*4294967295|0,n=Math.random()*4294967295|0;return(De[i&255]+De[i>>8&255]+De[i>>16&255]+De[i>>24&255]+"-"+De[t&255]+De[t>>8&255]+"-"+De[t>>16&15|64]+De[t>>24&255]+"-"+De[e&63|128]+De[e>>8&255]+"-"+De[e>>16&255]+De[e>>24&255]+De[n&255]+De[n>>8&255]+De[n>>16&255]+De[n>>24&255]).toLowerCase()}function Xt(i,t,e){return Math.max(t,Math.min(e,i))}function bu(i,t){return(i%t+t)%t}function cl(i,t,e){return(1-e)*i+e*t}function Us(i,t){switch(t.constructor){case Float32Array:return i;case Uint32Array:return i/4294967295;case Uint16Array:return i/65535;case Uint8Array:return i/255;case Int32Array:return Math.max(i/2147483647,-1);case Int16Array:return Math.max(i/32767,-1);case Int8Array:return Math.max(i/127,-1);default:throw new Error("Invalid component type.")}}function Ve(i,t){switch(t.constructor){case Float32Array:return i;case Uint32Array:return Math.round(i*4294967295);case Uint16Array:return Math.round(i*65535);case Uint8Array:return Math.round(i*255);case Int32Array:return Math.round(i*2147483647);case Int16Array:return Math.round(i*32767);case Int8Array:return Math.round(i*127);default:throw new Error("Invalid component type.")}}var jt=class i{static{i.prototype.isVector2=!0}constructor(t=0,e=0){this.x=t,this.y=e}get width(){return this.x}set width(t){this.x=t}get height(){return this.y}set height(t){this.y=t}set(t,e){return this.x=t,this.y=e,this}setScalar(t){return this.x=t,this.y=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y)}copy(t){return this.x=t.x,this.y=t.y,this}add(t){return this.x+=t.x,this.y+=t.y,this}addScalar(t){return this.x+=t,this.y+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this}subScalar(t){return this.x-=t,this.y-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this}multiply(t){return this.x*=t.x,this.y*=t.y,this}multiplyScalar(t){return this.x*=t,this.y*=t,this}divide(t){return this.x/=t.x,this.y/=t.y,this}divideScalar(t){return this.multiplyScalar(1/t)}applyMatrix3(t){let e=this.x,n=this.y,s=t.elements;return this.x=s[0]*e+s[3]*n+s[6],this.y=s[1]*e+s[4]*n+s[7],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this}clamp(t,e){return this.x=Xt(this.x,t.x,e.x),this.y=Xt(this.y,t.y,e.y),this}clampScalar(t,e){return this.x=Xt(this.x,t,e),this.y=Xt(this.y,t,e),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Xt(n,t,e))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this}negate(){return this.x=-this.x,this.y=-this.y,this}dot(t){return this.x*t.x+this.y*t.y}cross(t){return this.x*t.y-this.y*t.x}lengthSq(){return this.x*this.x+this.y*this.y}length(){return Math.sqrt(this.x*this.x+this.y*this.y)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)}normalize(){return this.divideScalar(this.length()||1)}angle(){return Math.atan2(-this.y,-this.x)+Math.PI}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let n=this.dot(t)/e;return Math.acos(Xt(n,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,n=this.y-t.y;return e*e+n*n}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this}equals(t){return t.x===this.x&&t.y===this.y}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this}rotateAround(t,e){let n=Math.cos(e),s=Math.sin(e),r=this.x-t.x,a=this.y-t.y;return this.x=r*n-a*s+t.x,this.y=r*s+a*n+t.y,this}random(){return this.x=Math.random(),this.y=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y}},Nn=class{constructor(t=0,e=0,n=0,s=1){this.isQuaternion=!0,this._x=t,this._y=e,this._z=n,this._w=s}static slerpFlat(t,e,n,s,r,a,o){let l=n[s+0],c=n[s+1],d=n[s+2],u=n[s+3],h=r[a+0],p=r[a+1],_=r[a+2],y=r[a+3];if(u!==y||l!==h||c!==p||d!==_){let m=l*h+c*p+d*_+u*y;m<0&&(h=-h,p=-p,_=-_,y=-y,m=-m);let f=1-o;if(m<.9995){let S=Math.acos(m),M=Math.sin(S);f=Math.sin(f*S)/M,o=Math.sin(o*S)/M,l=l*f+h*o,c=c*f+p*o,d=d*f+_*o,u=u*f+y*o}else{l=l*f+h*o,c=c*f+p*o,d=d*f+_*o,u=u*f+y*o;let S=1/Math.sqrt(l*l+c*c+d*d+u*u);l*=S,c*=S,d*=S,u*=S}}t[e]=l,t[e+1]=c,t[e+2]=d,t[e+3]=u}static multiplyQuaternionsFlat(t,e,n,s,r,a){let o=n[s],l=n[s+1],c=n[s+2],d=n[s+3],u=r[a],h=r[a+1],p=r[a+2],_=r[a+3];return t[e]=o*_+d*u+l*p-c*h,t[e+1]=l*_+d*h+c*u-o*p,t[e+2]=c*_+d*p+o*h-l*u,t[e+3]=d*_-o*u-l*h-c*p,t}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get w(){return this._w}set w(t){this._w=t,this._onChangeCallback()}set(t,e,n,s){return this._x=t,this._y=e,this._z=n,this._w=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._w)}copy(t){return this._x=t.x,this._y=t.y,this._z=t.z,this._w=t.w,this._onChangeCallback(),this}setFromEuler(t,e=!0){let n=t._x,s=t._y,r=t._z,a=t._order,o=Math.cos,l=Math.sin,c=o(n/2),d=o(s/2),u=o(r/2),h=l(n/2),p=l(s/2),_=l(r/2);switch(a){case"XYZ":this._x=h*d*u+c*p*_,this._y=c*p*u-h*d*_,this._z=c*d*_+h*p*u,this._w=c*d*u-h*p*_;break;case"YXZ":this._x=h*d*u+c*p*_,this._y=c*p*u-h*d*_,this._z=c*d*_-h*p*u,this._w=c*d*u+h*p*_;break;case"ZXY":this._x=h*d*u-c*p*_,this._y=c*p*u+h*d*_,this._z=c*d*_+h*p*u,this._w=c*d*u-h*p*_;break;case"ZYX":this._x=h*d*u-c*p*_,this._y=c*p*u+h*d*_,this._z=c*d*_-h*p*u,this._w=c*d*u+h*p*_;break;case"YZX":this._x=h*d*u+c*p*_,this._y=c*p*u+h*d*_,this._z=c*d*_-h*p*u,this._w=c*d*u-h*p*_;break;case"XZY":this._x=h*d*u-c*p*_,this._y=c*p*u-h*d*_,this._z=c*d*_+h*p*u,this._w=c*d*u+h*p*_;break;default:Et("Quaternion: .setFromEuler() encountered an unknown order: "+a)}return e===!0&&this._onChangeCallback(),this}setFromAxisAngle(t,e){let n=e/2,s=Math.sin(n);return this._x=t.x*s,this._y=t.y*s,this._z=t.z*s,this._w=Math.cos(n),this._onChangeCallback(),this}setFromRotationMatrix(t){let e=t.elements,n=e[0],s=e[4],r=e[8],a=e[1],o=e[5],l=e[9],c=e[2],d=e[6],u=e[10],h=n+o+u;if(h>0){let p=.5/Math.sqrt(h+1);this._w=.25/p,this._x=(d-l)*p,this._y=(r-c)*p,this._z=(a-s)*p}else if(n>o&&n>u){let p=2*Math.sqrt(1+n-o-u);this._w=(d-l)/p,this._x=.25*p,this._y=(s+a)/p,this._z=(r+c)/p}else if(o>u){let p=2*Math.sqrt(1+o-n-u);this._w=(r-c)/p,this._x=(s+a)/p,this._y=.25*p,this._z=(l+d)/p}else{let p=2*Math.sqrt(1+u-n-o);this._w=(a-s)/p,this._x=(r+c)/p,this._y=(l+d)/p,this._z=.25*p}return this._onChangeCallback(),this}setFromUnitVectors(t,e){let n=t.dot(e)+1;return n<1e-8?(n=0,Math.abs(t.x)>Math.abs(t.z)?(this._x=-t.y,this._y=t.x,this._z=0,this._w=n):(this._x=0,this._y=-t.z,this._z=t.y,this._w=n)):(this._x=t.y*e.z-t.z*e.y,this._y=t.z*e.x-t.x*e.z,this._z=t.x*e.y-t.y*e.x,this._w=n),this.normalize()}angleTo(t){return 2*Math.acos(Math.abs(Xt(this.dot(t),-1,1)))}rotateTowards(t,e){let n=this.angleTo(t);if(n===0)return this;let s=Math.min(1,e/n);return this.slerp(t,s),this}identity(){return this.set(0,0,0,1)}invert(){return this.conjugate()}conjugate(){return this._x*=-1,this._y*=-1,this._z*=-1,this._onChangeCallback(),this}dot(t){return this._x*t._x+this._y*t._y+this._z*t._z+this._w*t._w}lengthSq(){return this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w}length(){return Math.sqrt(this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w)}normalize(){let t=this.length();return t===0?(this._x=0,this._y=0,this._z=0,this._w=1):(t=1/t,this._x=this._x*t,this._y=this._y*t,this._z=this._z*t,this._w=this._w*t),this._onChangeCallback(),this}multiply(t){return this.multiplyQuaternions(this,t)}premultiply(t){return this.multiplyQuaternions(t,this)}multiplyQuaternions(t,e){let n=t._x,s=t._y,r=t._z,a=t._w,o=e._x,l=e._y,c=e._z,d=e._w;return this._x=n*d+a*o+s*c-r*l,this._y=s*d+a*l+r*o-n*c,this._z=r*d+a*c+n*l-s*o,this._w=a*d-n*o-s*l-r*c,this._onChangeCallback(),this}slerp(t,e){let n=t._x,s=t._y,r=t._z,a=t._w,o=this.dot(t);o<0&&(n=-n,s=-s,r=-r,a=-a,o=-o);let l=1-e;if(o<.9995){let c=Math.acos(o),d=Math.sin(c);l=Math.sin(l*c)/d,e=Math.sin(e*c)/d,this._x=this._x*l+n*e,this._y=this._y*l+s*e,this._z=this._z*l+r*e,this._w=this._w*l+a*e,this._onChangeCallback()}else this._x=this._x*l+n*e,this._y=this._y*l+s*e,this._z=this._z*l+r*e,this._w=this._w*l+a*e,this.normalize();return this}slerpQuaternions(t,e,n){return this.copy(t).slerp(e,n)}random(){let t=2*Math.PI*Math.random(),e=2*Math.PI*Math.random(),n=Math.random(),s=Math.sqrt(1-n),r=Math.sqrt(n);return this.set(s*Math.sin(t),s*Math.cos(t),r*Math.sin(e),r*Math.cos(e))}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._w===this._w}fromArray(t,e=0){return this._x=t[e],this._y=t[e+1],this._z=t[e+2],this._w=t[e+3],this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._w,t}fromBufferAttribute(t,e){return this._x=t.getX(e),this._y=t.getY(e),this._z=t.getZ(e),this._w=t.getW(e),this._onChangeCallback(),this}toJSON(){return this.toArray()}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._w}},k=class i{static{i.prototype.isVector3=!0}constructor(t=0,e=0,n=0){this.x=t,this.y=e,this.z=n}set(t,e,n){return n===void 0&&(n=this.z),this.x=t,this.y=e,this.z=n,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this}multiplyVectors(t,e){return this.x=t.x*e.x,this.y=t.y*e.y,this.z=t.z*e.z,this}applyEuler(t){return this.applyQuaternion(lh.setFromEuler(t))}applyAxisAngle(t,e){return this.applyQuaternion(lh.setFromAxisAngle(t,e))}applyMatrix3(t){let e=this.x,n=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[3]*n+r[6]*s,this.y=r[1]*e+r[4]*n+r[7]*s,this.z=r[2]*e+r[5]*n+r[8]*s,this}applyNormalMatrix(t){return this.applyMatrix3(t).normalize()}applyMatrix4(t){let e=this.x,n=this.y,s=this.z,r=t.elements,a=1/(r[3]*e+r[7]*n+r[11]*s+r[15]);return this.x=(r[0]*e+r[4]*n+r[8]*s+r[12])*a,this.y=(r[1]*e+r[5]*n+r[9]*s+r[13])*a,this.z=(r[2]*e+r[6]*n+r[10]*s+r[14])*a,this}applyQuaternion(t){let e=this.x,n=this.y,s=this.z,r=t.x,a=t.y,o=t.z,l=t.w,c=2*(a*s-o*n),d=2*(o*e-r*s),u=2*(r*n-a*e);return this.x=e+l*c+a*u-o*d,this.y=n+l*d+o*c-r*u,this.z=s+l*u+r*d-a*c,this}project(t){return this.applyMatrix4(t.matrixWorldInverse).applyMatrix4(t.projectionMatrix)}unproject(t){return this.applyMatrix4(t.projectionMatrixInverse).applyMatrix4(t.matrixWorld)}transformDirection(t){let e=this.x,n=this.y,s=this.z,r=t.elements;return this.x=r[0]*e+r[4]*n+r[8]*s,this.y=r[1]*e+r[5]*n+r[9]*s,this.z=r[2]*e+r[6]*n+r[10]*s,this.normalize()}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this}divideScalar(t){return this.multiplyScalar(1/t)}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this}clamp(t,e){return this.x=Xt(this.x,t.x,e.x),this.y=Xt(this.y,t.y,e.y),this.z=Xt(this.z,t.z,e.z),this}clampScalar(t,e){return this.x=Xt(this.x,t,e),this.y=Xt(this.y,t,e),this.z=Xt(this.z,t,e),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Xt(n,t,e))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this.z=t.z+(e.z-t.z)*n,this}cross(t){return this.crossVectors(this,t)}crossVectors(t,e){let n=t.x,s=t.y,r=t.z,a=e.x,o=e.y,l=e.z;return this.x=s*l-r*o,this.y=r*a-n*l,this.z=n*o-s*a,this}projectOnVector(t){let e=t.lengthSq();if(e===0)return this.set(0,0,0);let n=t.dot(this)/e;return this.copy(t).multiplyScalar(n)}projectOnPlane(t){return hl.copy(this).projectOnVector(t),this.sub(hl)}reflect(t){return this.sub(hl.copy(t).multiplyScalar(2*this.dot(t)))}angleTo(t){let e=Math.sqrt(this.lengthSq()*t.lengthSq());if(e===0)return Math.PI/2;let n=this.dot(t)/e;return Math.acos(Xt(n,-1,1))}distanceTo(t){return Math.sqrt(this.distanceToSquared(t))}distanceToSquared(t){let e=this.x-t.x,n=this.y-t.y,s=this.z-t.z;return e*e+n*n+s*s}manhattanDistanceTo(t){return Math.abs(this.x-t.x)+Math.abs(this.y-t.y)+Math.abs(this.z-t.z)}setFromSpherical(t){return this.setFromSphericalCoords(t.radius,t.phi,t.theta)}setFromSphericalCoords(t,e,n){let s=Math.sin(e)*t;return this.x=s*Math.sin(n),this.y=Math.cos(e)*t,this.z=s*Math.cos(n),this}setFromCylindrical(t){return this.setFromCylindricalCoords(t.radius,t.theta,t.y)}setFromCylindricalCoords(t,e,n){return this.x=t*Math.sin(e),this.y=n,this.z=t*Math.cos(e),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this}setFromMatrixScale(t){let e=this.setFromMatrixColumn(t,0).length(),n=this.setFromMatrixColumn(t,1).length(),s=this.setFromMatrixColumn(t,2).length();return this.x=e,this.y=n,this.z=s,this}setFromMatrixColumn(t,e){return this.fromArray(t.elements,e*4)}setFromMatrix3Column(t,e){return this.fromArray(t.elements,e*3)}setFromEuler(t){return this.x=t._x,this.y=t._y,this.z=t._z,this}setFromColor(t){return this.x=t.r,this.y=t.g,this.z=t.b,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this}randomDirection(){let t=Math.random()*Math.PI*2,e=Math.random()*2-1,n=Math.sqrt(1-e*e);return this.x=n*Math.cos(t),this.y=e,this.z=n*Math.sin(t),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z}},hl=new k,lh=new Nn,Pt=class i{static{i.prototype.isMatrix3=!0}constructor(t,e,n,s,r,a,o,l,c){this.elements=[1,0,0,0,1,0,0,0,1],t!==void 0&&this.set(t,e,n,s,r,a,o,l,c)}set(t,e,n,s,r,a,o,l,c){let d=this.elements;return d[0]=t,d[1]=s,d[2]=o,d[3]=e,d[4]=r,d[5]=l,d[6]=n,d[7]=a,d[8]=c,this}identity(){return this.set(1,0,0,0,1,0,0,0,1),this}copy(t){let e=this.elements,n=t.elements;return e[0]=n[0],e[1]=n[1],e[2]=n[2],e[3]=n[3],e[4]=n[4],e[5]=n[5],e[6]=n[6],e[7]=n[7],e[8]=n[8],this}extractBasis(t,e,n){return t.setFromMatrix3Column(this,0),e.setFromMatrix3Column(this,1),n.setFromMatrix3Column(this,2),this}setFromMatrix4(t){let e=t.elements;return this.set(e[0],e[4],e[8],e[1],e[5],e[9],e[2],e[6],e[10]),this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let n=t.elements,s=e.elements,r=this.elements,a=n[0],o=n[3],l=n[6],c=n[1],d=n[4],u=n[7],h=n[2],p=n[5],_=n[8],y=s[0],m=s[3],f=s[6],S=s[1],M=s[4],E=s[7],R=s[2],w=s[5],I=s[8];return r[0]=a*y+o*S+l*R,r[3]=a*m+o*M+l*w,r[6]=a*f+o*E+l*I,r[1]=c*y+d*S+u*R,r[4]=c*m+d*M+u*w,r[7]=c*f+d*E+u*I,r[2]=h*y+p*S+_*R,r[5]=h*m+p*M+_*w,r[8]=h*f+p*E+_*I,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[3]*=t,e[6]*=t,e[1]*=t,e[4]*=t,e[7]*=t,e[2]*=t,e[5]*=t,e[8]*=t,this}determinant(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],d=t[8];return e*a*d-e*o*c-n*r*d+n*o*l+s*r*c-s*a*l}invert(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],d=t[8],u=d*a-o*c,h=o*l-d*r,p=c*r-a*l,_=e*u+n*h+s*p;if(_===0)return this.set(0,0,0,0,0,0,0,0,0);let y=1/_;return t[0]=u*y,t[1]=(s*c-d*n)*y,t[2]=(o*n-s*a)*y,t[3]=h*y,t[4]=(d*e-s*l)*y,t[5]=(s*r-o*e)*y,t[6]=p*y,t[7]=(n*l-c*e)*y,t[8]=(a*e-n*r)*y,this}transpose(){let t,e=this.elements;return t=e[1],e[1]=e[3],e[3]=t,t=e[2],e[2]=e[6],e[6]=t,t=e[5],e[5]=e[7],e[7]=t,this}getNormalMatrix(t){return this.setFromMatrix4(t).invert().transpose()}transposeIntoArray(t){let e=this.elements;return t[0]=e[0],t[1]=e[3],t[2]=e[6],t[3]=e[1],t[4]=e[4],t[5]=e[7],t[6]=e[2],t[7]=e[5],t[8]=e[8],this}setUvTransform(t,e,n,s,r,a,o){let l=Math.cos(r),c=Math.sin(r);return this.set(n*l,n*c,-n*(l*a+c*o)+a+t,-s*c,s*l,-s*(-c*a+l*o)+o+e,0,0,1),this}scale(t,e){return this.premultiply(dl.makeScale(t,e)),this}rotate(t){return this.premultiply(dl.makeRotation(-t)),this}translate(t,e){return this.premultiply(dl.makeTranslation(t,e)),this}makeTranslation(t,e){return t.isVector2?this.set(1,0,t.x,0,1,t.y,0,0,1):this.set(1,0,t,0,1,e,0,0,1),this}makeRotation(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,-n,0,n,e,0,0,0,1),this}makeScale(t,e){return this.set(t,0,0,0,e,0,0,0,1),this}equals(t){let e=this.elements,n=t.elements;for(let s=0;s<9;s++)if(e[s]!==n[s])return!1;return!0}fromArray(t,e=0){for(let n=0;n<9;n++)this.elements[n]=t[n+e];return this}toArray(t=[],e=0){let n=this.elements;return t[e]=n[0],t[e+1]=n[1],t[e+2]=n[2],t[e+3]=n[3],t[e+4]=n[4],t[e+5]=n[5],t[e+6]=n[6],t[e+7]=n[7],t[e+8]=n[8],t}clone(){return new this.constructor().fromArray(this.elements)}},dl=new Pt,ch=new Pt().set(.4123908,.3575843,.1804808,.212639,.7151687,.0721923,.0193308,.1191948,.9505322),hh=new Pt().set(3.2409699,-1.5373832,-.4986108,-.9692436,1.8759675,.0415551,.0556301,-.203977,1.0569715);function Su(){let i={enabled:!0,workingColorSpace:Gs,spaces:{},convert:function(s,r,a){return this.enabled===!1||r===a||!r||!a||(this.spaces[r].transfer===te&&(s.r=Yn(s.r),s.g=Yn(s.g),s.b=Yn(s.b)),this.spaces[r].primaries!==this.spaces[a].primaries&&(s.applyMatrix3(this.spaces[r].toXYZ),s.applyMatrix3(this.spaces[a].fromXYZ)),this.spaces[a].transfer===te&&(s.r=hs(s.r),s.g=hs(s.g),s.b=hs(s.b))),s},workingToColorSpace:function(s,r){return this.convert(s,this.workingColorSpace,r)},colorSpaceToWorking:function(s,r){return this.convert(s,r,this.workingColorSpace)},getPrimaries:function(s){return this.spaces[s].primaries},getTransfer:function(s){return s===Qn?Hs:this.spaces[s].transfer},getToneMappingMode:function(s){return this.spaces[s].outputColorSpaceConfig.toneMappingMode||"standard"},getLuminanceCoefficients:function(s,r=this.workingColorSpace){return s.fromArray(this.spaces[r].luminanceCoefficients)},define:function(s){Object.assign(this.spaces,s)},_getMatrix:function(s,r,a){return s.copy(this.spaces[r].toXYZ).multiply(this.spaces[a].fromXYZ)},_getDrawingBufferColorSpace:function(s){return this.spaces[s].outputColorSpaceConfig.drawingBufferColorSpace},_getUnpackColorSpace:function(s=this.workingColorSpace){return this.spaces[s].workingColorSpaceConfig.unpackColorSpace},fromWorkingColorSpace:function(s,r){return ga("ColorManagement: .fromWorkingColorSpace() has been renamed to .workingToColorSpace()."),i.workingToColorSpace(s,r)},toWorkingColorSpace:function(s,r){return ga("ColorManagement: .toWorkingColorSpace() has been renamed to .colorSpaceToWorking()."),i.colorSpaceToWorking(s,r)}},t=[.64,.33,.3,.6,.15,.06],e=[.2126,.7152,.0722],n=[.3127,.329];return i.define({[Gs]:{primaries:t,whitePoint:n,transfer:Hs,toXYZ:ch,fromXYZ:hh,luminanceCoefficients:e,workingColorSpaceConfig:{unpackColorSpace:Ze},outputColorSpaceConfig:{drawingBufferColorSpace:Ze}},[Ze]:{primaries:t,whitePoint:n,transfer:te,toXYZ:ch,fromXYZ:hh,luminanceCoefficients:e,outputColorSpaceConfig:{drawingBufferColorSpace:Ze}}}),i}var $t=Su();function Yn(i){return i<.04045?i*.0773993808:Math.pow(i*.9478672986+.0521327014,2.4)}function hs(i){return i<.0031308?i*12.92:1.055*Math.pow(i,.41666)-.055}var ji,va=class{static getDataURL(t,e="image/png"){if(/^data:/i.test(t.src)||typeof HTMLCanvasElement>"u")return t.src;let n;if(t instanceof HTMLCanvasElement)n=t;else{ji===void 0&&(ji=Ws("canvas")),ji.width=t.width,ji.height=t.height;let s=ji.getContext("2d");t instanceof ImageData?s.putImageData(t,0,0):s.drawImage(t,0,0,t.width,t.height),n=ji}return n.toDataURL(e)}static sRGBToLinear(t){if(typeof HTMLImageElement<"u"&&t instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&t instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&t instanceof ImageBitmap){let e=Ws("canvas");e.width=t.width,e.height=t.height;let n=e.getContext("2d");n.drawImage(t,0,0,t.width,t.height);let s=n.getImageData(0,0,t.width,t.height),r=s.data;for(let a=0;a<r.length;a++)r[a]=Yn(r[a]/255)*255;return n.putImageData(s,0,0),e}else if(t.data){let e=t.data.slice(0);for(let n=0;n<e.length;n++)e instanceof Uint8Array||e instanceof Uint8ClampedArray?e[n]=Math.floor(Yn(e[n]/255)*255):e[n]=Yn(e[n]);return{data:e,width:t.width,height:t.height}}else return Et("ImageUtils.sRGBToLinear(): Unsupported image type. No color space conversion applied."),t}},Mu=0,fs=class{constructor(t=null){this.isSource=!0,Object.defineProperty(this,"id",{value:Mu++}),this.uuid=gr(),this.data=t,this.dataReady=!0,this.version=0}getSize(t){let e=this.data;return typeof HTMLVideoElement<"u"&&e instanceof HTMLVideoElement?t.set(e.videoWidth,e.videoHeight,0):typeof VideoFrame<"u"&&e instanceof VideoFrame?t.set(e.displayWidth,e.displayHeight,0):e!==null?t.set(e.width,e.height,e.depth||0):t.set(0,0,0),t}set needsUpdate(t){t===!0&&this.version++}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.images[this.uuid]!==void 0)return t.images[this.uuid];let n={uuid:this.uuid,url:""},s=this.data;if(s!==null){let r;if(Array.isArray(s)){r=[];for(let a=0,o=s.length;a<o;a++)s[a].isDataTexture?r.push(ul(s[a].image)):r.push(ul(s[a]))}else r=ul(s);n.url=r}return e||(t.images[this.uuid]=n),n}};function ul(i){return typeof HTMLImageElement<"u"&&i instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&i instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&i instanceof ImageBitmap?va.getDataURL(i):i.data?{data:Array.from(i.data),width:i.width,height:i.height,type:i.data.constructor.name}:(Et("Texture: Unable to serialize Texture."),{})}var wu=0,fl=new k,Ge=class i extends Dn{constructor(t=i.DEFAULT_IMAGE,e=i.DEFAULT_MAPPING,n=Pn,s=Pn,r=Pe,a=xi,o=hn,l=He,c=i.DEFAULT_ANISOTROPY,d=Qn){super(),this.isTexture=!0,Object.defineProperty(this,"id",{value:wu++}),this.uuid=gr(),this.name="",this.source=new fs(t),this.mipmaps=[],this.mapping=e,this.channel=0,this.wrapS=n,this.wrapT=s,this.magFilter=r,this.minFilter=a,this.anisotropy=c,this.format=o,this.internalFormat=null,this.type=l,this.offset=new jt(0,0),this.repeat=new jt(1,1),this.center=new jt(0,0),this.rotation=0,this.matrixAutoUpdate=!0,this.matrix=new Pt,this.generateMipmaps=!0,this.premultiplyAlpha=!1,this.flipY=!0,this.unpackAlignment=4,this.colorSpace=d,this.userData={},this.updateRanges=[],this.version=0,this.onUpdate=null,this.renderTarget=null,this.isRenderTargetTexture=!1,this.isArrayTexture=!!(t&&t.depth&&t.depth>1),this.pmremVersion=0,this.normalized=!1}get width(){return this.source.getSize(fl).x}get height(){return this.source.getSize(fl).y}get depth(){return this.source.getSize(fl).z}get image(){return this.source.data}set image(t){this.source.data=t}updateMatrix(){this.matrix.setUvTransform(this.offset.x,this.offset.y,this.repeat.x,this.repeat.y,this.rotation,this.center.x,this.center.y)}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}clone(){return new this.constructor().copy(this)}copy(t){return this.name=t.name,this.source=t.source,this.mipmaps=t.mipmaps.slice(0),this.mapping=t.mapping,this.channel=t.channel,this.wrapS=t.wrapS,this.wrapT=t.wrapT,this.magFilter=t.magFilter,this.minFilter=t.minFilter,this.anisotropy=t.anisotropy,this.format=t.format,this.internalFormat=t.internalFormat,this.type=t.type,this.normalized=t.normalized,this.offset.copy(t.offset),this.repeat.copy(t.repeat),this.center.copy(t.center),this.rotation=t.rotation,this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrix.copy(t.matrix),this.generateMipmaps=t.generateMipmaps,this.premultiplyAlpha=t.premultiplyAlpha,this.flipY=t.flipY,this.unpackAlignment=t.unpackAlignment,this.colorSpace=t.colorSpace,this.renderTarget=t.renderTarget,this.isRenderTargetTexture=t.isRenderTargetTexture,this.isArrayTexture=t.isArrayTexture,this.userData=JSON.parse(JSON.stringify(t.userData)),this.needsUpdate=!0,this}setValues(t){for(let e in t){let n=t[e];if(n===void 0){Et(`Texture.setValues(): parameter '${e}' has value of undefined.`);continue}let s=this[e];if(s===void 0){Et(`Texture.setValues(): property '${e}' does not exist.`);continue}s&&n&&s.isVector2&&n.isVector2||s&&n&&s.isVector3&&n.isVector3||s&&n&&s.isMatrix3&&n.isMatrix3?s.copy(n):this[e]=n}}toJSON(t){let e=t===void 0||typeof t=="string";if(!e&&t.textures[this.uuid]!==void 0)return t.textures[this.uuid];let n={metadata:{version:4.7,type:"Texture",generator:"Texture.toJSON"},uuid:this.uuid,name:this.name,image:this.source.toJSON(t).uuid,mapping:this.mapping,channel:this.channel,repeat:[this.repeat.x,this.repeat.y],offset:[this.offset.x,this.offset.y],center:[this.center.x,this.center.y],rotation:this.rotation,wrap:[this.wrapS,this.wrapT],format:this.format,internalFormat:this.internalFormat,type:this.type,normalized:this.normalized,colorSpace:this.colorSpace,minFilter:this.minFilter,magFilter:this.magFilter,anisotropy:this.anisotropy,flipY:this.flipY,generateMipmaps:this.generateMipmaps,premultiplyAlpha:this.premultiplyAlpha,unpackAlignment:this.unpackAlignment};return Object.keys(this.userData).length>0&&(n.userData=this.userData),e||(t.textures[this.uuid]=n),n}dispose(){this.dispatchEvent({type:"dispose"})}transformUv(t){if(this.mapping!==ec)return t;if(t.applyMatrix3(this.matrix),t.x<0||t.x>1)switch(this.wrapS){case fa:t.x=t.x-Math.floor(t.x);break;case Pn:t.x=t.x<0?0:1;break;case pa:Math.abs(Math.floor(t.x)%2)===1?t.x=Math.ceil(t.x)-t.x:t.x=t.x-Math.floor(t.x);break}if(t.y<0||t.y>1)switch(this.wrapT){case fa:t.y=t.y-Math.floor(t.y);break;case Pn:t.y=t.y<0?0:1;break;case pa:Math.abs(Math.floor(t.y)%2)===1?t.y=Math.ceil(t.y)-t.y:t.y=t.y-Math.floor(t.y);break}return this.flipY&&(t.y=1-t.y),t}set needsUpdate(t){t===!0&&(this.version++,this.source.needsUpdate=!0)}set needsPMREMUpdate(t){t===!0&&this.pmremVersion++}};Ge.DEFAULT_IMAGE=null;Ge.DEFAULT_MAPPING=ec;Ge.DEFAULT_ANISOTROPY=1;var pe=class i{static{i.prototype.isVector4=!0}constructor(t=0,e=0,n=0,s=1){this.x=t,this.y=e,this.z=n,this.w=s}get width(){return this.z}set width(t){this.z=t}get height(){return this.w}set height(t){this.w=t}set(t,e,n,s){return this.x=t,this.y=e,this.z=n,this.w=s,this}setScalar(t){return this.x=t,this.y=t,this.z=t,this.w=t,this}setX(t){return this.x=t,this}setY(t){return this.y=t,this}setZ(t){return this.z=t,this}setW(t){return this.w=t,this}setComponent(t,e){switch(t){case 0:this.x=e;break;case 1:this.y=e;break;case 2:this.z=e;break;case 3:this.w=e;break;default:throw new Error("index is out of range: "+t)}return this}getComponent(t){switch(t){case 0:return this.x;case 1:return this.y;case 2:return this.z;case 3:return this.w;default:throw new Error("index is out of range: "+t)}}clone(){return new this.constructor(this.x,this.y,this.z,this.w)}copy(t){return this.x=t.x,this.y=t.y,this.z=t.z,this.w=t.w!==void 0?t.w:1,this}add(t){return this.x+=t.x,this.y+=t.y,this.z+=t.z,this.w+=t.w,this}addScalar(t){return this.x+=t,this.y+=t,this.z+=t,this.w+=t,this}addVectors(t,e){return this.x=t.x+e.x,this.y=t.y+e.y,this.z=t.z+e.z,this.w=t.w+e.w,this}addScaledVector(t,e){return this.x+=t.x*e,this.y+=t.y*e,this.z+=t.z*e,this.w+=t.w*e,this}sub(t){return this.x-=t.x,this.y-=t.y,this.z-=t.z,this.w-=t.w,this}subScalar(t){return this.x-=t,this.y-=t,this.z-=t,this.w-=t,this}subVectors(t,e){return this.x=t.x-e.x,this.y=t.y-e.y,this.z=t.z-e.z,this.w=t.w-e.w,this}multiply(t){return this.x*=t.x,this.y*=t.y,this.z*=t.z,this.w*=t.w,this}multiplyScalar(t){return this.x*=t,this.y*=t,this.z*=t,this.w*=t,this}applyMatrix4(t){let e=this.x,n=this.y,s=this.z,r=this.w,a=t.elements;return this.x=a[0]*e+a[4]*n+a[8]*s+a[12]*r,this.y=a[1]*e+a[5]*n+a[9]*s+a[13]*r,this.z=a[2]*e+a[6]*n+a[10]*s+a[14]*r,this.w=a[3]*e+a[7]*n+a[11]*s+a[15]*r,this}divide(t){return this.x/=t.x,this.y/=t.y,this.z/=t.z,this.w/=t.w,this}divideScalar(t){return this.multiplyScalar(1/t)}setAxisAngleFromQuaternion(t){this.w=2*Math.acos(t.w);let e=Math.sqrt(1-t.w*t.w);return e<1e-4?(this.x=1,this.y=0,this.z=0):(this.x=t.x/e,this.y=t.y/e,this.z=t.z/e),this}setAxisAngleFromRotationMatrix(t){let e,n,s,r,l=t.elements,c=l[0],d=l[4],u=l[8],h=l[1],p=l[5],_=l[9],y=l[2],m=l[6],f=l[10];if(Math.abs(d-h)<.01&&Math.abs(u-y)<.01&&Math.abs(_-m)<.01){if(Math.abs(d+h)<.1&&Math.abs(u+y)<.1&&Math.abs(_+m)<.1&&Math.abs(c+p+f-3)<.1)return this.set(1,0,0,0),this;e=Math.PI;let M=(c+1)/2,E=(p+1)/2,R=(f+1)/2,w=(d+h)/4,I=(u+y)/4,v=(_+m)/4;return M>E&&M>R?M<.01?(n=0,s=.707106781,r=.707106781):(n=Math.sqrt(M),s=w/n,r=I/n):E>R?E<.01?(n=.707106781,s=0,r=.707106781):(s=Math.sqrt(E),n=w/s,r=v/s):R<.01?(n=.707106781,s=.707106781,r=0):(r=Math.sqrt(R),n=I/r,s=v/r),this.set(n,s,r,e),this}let S=Math.sqrt((m-_)*(m-_)+(u-y)*(u-y)+(h-d)*(h-d));return Math.abs(S)<.001&&(S=1),this.x=(m-_)/S,this.y=(u-y)/S,this.z=(h-d)/S,this.w=Math.acos((c+p+f-1)/2),this}setFromMatrixPosition(t){let e=t.elements;return this.x=e[12],this.y=e[13],this.z=e[14],this.w=e[15],this}min(t){return this.x=Math.min(this.x,t.x),this.y=Math.min(this.y,t.y),this.z=Math.min(this.z,t.z),this.w=Math.min(this.w,t.w),this}max(t){return this.x=Math.max(this.x,t.x),this.y=Math.max(this.y,t.y),this.z=Math.max(this.z,t.z),this.w=Math.max(this.w,t.w),this}clamp(t,e){return this.x=Xt(this.x,t.x,e.x),this.y=Xt(this.y,t.y,e.y),this.z=Xt(this.z,t.z,e.z),this.w=Xt(this.w,t.w,e.w),this}clampScalar(t,e){return this.x=Xt(this.x,t,e),this.y=Xt(this.y,t,e),this.z=Xt(this.z,t,e),this.w=Xt(this.w,t,e),this}clampLength(t,e){let n=this.length();return this.divideScalar(n||1).multiplyScalar(Xt(n,t,e))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this.w=Math.floor(this.w),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this.w=Math.ceil(this.w),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this.w=Math.round(this.w),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this.w=Math.trunc(this.w),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this.w=-this.w,this}dot(t){return this.x*t.x+this.y*t.y+this.z*t.z+this.w*t.w}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)+Math.abs(this.w)}normalize(){return this.divideScalar(this.length()||1)}setLength(t){return this.normalize().multiplyScalar(t)}lerp(t,e){return this.x+=(t.x-this.x)*e,this.y+=(t.y-this.y)*e,this.z+=(t.z-this.z)*e,this.w+=(t.w-this.w)*e,this}lerpVectors(t,e,n){return this.x=t.x+(e.x-t.x)*n,this.y=t.y+(e.y-t.y)*n,this.z=t.z+(e.z-t.z)*n,this.w=t.w+(e.w-t.w)*n,this}equals(t){return t.x===this.x&&t.y===this.y&&t.z===this.z&&t.w===this.w}fromArray(t,e=0){return this.x=t[e],this.y=t[e+1],this.z=t[e+2],this.w=t[e+3],this}toArray(t=[],e=0){return t[e]=this.x,t[e+1]=this.y,t[e+2]=this.z,t[e+3]=this.w,t}fromBufferAttribute(t,e){return this.x=t.getX(e),this.y=t.getY(e),this.z=t.getZ(e),this.w=t.getW(e),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this.w=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z,yield this.w}},xa=class extends Dn{constructor(t=1,e=1,n={}){super(),n=Object.assign({generateMipmaps:!1,internalFormat:null,minFilter:Pe,depthBuffer:!0,stencilBuffer:!1,resolveDepthBuffer:!0,resolveStencilBuffer:!0,depthTexture:null,samples:0,count:1,depth:1,multiview:!1},n),this.isRenderTarget=!0,this.width=t,this.height=e,this.depth=n.depth,this.scissor=new pe(0,0,t,e),this.scissorTest=!1,this.viewport=new pe(0,0,t,e),this.textures=[];let s={width:t,height:e,depth:n.depth},r=new Ge(s),a=n.count;for(let o=0;o<a;o++)this.textures[o]=r.clone(),this.textures[o].isRenderTargetTexture=!0,this.textures[o].renderTarget=this;this._setTextureOptions(n),this.depthBuffer=n.depthBuffer,this.stencilBuffer=n.stencilBuffer,this.resolveDepthBuffer=n.resolveDepthBuffer,this.resolveStencilBuffer=n.resolveStencilBuffer,this._depthTexture=null,this.depthTexture=n.depthTexture,this.samples=n.samples,this.multiview=n.multiview}_setTextureOptions(t={}){let e={minFilter:Pe,generateMipmaps:!1,flipY:!1,internalFormat:null};t.mapping!==void 0&&(e.mapping=t.mapping),t.wrapS!==void 0&&(e.wrapS=t.wrapS),t.wrapT!==void 0&&(e.wrapT=t.wrapT),t.wrapR!==void 0&&(e.wrapR=t.wrapR),t.magFilter!==void 0&&(e.magFilter=t.magFilter),t.minFilter!==void 0&&(e.minFilter=t.minFilter),t.format!==void 0&&(e.format=t.format),t.type!==void 0&&(e.type=t.type),t.anisotropy!==void 0&&(e.anisotropy=t.anisotropy),t.colorSpace!==void 0&&(e.colorSpace=t.colorSpace),t.flipY!==void 0&&(e.flipY=t.flipY),t.generateMipmaps!==void 0&&(e.generateMipmaps=t.generateMipmaps),t.internalFormat!==void 0&&(e.internalFormat=t.internalFormat);for(let n=0;n<this.textures.length;n++)this.textures[n].setValues(e)}get texture(){return this.textures[0]}set texture(t){this.textures[0]=t}set depthTexture(t){this._depthTexture!==null&&(this._depthTexture.renderTarget=null),t!==null&&(t.renderTarget=this),this._depthTexture=t}get depthTexture(){return this._depthTexture}setSize(t,e,n=1){if(this.width!==t||this.height!==e||this.depth!==n){this.width=t,this.height=e,this.depth=n;for(let s=0,r=this.textures.length;s<r;s++)this.textures[s].image.width=t,this.textures[s].image.height=e,this.textures[s].image.depth=n,this.textures[s].isData3DTexture!==!0&&(this.textures[s].isArrayTexture=this.textures[s].image.depth>1);this.dispose()}this.viewport.set(0,0,t,e),this.scissor.set(0,0,t,e)}clone(){return new this.constructor().copy(this)}copy(t){this.width=t.width,this.height=t.height,this.depth=t.depth,this.scissor.copy(t.scissor),this.scissorTest=t.scissorTest,this.viewport.copy(t.viewport),this.textures.length=0;for(let e=0,n=t.textures.length;e<n;e++){this.textures[e]=t.textures[e].clone(),this.textures[e].isRenderTargetTexture=!0,this.textures[e].renderTarget=this;let s=Object.assign({},t.textures[e].image);this.textures[e].source=new fs(s)}return this.depthBuffer=t.depthBuffer,this.stencilBuffer=t.stencilBuffer,this.resolveDepthBuffer=t.resolveDepthBuffer,this.resolveStencilBuffer=t.resolveStencilBuffer,t.depthTexture!==null&&(this.depthTexture=t.depthTexture.clone()),this.samples=t.samples,this.multiview=t.multiview,this}dispose(){this.dispatchEvent({type:"dispose"})}},Ke=class extends xa{constructor(t=1,e=1,n={}){super(t,e,n),this.isWebGLRenderTarget=!0}},$s=class extends Ge{constructor(t=null,e=1,n=1,s=1){super(null),this.isDataArrayTexture=!0,this.image={data:t,width:e,height:n,depth:s},this.magFilter=Re,this.minFilter=Re,this.wrapR=Pn,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1,this.layerUpdates=new Set}addLayerUpdate(t){this.layerUpdates.add(t)}clearLayerUpdates(){this.layerUpdates.clear()}};var ya=class extends Ge{constructor(t=null,e=1,n=1,s=1){super(null),this.isData3DTexture=!0,this.image={data:t,width:e,height:n,depth:s},this.magFilter=Re,this.minFilter=Re,this.wrapR=Pn,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var fe=class i{static{i.prototype.isMatrix4=!0}constructor(t,e,n,s,r,a,o,l,c,d,u,h,p,_,y,m){this.elements=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],t!==void 0&&this.set(t,e,n,s,r,a,o,l,c,d,u,h,p,_,y,m)}set(t,e,n,s,r,a,o,l,c,d,u,h,p,_,y,m){let f=this.elements;return f[0]=t,f[4]=e,f[8]=n,f[12]=s,f[1]=r,f[5]=a,f[9]=o,f[13]=l,f[2]=c,f[6]=d,f[10]=u,f[14]=h,f[3]=p,f[7]=_,f[11]=y,f[15]=m,this}identity(){return this.set(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1),this}clone(){return new i().fromArray(this.elements)}copy(t){let e=this.elements,n=t.elements;return e[0]=n[0],e[1]=n[1],e[2]=n[2],e[3]=n[3],e[4]=n[4],e[5]=n[5],e[6]=n[6],e[7]=n[7],e[8]=n[8],e[9]=n[9],e[10]=n[10],e[11]=n[11],e[12]=n[12],e[13]=n[13],e[14]=n[14],e[15]=n[15],this}copyPosition(t){let e=this.elements,n=t.elements;return e[12]=n[12],e[13]=n[13],e[14]=n[14],this}setFromMatrix3(t){let e=t.elements;return this.set(e[0],e[3],e[6],0,e[1],e[4],e[7],0,e[2],e[5],e[8],0,0,0,0,1),this}extractBasis(t,e,n){return this.determinant()===0?(t.set(1,0,0),e.set(0,1,0),n.set(0,0,1),this):(t.setFromMatrixColumn(this,0),e.setFromMatrixColumn(this,1),n.setFromMatrixColumn(this,2),this)}makeBasis(t,e,n){return this.set(t.x,e.x,n.x,0,t.y,e.y,n.y,0,t.z,e.z,n.z,0,0,0,0,1),this}extractRotation(t){if(t.determinant()===0)return this.identity();let e=this.elements,n=t.elements,s=1/Qi.setFromMatrixColumn(t,0).length(),r=1/Qi.setFromMatrixColumn(t,1).length(),a=1/Qi.setFromMatrixColumn(t,2).length();return e[0]=n[0]*s,e[1]=n[1]*s,e[2]=n[2]*s,e[3]=0,e[4]=n[4]*r,e[5]=n[5]*r,e[6]=n[6]*r,e[7]=0,e[8]=n[8]*a,e[9]=n[9]*a,e[10]=n[10]*a,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromEuler(t){let e=this.elements,n=t.x,s=t.y,r=t.z,a=Math.cos(n),o=Math.sin(n),l=Math.cos(s),c=Math.sin(s),d=Math.cos(r),u=Math.sin(r);if(t.order==="XYZ"){let h=a*d,p=a*u,_=o*d,y=o*u;e[0]=l*d,e[4]=-l*u,e[8]=c,e[1]=p+_*c,e[5]=h-y*c,e[9]=-o*l,e[2]=y-h*c,e[6]=_+p*c,e[10]=a*l}else if(t.order==="YXZ"){let h=l*d,p=l*u,_=c*d,y=c*u;e[0]=h+y*o,e[4]=_*o-p,e[8]=a*c,e[1]=a*u,e[5]=a*d,e[9]=-o,e[2]=p*o-_,e[6]=y+h*o,e[10]=a*l}else if(t.order==="ZXY"){let h=l*d,p=l*u,_=c*d,y=c*u;e[0]=h-y*o,e[4]=-a*u,e[8]=_+p*o,e[1]=p+_*o,e[5]=a*d,e[9]=y-h*o,e[2]=-a*c,e[6]=o,e[10]=a*l}else if(t.order==="ZYX"){let h=a*d,p=a*u,_=o*d,y=o*u;e[0]=l*d,e[4]=_*c-p,e[8]=h*c+y,e[1]=l*u,e[5]=y*c+h,e[9]=p*c-_,e[2]=-c,e[6]=o*l,e[10]=a*l}else if(t.order==="YZX"){let h=a*l,p=a*c,_=o*l,y=o*c;e[0]=l*d,e[4]=y-h*u,e[8]=_*u+p,e[1]=u,e[5]=a*d,e[9]=-o*d,e[2]=-c*d,e[6]=p*u+_,e[10]=h-y*u}else if(t.order==="XZY"){let h=a*l,p=a*c,_=o*l,y=o*c;e[0]=l*d,e[4]=-u,e[8]=c*d,e[1]=h*u+y,e[5]=a*d,e[9]=p*u-_,e[2]=_*u-p,e[6]=o*d,e[10]=y*u+h}return e[3]=0,e[7]=0,e[11]=0,e[12]=0,e[13]=0,e[14]=0,e[15]=1,this}makeRotationFromQuaternion(t){return this.compose(Au,t,Eu)}lookAt(t,e,n){let s=this.elements;return qe.subVectors(t,e),qe.lengthSq()===0&&(qe.z=1),qe.normalize(),ri.crossVectors(n,qe),ri.lengthSq()===0&&(Math.abs(n.z)===1?qe.x+=1e-4:qe.z+=1e-4,qe.normalize(),ri.crossVectors(n,qe)),ri.normalize(),Ur.crossVectors(qe,ri),s[0]=ri.x,s[4]=Ur.x,s[8]=qe.x,s[1]=ri.y,s[5]=Ur.y,s[9]=qe.y,s[2]=ri.z,s[6]=Ur.z,s[10]=qe.z,this}multiply(t){return this.multiplyMatrices(this,t)}premultiply(t){return this.multiplyMatrices(t,this)}multiplyMatrices(t,e){let n=t.elements,s=e.elements,r=this.elements,a=n[0],o=n[4],l=n[8],c=n[12],d=n[1],u=n[5],h=n[9],p=n[13],_=n[2],y=n[6],m=n[10],f=n[14],S=n[3],M=n[7],E=n[11],R=n[15],w=s[0],I=s[4],v=s[8],A=s[12],D=s[1],T=s[5],O=s[9],H=s[13],$=s[2],U=s[6],L=s[10],B=s[14],K=s[3],tt=s[7],ct=s[11],bt=s[15];return r[0]=a*w+o*D+l*$+c*K,r[4]=a*I+o*T+l*U+c*tt,r[8]=a*v+o*O+l*L+c*ct,r[12]=a*A+o*H+l*B+c*bt,r[1]=d*w+u*D+h*$+p*K,r[5]=d*I+u*T+h*U+p*tt,r[9]=d*v+u*O+h*L+p*ct,r[13]=d*A+u*H+h*B+p*bt,r[2]=_*w+y*D+m*$+f*K,r[6]=_*I+y*T+m*U+f*tt,r[10]=_*v+y*O+m*L+f*ct,r[14]=_*A+y*H+m*B+f*bt,r[3]=S*w+M*D+E*$+R*K,r[7]=S*I+M*T+E*U+R*tt,r[11]=S*v+M*O+E*L+R*ct,r[15]=S*A+M*H+E*B+R*bt,this}multiplyScalar(t){let e=this.elements;return e[0]*=t,e[4]*=t,e[8]*=t,e[12]*=t,e[1]*=t,e[5]*=t,e[9]*=t,e[13]*=t,e[2]*=t,e[6]*=t,e[10]*=t,e[14]*=t,e[3]*=t,e[7]*=t,e[11]*=t,e[15]*=t,this}determinant(){let t=this.elements,e=t[0],n=t[4],s=t[8],r=t[12],a=t[1],o=t[5],l=t[9],c=t[13],d=t[2],u=t[6],h=t[10],p=t[14],_=t[3],y=t[7],m=t[11],f=t[15],S=l*p-c*h,M=o*p-c*u,E=o*h-l*u,R=a*p-c*d,w=a*h-l*d,I=a*u-o*d;return e*(y*S-m*M+f*E)-n*(_*S-m*R+f*w)+s*(_*M-y*R+f*I)-r*(_*E-y*w+m*I)}transpose(){let t=this.elements,e;return e=t[1],t[1]=t[4],t[4]=e,e=t[2],t[2]=t[8],t[8]=e,e=t[6],t[6]=t[9],t[9]=e,e=t[3],t[3]=t[12],t[12]=e,e=t[7],t[7]=t[13],t[13]=e,e=t[11],t[11]=t[14],t[14]=e,this}setPosition(t,e,n){let s=this.elements;return t.isVector3?(s[12]=t.x,s[13]=t.y,s[14]=t.z):(s[12]=t,s[13]=e,s[14]=n),this}invert(){let t=this.elements,e=t[0],n=t[1],s=t[2],r=t[3],a=t[4],o=t[5],l=t[6],c=t[7],d=t[8],u=t[9],h=t[10],p=t[11],_=t[12],y=t[13],m=t[14],f=t[15],S=e*o-n*a,M=e*l-s*a,E=e*c-r*a,R=n*l-s*o,w=n*c-r*o,I=s*c-r*l,v=d*y-u*_,A=d*m-h*_,D=d*f-p*_,T=u*m-h*y,O=u*f-p*y,H=h*f-p*m,$=S*H-M*O+E*T+R*D-w*A+I*v;if($===0)return this.set(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0);let U=1/$;return t[0]=(o*H-l*O+c*T)*U,t[1]=(s*O-n*H-r*T)*U,t[2]=(y*I-m*w+f*R)*U,t[3]=(h*w-u*I-p*R)*U,t[4]=(l*D-a*H-c*A)*U,t[5]=(e*H-s*D+r*A)*U,t[6]=(m*E-_*I-f*M)*U,t[7]=(d*I-h*E+p*M)*U,t[8]=(a*O-o*D+c*v)*U,t[9]=(n*D-e*O-r*v)*U,t[10]=(_*w-y*E+f*S)*U,t[11]=(u*E-d*w-p*S)*U,t[12]=(o*A-a*T-l*v)*U,t[13]=(e*T-n*A+s*v)*U,t[14]=(y*M-_*R-m*S)*U,t[15]=(d*R-u*M+h*S)*U,this}scale(t){let e=this.elements,n=t.x,s=t.y,r=t.z;return e[0]*=n,e[4]*=s,e[8]*=r,e[1]*=n,e[5]*=s,e[9]*=r,e[2]*=n,e[6]*=s,e[10]*=r,e[3]*=n,e[7]*=s,e[11]*=r,this}getMaxScaleOnAxis(){let t=this.elements,e=t[0]*t[0]+t[1]*t[1]+t[2]*t[2],n=t[4]*t[4]+t[5]*t[5]+t[6]*t[6],s=t[8]*t[8]+t[9]*t[9]+t[10]*t[10];return Math.sqrt(Math.max(e,n,s))}makeTranslation(t,e,n){return t.isVector3?this.set(1,0,0,t.x,0,1,0,t.y,0,0,1,t.z,0,0,0,1):this.set(1,0,0,t,0,1,0,e,0,0,1,n,0,0,0,1),this}makeRotationX(t){let e=Math.cos(t),n=Math.sin(t);return this.set(1,0,0,0,0,e,-n,0,0,n,e,0,0,0,0,1),this}makeRotationY(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,0,n,0,0,1,0,0,-n,0,e,0,0,0,0,1),this}makeRotationZ(t){let e=Math.cos(t),n=Math.sin(t);return this.set(e,-n,0,0,n,e,0,0,0,0,1,0,0,0,0,1),this}makeRotationAxis(t,e){let n=Math.cos(e),s=Math.sin(e),r=1-n,a=t.x,o=t.y,l=t.z,c=r*a,d=r*o;return this.set(c*a+n,c*o-s*l,c*l+s*o,0,c*o+s*l,d*o+n,d*l-s*a,0,c*l-s*o,d*l+s*a,r*l*l+n,0,0,0,0,1),this}makeScale(t,e,n){return this.set(t,0,0,0,0,e,0,0,0,0,n,0,0,0,0,1),this}makeShear(t,e,n,s,r,a){return this.set(1,n,r,0,t,1,a,0,e,s,1,0,0,0,0,1),this}compose(t,e,n){let s=this.elements,r=e._x,a=e._y,o=e._z,l=e._w,c=r+r,d=a+a,u=o+o,h=r*c,p=r*d,_=r*u,y=a*d,m=a*u,f=o*u,S=l*c,M=l*d,E=l*u,R=n.x,w=n.y,I=n.z;return s[0]=(1-(y+f))*R,s[1]=(p+E)*R,s[2]=(_-M)*R,s[3]=0,s[4]=(p-E)*w,s[5]=(1-(h+f))*w,s[6]=(m+S)*w,s[7]=0,s[8]=(_+M)*I,s[9]=(m-S)*I,s[10]=(1-(h+y))*I,s[11]=0,s[12]=t.x,s[13]=t.y,s[14]=t.z,s[15]=1,this}decompose(t,e,n){let s=this.elements;t.x=s[12],t.y=s[13],t.z=s[14];let r=this.determinant();if(r===0)return n.set(1,1,1),e.identity(),this;let a=Qi.set(s[0],s[1],s[2]).length(),o=Qi.set(s[4],s[5],s[6]).length(),l=Qi.set(s[8],s[9],s[10]).length();r<0&&(a=-a),mn.copy(this);let c=1/a,d=1/o,u=1/l;return mn.elements[0]*=c,mn.elements[1]*=c,mn.elements[2]*=c,mn.elements[4]*=d,mn.elements[5]*=d,mn.elements[6]*=d,mn.elements[8]*=u,mn.elements[9]*=u,mn.elements[10]*=u,e.setFromRotationMatrix(mn),n.x=a,n.y=o,n.z=l,this}makePerspective(t,e,n,s,r,a,o=vn,l=!1){let c=this.elements,d=2*r/(e-t),u=2*r/(n-s),h=(e+t)/(e-t),p=(n+s)/(n-s),_,y;if(l)_=r/(a-r),y=a*r/(a-r);else if(o===vn)_=-(a+r)/(a-r),y=-2*a*r/(a-r);else if(o===ds)_=-a/(a-r),y=-a*r/(a-r);else throw new Error("THREE.Matrix4.makePerspective(): Invalid coordinate system: "+o);return c[0]=d,c[4]=0,c[8]=h,c[12]=0,c[1]=0,c[5]=u,c[9]=p,c[13]=0,c[2]=0,c[6]=0,c[10]=_,c[14]=y,c[3]=0,c[7]=0,c[11]=-1,c[15]=0,this}makeOrthographic(t,e,n,s,r,a,o=vn,l=!1){let c=this.elements,d=2/(e-t),u=2/(n-s),h=-(e+t)/(e-t),p=-(n+s)/(n-s),_,y;if(l)_=1/(a-r),y=a/(a-r);else if(o===vn)_=-2/(a-r),y=-(a+r)/(a-r);else if(o===ds)_=-1/(a-r),y=-r/(a-r);else throw new Error("THREE.Matrix4.makeOrthographic(): Invalid coordinate system: "+o);return c[0]=d,c[4]=0,c[8]=0,c[12]=h,c[1]=0,c[5]=u,c[9]=0,c[13]=p,c[2]=0,c[6]=0,c[10]=_,c[14]=y,c[3]=0,c[7]=0,c[11]=0,c[15]=1,this}equals(t){let e=this.elements,n=t.elements;for(let s=0;s<16;s++)if(e[s]!==n[s])return!1;return!0}fromArray(t,e=0){for(let n=0;n<16;n++)this.elements[n]=t[n+e];return this}toArray(t=[],e=0){let n=this.elements;return t[e]=n[0],t[e+1]=n[1],t[e+2]=n[2],t[e+3]=n[3],t[e+4]=n[4],t[e+5]=n[5],t[e+6]=n[6],t[e+7]=n[7],t[e+8]=n[8],t[e+9]=n[9],t[e+10]=n[10],t[e+11]=n[11],t[e+12]=n[12],t[e+13]=n[13],t[e+14]=n[14],t[e+15]=n[15],t}},Qi=new k,mn=new fe,Au=new k(0,0,0),Eu=new k(1,1,1),ri=new k,Ur=new k,qe=new k,dh=new fe,uh=new Nn,Jn=class i{constructor(t=0,e=0,n=0,s=i.DEFAULT_ORDER){this.isEuler=!0,this._x=t,this._y=e,this._z=n,this._order=s}get x(){return this._x}set x(t){this._x=t,this._onChangeCallback()}get y(){return this._y}set y(t){this._y=t,this._onChangeCallback()}get z(){return this._z}set z(t){this._z=t,this._onChangeCallback()}get order(){return this._order}set order(t){this._order=t,this._onChangeCallback()}set(t,e,n,s=this._order){return this._x=t,this._y=e,this._z=n,this._order=s,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._order)}copy(t){return this._x=t._x,this._y=t._y,this._z=t._z,this._order=t._order,this._onChangeCallback(),this}setFromRotationMatrix(t,e=this._order,n=!0){let s=t.elements,r=s[0],a=s[4],o=s[8],l=s[1],c=s[5],d=s[9],u=s[2],h=s[6],p=s[10];switch(e){case"XYZ":this._y=Math.asin(Xt(o,-1,1)),Math.abs(o)<.9999999?(this._x=Math.atan2(-d,p),this._z=Math.atan2(-a,r)):(this._x=Math.atan2(h,c),this._z=0);break;case"YXZ":this._x=Math.asin(-Xt(d,-1,1)),Math.abs(d)<.9999999?(this._y=Math.atan2(o,p),this._z=Math.atan2(l,c)):(this._y=Math.atan2(-u,r),this._z=0);break;case"ZXY":this._x=Math.asin(Xt(h,-1,1)),Math.abs(h)<.9999999?(this._y=Math.atan2(-u,p),this._z=Math.atan2(-a,c)):(this._y=0,this._z=Math.atan2(l,r));break;case"ZYX":this._y=Math.asin(-Xt(u,-1,1)),Math.abs(u)<.9999999?(this._x=Math.atan2(h,p),this._z=Math.atan2(l,r)):(this._x=0,this._z=Math.atan2(-a,c));break;case"YZX":this._z=Math.asin(Xt(l,-1,1)),Math.abs(l)<.9999999?(this._x=Math.atan2(-d,c),this._y=Math.atan2(-u,r)):(this._x=0,this._y=Math.atan2(o,p));break;case"XZY":this._z=Math.asin(-Xt(a,-1,1)),Math.abs(a)<.9999999?(this._x=Math.atan2(h,c),this._y=Math.atan2(o,r)):(this._x=Math.atan2(-d,p),this._y=0);break;default:Et("Euler: .setFromRotationMatrix() encountered an unknown order: "+e)}return this._order=e,n===!0&&this._onChangeCallback(),this}setFromQuaternion(t,e,n){return dh.makeRotationFromQuaternion(t),this.setFromRotationMatrix(dh,e,n)}setFromVector3(t,e=this._order){return this.set(t.x,t.y,t.z,e)}reorder(t){return uh.setFromEuler(this),this.setFromQuaternion(uh,t)}equals(t){return t._x===this._x&&t._y===this._y&&t._z===this._z&&t._order===this._order}fromArray(t){return this._x=t[0],this._y=t[1],this._z=t[2],t[3]!==void 0&&(this._order=t[3]),this._onChangeCallback(),this}toArray(t=[],e=0){return t[e]=this._x,t[e+1]=this._y,t[e+2]=this._z,t[e+3]=this._order,t}_onChange(t){return this._onChangeCallback=t,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._order}};Jn.DEFAULT_ORDER="XYZ";var Xs=class{constructor(){this.mask=1}set(t){this.mask=(1<<t|0)>>>0}enable(t){this.mask|=1<<t|0}enableAll(){this.mask=-1}toggle(t){this.mask^=1<<t|0}disable(t){this.mask&=~(1<<t|0)}disableAll(){this.mask=0}test(t){return(this.mask&t.mask)!==0}isEnabled(t){return(this.mask&(1<<t|0))!==0}},Tu=0,fh=new k,ts=new Nn,Hn=new fe,Fr=new k,Fs=new k,Cu=new k,Ru=new Nn,ph=new k(1,0,0),mh=new k(0,1,0),gh=new k(0,0,1),_h={type:"added"},Iu={type:"removed"},es={type:"childadded",child:null},pl={type:"childremoved",child:null},Oe=class i extends Dn{constructor(){super(),this.isObject3D=!0,Object.defineProperty(this,"id",{value:Tu++}),this.uuid=gr(),this.name="",this.type="Object3D",this.parent=null,this.children=[],this.up=i.DEFAULT_UP.clone();let t=new k,e=new Jn,n=new Nn,s=new k(1,1,1);function r(){n.setFromEuler(e,!1)}function a(){e.setFromQuaternion(n,void 0,!1)}e._onChange(r),n._onChange(a),Object.defineProperties(this,{position:{configurable:!0,enumerable:!0,value:t},rotation:{configurable:!0,enumerable:!0,value:e},quaternion:{configurable:!0,enumerable:!0,value:n},scale:{configurable:!0,enumerable:!0,value:s},modelViewMatrix:{value:new fe},normalMatrix:{value:new Pt}}),this.matrix=new fe,this.matrixWorld=new fe,this.matrixAutoUpdate=i.DEFAULT_MATRIX_AUTO_UPDATE,this.matrixWorldAutoUpdate=i.DEFAULT_MATRIX_WORLD_AUTO_UPDATE,this.matrixWorldNeedsUpdate=!1,this.layers=new Xs,this.visible=!0,this.castShadow=!1,this.receiveShadow=!1,this.frustumCulled=!0,this.renderOrder=0,this.animations=[],this.customDepthMaterial=void 0,this.customDistanceMaterial=void 0,this.static=!1,this.userData={},this.pivot=null}onBeforeShadow(){}onAfterShadow(){}onBeforeRender(){}onAfterRender(){}applyMatrix4(t){this.matrixAutoUpdate&&this.updateMatrix(),this.matrix.premultiply(t),this.matrix.decompose(this.position,this.quaternion,this.scale)}applyQuaternion(t){return this.quaternion.premultiply(t),this}setRotationFromAxisAngle(t,e){this.quaternion.setFromAxisAngle(t,e)}setRotationFromEuler(t){this.quaternion.setFromEuler(t,!0)}setRotationFromMatrix(t){this.quaternion.setFromRotationMatrix(t)}setRotationFromQuaternion(t){this.quaternion.copy(t)}rotateOnAxis(t,e){return ts.setFromAxisAngle(t,e),this.quaternion.multiply(ts),this}rotateOnWorldAxis(t,e){return ts.setFromAxisAngle(t,e),this.quaternion.premultiply(ts),this}rotateX(t){return this.rotateOnAxis(ph,t)}rotateY(t){return this.rotateOnAxis(mh,t)}rotateZ(t){return this.rotateOnAxis(gh,t)}translateOnAxis(t,e){return fh.copy(t).applyQuaternion(this.quaternion),this.position.add(fh.multiplyScalar(e)),this}translateX(t){return this.translateOnAxis(ph,t)}translateY(t){return this.translateOnAxis(mh,t)}translateZ(t){return this.translateOnAxis(gh,t)}localToWorld(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(this.matrixWorld)}worldToLocal(t){return this.updateWorldMatrix(!0,!1),t.applyMatrix4(Hn.copy(this.matrixWorld).invert())}lookAt(t,e,n){t.isVector3?Fr.copy(t):Fr.set(t,e,n);let s=this.parent;this.updateWorldMatrix(!0,!1),Fs.setFromMatrixPosition(this.matrixWorld),this.isCamera||this.isLight?Hn.lookAt(Fs,Fr,this.up):Hn.lookAt(Fr,Fs,this.up),this.quaternion.setFromRotationMatrix(Hn),s&&(Hn.extractRotation(s.matrixWorld),ts.setFromRotationMatrix(Hn),this.quaternion.premultiply(ts.invert()))}add(t){if(arguments.length>1){for(let e=0;e<arguments.length;e++)this.add(arguments[e]);return this}return t===this?(Ct("Object3D.add: object can't be added as a child of itself.",t),this):(t&&t.isObject3D?(t.removeFromParent(),t.parent=this,this.children.push(t),t.dispatchEvent(_h),es.child=t,this.dispatchEvent(es),es.child=null):Ct("Object3D.add: object not an instance of THREE.Object3D.",t),this)}remove(t){if(arguments.length>1){for(let n=0;n<arguments.length;n++)this.remove(arguments[n]);return this}let e=this.children.indexOf(t);return e!==-1&&(t.parent=null,this.children.splice(e,1),t.dispatchEvent(Iu),pl.child=t,this.dispatchEvent(pl),pl.child=null),this}removeFromParent(){let t=this.parent;return t!==null&&t.remove(this),this}clear(){return this.remove(...this.children)}attach(t){return this.updateWorldMatrix(!0,!1),Hn.copy(this.matrixWorld).invert(),t.parent!==null&&(t.parent.updateWorldMatrix(!0,!1),Hn.multiply(t.parent.matrixWorld)),t.applyMatrix4(Hn),t.removeFromParent(),t.parent=this,this.children.push(t),t.updateWorldMatrix(!1,!0),t.dispatchEvent(_h),es.child=t,this.dispatchEvent(es),es.child=null,this}getObjectById(t){return this.getObjectByProperty("id",t)}getObjectByName(t){return this.getObjectByProperty("name",t)}getObjectByProperty(t,e){if(this[t]===e)return this;for(let n=0,s=this.children.length;n<s;n++){let a=this.children[n].getObjectByProperty(t,e);if(a!==void 0)return a}}getObjectsByProperty(t,e,n=[]){this[t]===e&&n.push(this);let s=this.children;for(let r=0,a=s.length;r<a;r++)s[r].getObjectsByProperty(t,e,n);return n}getWorldPosition(t){return this.updateWorldMatrix(!0,!1),t.setFromMatrixPosition(this.matrixWorld)}getWorldQuaternion(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Fs,t,Cu),t}getWorldScale(t){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Fs,Ru,t),t}getWorldDirection(t){this.updateWorldMatrix(!0,!1);let e=this.matrixWorld.elements;return t.set(e[8],e[9],e[10]).normalize()}raycast(){}traverse(t){t(this);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].traverse(t)}traverseVisible(t){if(this.visible===!1)return;t(this);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].traverseVisible(t)}traverseAncestors(t){let e=this.parent;e!==null&&(t(e),e.traverseAncestors(t))}updateMatrix(){this.matrix.compose(this.position,this.quaternion,this.scale);let t=this.pivot;if(t!==null){let e=t.x,n=t.y,s=t.z,r=this.matrix.elements;r[12]+=e-r[0]*e-r[4]*n-r[8]*s,r[13]+=n-r[1]*e-r[5]*n-r[9]*s,r[14]+=s-r[2]*e-r[6]*n-r[10]*s}this.matrixWorldNeedsUpdate=!0}updateMatrixWorld(t){this.matrixAutoUpdate&&this.updateMatrix(),(this.matrixWorldNeedsUpdate||t)&&(this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),this.matrixWorldNeedsUpdate=!1,t=!0);let e=this.children;for(let n=0,s=e.length;n<s;n++)e[n].updateMatrixWorld(t)}updateWorldMatrix(t,e){let n=this.parent;if(t===!0&&n!==null&&n.updateWorldMatrix(!0,!1),this.matrixAutoUpdate&&this.updateMatrix(),this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),e===!0){let s=this.children;for(let r=0,a=s.length;r<a;r++)s[r].updateWorldMatrix(!1,!0)}}toJSON(t){let e=t===void 0||typeof t=="string",n={};e&&(t={geometries:{},materials:{},textures:{},images:{},shapes:{},skeletons:{},animations:{},nodes:{}},n.metadata={version:4.7,type:"Object",generator:"Object3D.toJSON"});let s={};s.uuid=this.uuid,s.type=this.type,this.name!==""&&(s.name=this.name),this.castShadow===!0&&(s.castShadow=!0),this.receiveShadow===!0&&(s.receiveShadow=!0),this.visible===!1&&(s.visible=!1),this.frustumCulled===!1&&(s.frustumCulled=!1),this.renderOrder!==0&&(s.renderOrder=this.renderOrder),this.static!==!1&&(s.static=this.static),Object.keys(this.userData).length>0&&(s.userData=this.userData),s.layers=this.layers.mask,s.matrix=this.matrix.toArray(),s.up=this.up.toArray(),this.pivot!==null&&(s.pivot=this.pivot.toArray()),this.matrixAutoUpdate===!1&&(s.matrixAutoUpdate=!1),this.morphTargetDictionary!==void 0&&(s.morphTargetDictionary=Object.assign({},this.morphTargetDictionary)),this.morphTargetInfluences!==void 0&&(s.morphTargetInfluences=this.morphTargetInfluences.slice()),this.isInstancedMesh&&(s.type="InstancedMesh",s.count=this.count,s.instanceMatrix=this.instanceMatrix.toJSON(),this.instanceColor!==null&&(s.instanceColor=this.instanceColor.toJSON())),this.isBatchedMesh&&(s.type="BatchedMesh",s.perObjectFrustumCulled=this.perObjectFrustumCulled,s.sortObjects=this.sortObjects,s.drawRanges=this._drawRanges,s.reservedRanges=this._reservedRanges,s.geometryInfo=this._geometryInfo.map(o=>({...o,boundingBox:o.boundingBox?o.boundingBox.toJSON():void 0,boundingSphere:o.boundingSphere?o.boundingSphere.toJSON():void 0})),s.instanceInfo=this._instanceInfo.map(o=>({...o})),s.availableInstanceIds=this._availableInstanceIds.slice(),s.availableGeometryIds=this._availableGeometryIds.slice(),s.nextIndexStart=this._nextIndexStart,s.nextVertexStart=this._nextVertexStart,s.geometryCount=this._geometryCount,s.maxInstanceCount=this._maxInstanceCount,s.maxVertexCount=this._maxVertexCount,s.maxIndexCount=this._maxIndexCount,s.geometryInitialized=this._geometryInitialized,s.matricesTexture=this._matricesTexture.toJSON(t),s.indirectTexture=this._indirectTexture.toJSON(t),this._colorsTexture!==null&&(s.colorsTexture=this._colorsTexture.toJSON(t)),this.boundingSphere!==null&&(s.boundingSphere=this.boundingSphere.toJSON()),this.boundingBox!==null&&(s.boundingBox=this.boundingBox.toJSON()));function r(o,l){return o[l.uuid]===void 0&&(o[l.uuid]=l.toJSON(t)),l.uuid}if(this.isScene)this.background&&(this.background.isColor?s.background=this.background.toJSON():this.background.isTexture&&(s.background=this.background.toJSON(t).uuid)),this.environment&&this.environment.isTexture&&this.environment.isRenderTargetTexture!==!0&&(s.environment=this.environment.toJSON(t).uuid);else if(this.isMesh||this.isLine||this.isPoints){s.geometry=r(t.geometries,this.geometry);let o=this.geometry.parameters;if(o!==void 0&&o.shapes!==void 0){let l=o.shapes;if(Array.isArray(l))for(let c=0,d=l.length;c<d;c++){let u=l[c];r(t.shapes,u)}else r(t.shapes,l)}}if(this.isSkinnedMesh&&(s.bindMode=this.bindMode,s.bindMatrix=this.bindMatrix.toArray(),this.skeleton!==void 0&&(r(t.skeletons,this.skeleton),s.skeleton=this.skeleton.uuid)),this.material!==void 0)if(Array.isArray(this.material)){let o=[];for(let l=0,c=this.material.length;l<c;l++)o.push(r(t.materials,this.material[l]));s.material=o}else s.material=r(t.materials,this.material);if(this.children.length>0){s.children=[];for(let o=0;o<this.children.length;o++)s.children.push(this.children[o].toJSON(t).object)}if(this.animations.length>0){s.animations=[];for(let o=0;o<this.animations.length;o++){let l=this.animations[o];s.animations.push(r(t.animations,l))}}if(e){let o=a(t.geometries),l=a(t.materials),c=a(t.textures),d=a(t.images),u=a(t.shapes),h=a(t.skeletons),p=a(t.animations),_=a(t.nodes);o.length>0&&(n.geometries=o),l.length>0&&(n.materials=l),c.length>0&&(n.textures=c),d.length>0&&(n.images=d),u.length>0&&(n.shapes=u),h.length>0&&(n.skeletons=h),p.length>0&&(n.animations=p),_.length>0&&(n.nodes=_)}return n.object=s,n;function a(o){let l=[];for(let c in o){let d=o[c];delete d.metadata,l.push(d)}return l}}clone(t){return new this.constructor().copy(this,t)}copy(t,e=!0){if(this.name=t.name,this.up.copy(t.up),this.position.copy(t.position),this.rotation.order=t.rotation.order,this.quaternion.copy(t.quaternion),this.scale.copy(t.scale),this.pivot=t.pivot!==null?t.pivot.clone():null,this.matrix.copy(t.matrix),this.matrixWorld.copy(t.matrixWorld),this.matrixAutoUpdate=t.matrixAutoUpdate,this.matrixWorldAutoUpdate=t.matrixWorldAutoUpdate,this.matrixWorldNeedsUpdate=t.matrixWorldNeedsUpdate,this.layers.mask=t.layers.mask,this.visible=t.visible,this.castShadow=t.castShadow,this.receiveShadow=t.receiveShadow,this.frustumCulled=t.frustumCulled,this.renderOrder=t.renderOrder,this.static=t.static,this.animations=t.animations.slice(),this.userData=JSON.parse(JSON.stringify(t.userData)),e===!0)for(let n=0;n<t.children.length;n++){let s=t.children[n];this.add(s.clone())}return this}};Oe.DEFAULT_UP=new k(0,1,0);Oe.DEFAULT_MATRIX_AUTO_UPDATE=!0;Oe.DEFAULT_MATRIX_WORLD_AUTO_UPDATE=!0;var Bi=class extends Oe{constructor(){super(),this.isGroup=!0,this.type="Group"}},Pu={type:"move"},ps=class{constructor(){this._targetRay=null,this._grip=null,this._hand=null}getHandSpace(){return this._hand===null&&(this._hand=new Bi,this._hand.matrixAutoUpdate=!1,this._hand.visible=!1,this._hand.joints={},this._hand.inputState={pinching:!1}),this._hand}getTargetRaySpace(){return this._targetRay===null&&(this._targetRay=new Bi,this._targetRay.matrixAutoUpdate=!1,this._targetRay.visible=!1,this._targetRay.hasLinearVelocity=!1,this._targetRay.linearVelocity=new k,this._targetRay.hasAngularVelocity=!1,this._targetRay.angularVelocity=new k),this._targetRay}getGripSpace(){return this._grip===null&&(this._grip=new Bi,this._grip.matrixAutoUpdate=!1,this._grip.visible=!1,this._grip.hasLinearVelocity=!1,this._grip.linearVelocity=new k,this._grip.hasAngularVelocity=!1,this._grip.angularVelocity=new k,this._grip.eventsEnabled=!1),this._grip}dispatchEvent(t){return this._targetRay!==null&&this._targetRay.dispatchEvent(t),this._grip!==null&&this._grip.dispatchEvent(t),this._hand!==null&&this._hand.dispatchEvent(t),this}connect(t){if(t&&t.hand){let e=this._hand;if(e)for(let n of t.hand.values())this._getHandJoint(e,n)}return this.dispatchEvent({type:"connected",data:t}),this}disconnect(t){return this.dispatchEvent({type:"disconnected",data:t}),this._targetRay!==null&&(this._targetRay.visible=!1),this._grip!==null&&(this._grip.visible=!1),this._hand!==null&&(this._hand.visible=!1),this}update(t,e,n){let s=null,r=null,a=null,o=this._targetRay,l=this._grip,c=this._hand;if(t&&e.session.visibilityState!=="visible-blurred"){if(c&&t.hand){a=!0;for(let y of t.hand.values()){let m=e.getJointPose(y,n),f=this._getHandJoint(c,y);m!==null&&(f.matrix.fromArray(m.transform.matrix),f.matrix.decompose(f.position,f.rotation,f.scale),f.matrixWorldNeedsUpdate=!0,f.jointRadius=m.radius),f.visible=m!==null}let d=c.joints["index-finger-tip"],u=c.joints["thumb-tip"],h=d.position.distanceTo(u.position),p=.02,_=.005;c.inputState.pinching&&h>p+_?(c.inputState.pinching=!1,this.dispatchEvent({type:"pinchend",handedness:t.handedness,target:this})):!c.inputState.pinching&&h<=p-_&&(c.inputState.pinching=!0,this.dispatchEvent({type:"pinchstart",handedness:t.handedness,target:this}))}else l!==null&&t.gripSpace&&(r=e.getPose(t.gripSpace,n),r!==null&&(l.matrix.fromArray(r.transform.matrix),l.matrix.decompose(l.position,l.rotation,l.scale),l.matrixWorldNeedsUpdate=!0,r.linearVelocity?(l.hasLinearVelocity=!0,l.linearVelocity.copy(r.linearVelocity)):l.hasLinearVelocity=!1,r.angularVelocity?(l.hasAngularVelocity=!0,l.angularVelocity.copy(r.angularVelocity)):l.hasAngularVelocity=!1,l.eventsEnabled&&l.dispatchEvent({type:"gripUpdated",data:t,target:this})));o!==null&&(s=e.getPose(t.targetRaySpace,n),s===null&&r!==null&&(s=r),s!==null&&(o.matrix.fromArray(s.transform.matrix),o.matrix.decompose(o.position,o.rotation,o.scale),o.matrixWorldNeedsUpdate=!0,s.linearVelocity?(o.hasLinearVelocity=!0,o.linearVelocity.copy(s.linearVelocity)):o.hasLinearVelocity=!1,s.angularVelocity?(o.hasAngularVelocity=!0,o.angularVelocity.copy(s.angularVelocity)):o.hasAngularVelocity=!1,this.dispatchEvent(Pu)))}return o!==null&&(o.visible=s!==null),l!==null&&(l.visible=r!==null),c!==null&&(c.visible=a!==null),this}_getHandJoint(t,e){if(t.joints[e.jointName]===void 0){let n=new Bi;n.matrixAutoUpdate=!1,n.visible=!1,t.joints[e.jointName]=n,t.add(n)}return t.joints[e.jointName]}},fd={aliceblue:15792383,antiquewhite:16444375,aqua:65535,aquamarine:8388564,azure:15794175,beige:16119260,bisque:16770244,black:0,blanchedalmond:16772045,blue:255,blueviolet:9055202,brown:10824234,burlywood:14596231,cadetblue:6266528,chartreuse:8388352,chocolate:13789470,coral:16744272,cornflowerblue:6591981,cornsilk:16775388,crimson:14423100,cyan:65535,darkblue:139,darkcyan:35723,darkgoldenrod:12092939,darkgray:11119017,darkgreen:25600,darkgrey:11119017,darkkhaki:12433259,darkmagenta:9109643,darkolivegreen:5597999,darkorange:16747520,darkorchid:10040012,darkred:9109504,darksalmon:15308410,darkseagreen:9419919,darkslateblue:4734347,darkslategray:3100495,darkslategrey:3100495,darkturquoise:52945,darkviolet:9699539,deeppink:16716947,deepskyblue:49151,dimgray:6908265,dimgrey:6908265,dodgerblue:2003199,firebrick:11674146,floralwhite:16775920,forestgreen:2263842,fuchsia:16711935,gainsboro:14474460,ghostwhite:16316671,gold:16766720,goldenrod:14329120,gray:8421504,green:32768,greenyellow:11403055,grey:8421504,honeydew:15794160,hotpink:16738740,indianred:13458524,indigo:4915330,ivory:16777200,khaki:15787660,lavender:15132410,lavenderblush:16773365,lawngreen:8190976,lemonchiffon:16775885,lightblue:11393254,lightcoral:15761536,lightcyan:14745599,lightgoldenrodyellow:16448210,lightgray:13882323,lightgreen:9498256,lightgrey:13882323,lightpink:16758465,lightsalmon:16752762,lightseagreen:2142890,lightskyblue:8900346,lightslategray:7833753,lightslategrey:7833753,lightsteelblue:11584734,lightyellow:16777184,lime:65280,limegreen:3329330,linen:16445670,magenta:16711935,maroon:8388608,mediumaquamarine:6737322,mediumblue:205,mediumorchid:12211667,mediumpurple:9662683,mediumseagreen:3978097,mediumslateblue:8087790,mediumspringgreen:64154,mediumturquoise:4772300,mediumvioletred:13047173,midnightblue:1644912,mintcream:16121850,mistyrose:16770273,moccasin:16770229,navajowhite:16768685,navy:128,oldlace:16643558,olive:8421376,olivedrab:7048739,orange:16753920,orangered:16729344,orchid:14315734,palegoldenrod:15657130,palegreen:10025880,paleturquoise:11529966,palevioletred:14381203,papayawhip:16773077,peachpuff:16767673,peru:13468991,pink:16761035,plum:14524637,powderblue:11591910,purple:8388736,rebeccapurple:6697881,red:16711680,rosybrown:12357519,royalblue:4286945,saddlebrown:9127187,salmon:16416882,sandybrown:16032864,seagreen:3050327,seashell:16774638,sienna:10506797,silver:12632256,skyblue:8900331,slateblue:6970061,slategray:7372944,slategrey:7372944,snow:16775930,springgreen:65407,steelblue:4620980,tan:13808780,teal:32896,thistle:14204888,tomato:16737095,turquoise:4251856,violet:15631086,wheat:16113331,white:16777215,whitesmoke:16119285,yellow:16776960,yellowgreen:10145074},ai={h:0,s:0,l:0},Or={h:0,s:0,l:0};function ml(i,t,e){return e<0&&(e+=1),e>1&&(e-=1),e<1/6?i+(t-i)*6*e:e<1/2?t:e<2/3?i+(t-i)*6*(2/3-e):i}var zt=class{constructor(t,e,n){return this.isColor=!0,this.r=1,this.g=1,this.b=1,this.set(t,e,n)}set(t,e,n){if(e===void 0&&n===void 0){let s=t;s&&s.isColor?this.copy(s):typeof s=="number"?this.setHex(s):typeof s=="string"&&this.setStyle(s)}else this.setRGB(t,e,n);return this}setScalar(t){return this.r=t,this.g=t,this.b=t,this}setHex(t,e=Ze){return t=Math.floor(t),this.r=(t>>16&255)/255,this.g=(t>>8&255)/255,this.b=(t&255)/255,$t.colorSpaceToWorking(this,e),this}setRGB(t,e,n,s=$t.workingColorSpace){return this.r=t,this.g=e,this.b=n,$t.colorSpaceToWorking(this,s),this}setHSL(t,e,n,s=$t.workingColorSpace){if(t=bu(t,1),e=Xt(e,0,1),n=Xt(n,0,1),e===0)this.r=this.g=this.b=n;else{let r=n<=.5?n*(1+e):n+e-n*e,a=2*n-r;this.r=ml(a,r,t+1/3),this.g=ml(a,r,t),this.b=ml(a,r,t-1/3)}return $t.colorSpaceToWorking(this,s),this}setStyle(t,e=Ze){function n(r){r!==void 0&&parseFloat(r)<1&&Et("Color: Alpha component of "+t+" will be ignored.")}let s;if(s=/^(\w+)\(([^\)]*)\)/.exec(t)){let r,a=s[1],o=s[2];switch(a){case"rgb":case"rgba":if(r=/^\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return n(r[4]),this.setRGB(Math.min(255,parseInt(r[1],10))/255,Math.min(255,parseInt(r[2],10))/255,Math.min(255,parseInt(r[3],10))/255,e);if(r=/^\s*(\d+)\%\s*,\s*(\d+)\%\s*,\s*(\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return n(r[4]),this.setRGB(Math.min(100,parseInt(r[1],10))/100,Math.min(100,parseInt(r[2],10))/100,Math.min(100,parseInt(r[3],10))/100,e);break;case"hsl":case"hsla":if(r=/^\s*(\d*\.?\d+)\s*,\s*(\d*\.?\d+)\%\s*,\s*(\d*\.?\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(o))return n(r[4]),this.setHSL(parseFloat(r[1])/360,parseFloat(r[2])/100,parseFloat(r[3])/100,e);break;default:Et("Color: Unknown color model "+t)}}else if(s=/^\#([A-Fa-f\d]+)$/.exec(t)){let r=s[1],a=r.length;if(a===3)return this.setRGB(parseInt(r.charAt(0),16)/15,parseInt(r.charAt(1),16)/15,parseInt(r.charAt(2),16)/15,e);if(a===6)return this.setHex(parseInt(r,16),e);Et("Color: Invalid hex color "+t)}else if(t&&t.length>0)return this.setColorName(t,e);return this}setColorName(t,e=Ze){let n=fd[t.toLowerCase()];return n!==void 0?this.setHex(n,e):Et("Color: Unknown color "+t),this}clone(){return new this.constructor(this.r,this.g,this.b)}copy(t){return this.r=t.r,this.g=t.g,this.b=t.b,this}copySRGBToLinear(t){return this.r=Yn(t.r),this.g=Yn(t.g),this.b=Yn(t.b),this}copyLinearToSRGB(t){return this.r=hs(t.r),this.g=hs(t.g),this.b=hs(t.b),this}convertSRGBToLinear(){return this.copySRGBToLinear(this),this}convertLinearToSRGB(){return this.copyLinearToSRGB(this),this}getHex(t=Ze){return $t.workingToColorSpace(Ne.copy(this),t),Math.round(Xt(Ne.r*255,0,255))*65536+Math.round(Xt(Ne.g*255,0,255))*256+Math.round(Xt(Ne.b*255,0,255))}getHexString(t=Ze){return("000000"+this.getHex(t).toString(16)).slice(-6)}getHSL(t,e=$t.workingColorSpace){$t.workingToColorSpace(Ne.copy(this),e);let n=Ne.r,s=Ne.g,r=Ne.b,a=Math.max(n,s,r),o=Math.min(n,s,r),l,c,d=(o+a)/2;if(o===a)l=0,c=0;else{let u=a-o;switch(c=d<=.5?u/(a+o):u/(2-a-o),a){case n:l=(s-r)/u+(s<r?6:0);break;case s:l=(r-n)/u+2;break;case r:l=(n-s)/u+4;break}l/=6}return t.h=l,t.s=c,t.l=d,t}getRGB(t,e=$t.workingColorSpace){return $t.workingToColorSpace(Ne.copy(this),e),t.r=Ne.r,t.g=Ne.g,t.b=Ne.b,t}getStyle(t=Ze){$t.workingToColorSpace(Ne.copy(this),t);let e=Ne.r,n=Ne.g,s=Ne.b;return t!==Ze?`color(${t} ${e.toFixed(3)} ${n.toFixed(3)} ${s.toFixed(3)})`:`rgb(${Math.round(e*255)},${Math.round(n*255)},${Math.round(s*255)})`}offsetHSL(t,e,n){return this.getHSL(ai),this.setHSL(ai.h+t,ai.s+e,ai.l+n)}add(t){return this.r+=t.r,this.g+=t.g,this.b+=t.b,this}addColors(t,e){return this.r=t.r+e.r,this.g=t.g+e.g,this.b=t.b+e.b,this}addScalar(t){return this.r+=t,this.g+=t,this.b+=t,this}sub(t){return this.r=Math.max(0,this.r-t.r),this.g=Math.max(0,this.g-t.g),this.b=Math.max(0,this.b-t.b),this}multiply(t){return this.r*=t.r,this.g*=t.g,this.b*=t.b,this}multiplyScalar(t){return this.r*=t,this.g*=t,this.b*=t,this}lerp(t,e){return this.r+=(t.r-this.r)*e,this.g+=(t.g-this.g)*e,this.b+=(t.b-this.b)*e,this}lerpColors(t,e,n){return this.r=t.r+(e.r-t.r)*n,this.g=t.g+(e.g-t.g)*n,this.b=t.b+(e.b-t.b)*n,this}lerpHSL(t,e){this.getHSL(ai),t.getHSL(Or);let n=cl(ai.h,Or.h,e),s=cl(ai.s,Or.s,e),r=cl(ai.l,Or.l,e);return this.setHSL(n,s,r),this}setFromVector3(t){return this.r=t.x,this.g=t.y,this.b=t.z,this}applyMatrix3(t){let e=this.r,n=this.g,s=this.b,r=t.elements;return this.r=r[0]*e+r[3]*n+r[6]*s,this.g=r[1]*e+r[4]*n+r[7]*s,this.b=r[2]*e+r[5]*n+r[8]*s,this}equals(t){return t.r===this.r&&t.g===this.g&&t.b===this.b}fromArray(t,e=0){return this.r=t[e],this.g=t[e+1],this.b=t[e+2],this}toArray(t=[],e=0){return t[e]=this.r,t[e+1]=this.g,t[e+2]=this.b,t}fromBufferAttribute(t,e){return this.r=t.getX(e),this.g=t.getY(e),this.b=t.getZ(e),this}toJSON(){return this.getHex()}*[Symbol.iterator](){yield this.r,yield this.g,yield this.b}},Ne=new zt;zt.NAMES=fd;var qs=class extends Oe{constructor(){super(),this.isScene=!0,this.type="Scene",this.background=null,this.environment=null,this.fog=null,this.backgroundBlurriness=0,this.backgroundIntensity=1,this.backgroundRotation=new Jn,this.environmentIntensity=1,this.environmentRotation=new Jn,this.overrideMaterial=null,typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}copy(t,e){return super.copy(t,e),t.background!==null&&(this.background=t.background.clone()),t.environment!==null&&(this.environment=t.environment.clone()),t.fog!==null&&(this.fog=t.fog.clone()),this.backgroundBlurriness=t.backgroundBlurriness,this.backgroundIntensity=t.backgroundIntensity,this.backgroundRotation.copy(t.backgroundRotation),this.environmentIntensity=t.environmentIntensity,this.environmentRotation.copy(t.environmentRotation),t.overrideMaterial!==null&&(this.overrideMaterial=t.overrideMaterial.clone()),this.matrixAutoUpdate=t.matrixAutoUpdate,this}toJSON(t){let e=super.toJSON(t);return this.fog!==null&&(e.object.fog=this.fog.toJSON()),this.backgroundBlurriness>0&&(e.object.backgroundBlurriness=this.backgroundBlurriness),this.backgroundIntensity!==1&&(e.object.backgroundIntensity=this.backgroundIntensity),e.object.backgroundRotation=this.backgroundRotation.toArray(),this.environmentIntensity!==1&&(e.object.environmentIntensity=this.environmentIntensity),e.object.environmentRotation=this.environmentRotation.toArray(),e}},gn=new k,Wn=new k,gl=new k,$n=new k,ns=new k,is=new k,vh=new k,_l=new k,vl=new k,xl=new k,yl=new pe,bl=new pe,Sl=new pe,di=class i{constructor(t=new k,e=new k,n=new k){this.a=t,this.b=e,this.c=n}static getNormal(t,e,n,s){s.subVectors(n,e),gn.subVectors(t,e),s.cross(gn);let r=s.lengthSq();return r>0?s.multiplyScalar(1/Math.sqrt(r)):s.set(0,0,0)}static getBarycoord(t,e,n,s,r){gn.subVectors(s,e),Wn.subVectors(n,e),gl.subVectors(t,e);let a=gn.dot(gn),o=gn.dot(Wn),l=gn.dot(gl),c=Wn.dot(Wn),d=Wn.dot(gl),u=a*c-o*o;if(u===0)return r.set(0,0,0),null;let h=1/u,p=(c*l-o*d)*h,_=(a*d-o*l)*h;return r.set(1-p-_,_,p)}static containsPoint(t,e,n,s){return this.getBarycoord(t,e,n,s,$n)===null?!1:$n.x>=0&&$n.y>=0&&$n.x+$n.y<=1}static getInterpolation(t,e,n,s,r,a,o,l){return this.getBarycoord(t,e,n,s,$n)===null?(l.x=0,l.y=0,"z"in l&&(l.z=0),"w"in l&&(l.w=0),null):(l.setScalar(0),l.addScaledVector(r,$n.x),l.addScaledVector(a,$n.y),l.addScaledVector(o,$n.z),l)}static getInterpolatedAttribute(t,e,n,s,r,a){return yl.setScalar(0),bl.setScalar(0),Sl.setScalar(0),yl.fromBufferAttribute(t,e),bl.fromBufferAttribute(t,n),Sl.fromBufferAttribute(t,s),a.setScalar(0),a.addScaledVector(yl,r.x),a.addScaledVector(bl,r.y),a.addScaledVector(Sl,r.z),a}static isFrontFacing(t,e,n,s){return gn.subVectors(n,e),Wn.subVectors(t,e),gn.cross(Wn).dot(s)<0}set(t,e,n){return this.a.copy(t),this.b.copy(e),this.c.copy(n),this}setFromPointsAndIndices(t,e,n,s){return this.a.copy(t[e]),this.b.copy(t[n]),this.c.copy(t[s]),this}setFromAttributeAndIndices(t,e,n,s){return this.a.fromBufferAttribute(t,e),this.b.fromBufferAttribute(t,n),this.c.fromBufferAttribute(t,s),this}clone(){return new this.constructor().copy(this)}copy(t){return this.a.copy(t.a),this.b.copy(t.b),this.c.copy(t.c),this}getArea(){return gn.subVectors(this.c,this.b),Wn.subVectors(this.a,this.b),gn.cross(Wn).length()*.5}getMidpoint(t){return t.addVectors(this.a,this.b).add(this.c).multiplyScalar(1/3)}getNormal(t){return i.getNormal(this.a,this.b,this.c,t)}getPlane(t){return t.setFromCoplanarPoints(this.a,this.b,this.c)}getBarycoord(t,e){return i.getBarycoord(t,this.a,this.b,this.c,e)}getInterpolation(t,e,n,s,r){return i.getInterpolation(t,this.a,this.b,this.c,e,n,s,r)}containsPoint(t){return i.containsPoint(t,this.a,this.b,this.c)}isFrontFacing(t){return i.isFrontFacing(this.a,this.b,this.c,t)}intersectsBox(t){return t.intersectsTriangle(this)}closestPointToPoint(t,e){let n=this.a,s=this.b,r=this.c,a,o;ns.subVectors(s,n),is.subVectors(r,n),_l.subVectors(t,n);let l=ns.dot(_l),c=is.dot(_l);if(l<=0&&c<=0)return e.copy(n);vl.subVectors(t,s);let d=ns.dot(vl),u=is.dot(vl);if(d>=0&&u<=d)return e.copy(s);let h=l*u-d*c;if(h<=0&&l>=0&&d<=0)return a=l/(l-d),e.copy(n).addScaledVector(ns,a);xl.subVectors(t,r);let p=ns.dot(xl),_=is.dot(xl);if(_>=0&&p<=_)return e.copy(r);let y=p*c-l*_;if(y<=0&&c>=0&&_<=0)return o=c/(c-_),e.copy(n).addScaledVector(is,o);let m=d*_-p*u;if(m<=0&&u-d>=0&&p-_>=0)return vh.subVectors(r,s),o=(u-d)/(u-d+(p-_)),e.copy(s).addScaledVector(vh,o);let f=1/(m+y+h);return a=y*f,o=h*f,e.copy(n).addScaledVector(ns,a).addScaledVector(is,o)}equals(t){return t.a.equals(this.a)&&t.b.equals(this.b)&&t.c.equals(this.c)}},fi=class{constructor(t=new k(1/0,1/0,1/0),e=new k(-1/0,-1/0,-1/0)){this.isBox3=!0,this.min=t,this.max=e}set(t,e){return this.min.copy(t),this.max.copy(e),this}setFromArray(t){this.makeEmpty();for(let e=0,n=t.length;e<n;e+=3)this.expandByPoint(_n.fromArray(t,e));return this}setFromBufferAttribute(t){this.makeEmpty();for(let e=0,n=t.count;e<n;e++)this.expandByPoint(_n.fromBufferAttribute(t,e));return this}setFromPoints(t){this.makeEmpty();for(let e=0,n=t.length;e<n;e++)this.expandByPoint(t[e]);return this}setFromCenterAndSize(t,e){let n=_n.copy(e).multiplyScalar(.5);return this.min.copy(t).sub(n),this.max.copy(t).add(n),this}setFromObject(t,e=!1){return this.makeEmpty(),this.expandByObject(t,e)}clone(){return new this.constructor().copy(this)}copy(t){return this.min.copy(t.min),this.max.copy(t.max),this}makeEmpty(){return this.min.x=this.min.y=this.min.z=1/0,this.max.x=this.max.y=this.max.z=-1/0,this}isEmpty(){return this.max.x<this.min.x||this.max.y<this.min.y||this.max.z<this.min.z}getCenter(t){return this.isEmpty()?t.set(0,0,0):t.addVectors(this.min,this.max).multiplyScalar(.5)}getSize(t){return this.isEmpty()?t.set(0,0,0):t.subVectors(this.max,this.min)}expandByPoint(t){return this.min.min(t),this.max.max(t),this}expandByVector(t){return this.min.sub(t),this.max.add(t),this}expandByScalar(t){return this.min.addScalar(-t),this.max.addScalar(t),this}expandByObject(t,e=!1){t.updateWorldMatrix(!1,!1);let n=t.geometry;if(n!==void 0){let r=n.getAttribute("position");if(e===!0&&r!==void 0&&t.isInstancedMesh!==!0)for(let a=0,o=r.count;a<o;a++)t.isMesh===!0?t.getVertexPosition(a,_n):_n.fromBufferAttribute(r,a),_n.applyMatrix4(t.matrixWorld),this.expandByPoint(_n);else t.boundingBox!==void 0?(t.boundingBox===null&&t.computeBoundingBox(),Br.copy(t.boundingBox)):(n.boundingBox===null&&n.computeBoundingBox(),Br.copy(n.boundingBox)),Br.applyMatrix4(t.matrixWorld),this.union(Br)}let s=t.children;for(let r=0,a=s.length;r<a;r++)this.expandByObject(s[r],e);return this}containsPoint(t){return t.x>=this.min.x&&t.x<=this.max.x&&t.y>=this.min.y&&t.y<=this.max.y&&t.z>=this.min.z&&t.z<=this.max.z}containsBox(t){return this.min.x<=t.min.x&&t.max.x<=this.max.x&&this.min.y<=t.min.y&&t.max.y<=this.max.y&&this.min.z<=t.min.z&&t.max.z<=this.max.z}getParameter(t,e){return e.set((t.x-this.min.x)/(this.max.x-this.min.x),(t.y-this.min.y)/(this.max.y-this.min.y),(t.z-this.min.z)/(this.max.z-this.min.z))}intersectsBox(t){return t.max.x>=this.min.x&&t.min.x<=this.max.x&&t.max.y>=this.min.y&&t.min.y<=this.max.y&&t.max.z>=this.min.z&&t.min.z<=this.max.z}intersectsSphere(t){return this.clampPoint(t.center,_n),_n.distanceToSquared(t.center)<=t.radius*t.radius}intersectsPlane(t){let e,n;return t.normal.x>0?(e=t.normal.x*this.min.x,n=t.normal.x*this.max.x):(e=t.normal.x*this.max.x,n=t.normal.x*this.min.x),t.normal.y>0?(e+=t.normal.y*this.min.y,n+=t.normal.y*this.max.y):(e+=t.normal.y*this.max.y,n+=t.normal.y*this.min.y),t.normal.z>0?(e+=t.normal.z*this.min.z,n+=t.normal.z*this.max.z):(e+=t.normal.z*this.max.z,n+=t.normal.z*this.min.z),e<=-t.constant&&n>=-t.constant}intersectsTriangle(t){if(this.isEmpty())return!1;this.getCenter(Os),kr.subVectors(this.max,Os),ss.subVectors(t.a,Os),rs.subVectors(t.b,Os),as.subVectors(t.c,Os),oi.subVectors(rs,ss),li.subVectors(as,rs),Di.subVectors(ss,as);let e=[0,-oi.z,oi.y,0,-li.z,li.y,0,-Di.z,Di.y,oi.z,0,-oi.x,li.z,0,-li.x,Di.z,0,-Di.x,-oi.y,oi.x,0,-li.y,li.x,0,-Di.y,Di.x,0];return!Ml(e,ss,rs,as,kr)||(e=[1,0,0,0,1,0,0,0,1],!Ml(e,ss,rs,as,kr))?!1:(zr.crossVectors(oi,li),e=[zr.x,zr.y,zr.z],Ml(e,ss,rs,as,kr))}clampPoint(t,e){return e.copy(t).clamp(this.min,this.max)}distanceToPoint(t){return this.clampPoint(t,_n).distanceTo(t)}getBoundingSphere(t){return this.isEmpty()?t.makeEmpty():(this.getCenter(t.center),t.radius=this.getSize(_n).length()*.5),t}intersect(t){return this.min.max(t.min),this.max.min(t.max),this.isEmpty()&&this.makeEmpty(),this}union(t){return this.min.min(t.min),this.max.max(t.max),this}applyMatrix4(t){return this.isEmpty()?this:(Xn[0].set(this.min.x,this.min.y,this.min.z).applyMatrix4(t),Xn[1].set(this.min.x,this.min.y,this.max.z).applyMatrix4(t),Xn[2].set(this.min.x,this.max.y,this.min.z).applyMatrix4(t),Xn[3].set(this.min.x,this.max.y,this.max.z).applyMatrix4(t),Xn[4].set(this.max.x,this.min.y,this.min.z).applyMatrix4(t),Xn[5].set(this.max.x,this.min.y,this.max.z).applyMatrix4(t),Xn[6].set(this.max.x,this.max.y,this.min.z).applyMatrix4(t),Xn[7].set(this.max.x,this.max.y,this.max.z).applyMatrix4(t),this.setFromPoints(Xn),this)}translate(t){return this.min.add(t),this.max.add(t),this}equals(t){return t.min.equals(this.min)&&t.max.equals(this.max)}toJSON(){return{min:this.min.toArray(),max:this.max.toArray()}}fromJSON(t){return this.min.fromArray(t.min),this.max.fromArray(t.max),this}},Xn=[new k,new k,new k,new k,new k,new k,new k,new k],_n=new k,Br=new fi,ss=new k,rs=new k,as=new k,oi=new k,li=new k,Di=new k,Os=new k,kr=new k,zr=new k,Ni=new k;function Ml(i,t,e,n,s){for(let r=0,a=i.length-3;r<=a;r+=3){Ni.fromArray(i,r);let o=s.x*Math.abs(Ni.x)+s.y*Math.abs(Ni.y)+s.z*Math.abs(Ni.z),l=t.dot(Ni),c=e.dot(Ni),d=n.dot(Ni);if(Math.max(-Math.max(l,c,d),Math.min(l,c,d))>o)return!1}return!0}var be=new k,Vr=new jt,Lu=0,Je=class extends Dn{constructor(t,e,n=!1){if(super(),Array.isArray(t))throw new TypeError("THREE.BufferAttribute: array should be a Typed Array.");this.isBufferAttribute=!0,Object.defineProperty(this,"id",{value:Lu++}),this.name="",this.array=t,this.itemSize=e,this.count=t!==void 0?t.length/e:0,this.normalized=n,this.usage=Bl,this.updateRanges=[],this.gpuType=bn,this.version=0}onUploadCallback(){}set needsUpdate(t){t===!0&&this.version++}setUsage(t){return this.usage=t,this}addUpdateRange(t,e){this.updateRanges.push({start:t,count:e})}clearUpdateRanges(){this.updateRanges.length=0}copy(t){return this.name=t.name,this.array=new t.array.constructor(t.array),this.itemSize=t.itemSize,this.count=t.count,this.normalized=t.normalized,this.usage=t.usage,this.gpuType=t.gpuType,this}copyAt(t,e,n){t*=this.itemSize,n*=e.itemSize;for(let s=0,r=this.itemSize;s<r;s++)this.array[t+s]=e.array[n+s];return this}copyArray(t){return this.array.set(t),this}applyMatrix3(t){if(this.itemSize===2)for(let e=0,n=this.count;e<n;e++)Vr.fromBufferAttribute(this,e),Vr.applyMatrix3(t),this.setXY(e,Vr.x,Vr.y);else if(this.itemSize===3)for(let e=0,n=this.count;e<n;e++)be.fromBufferAttribute(this,e),be.applyMatrix3(t),this.setXYZ(e,be.x,be.y,be.z);return this}applyMatrix4(t){for(let e=0,n=this.count;e<n;e++)be.fromBufferAttribute(this,e),be.applyMatrix4(t),this.setXYZ(e,be.x,be.y,be.z);return this}applyNormalMatrix(t){for(let e=0,n=this.count;e<n;e++)be.fromBufferAttribute(this,e),be.applyNormalMatrix(t),this.setXYZ(e,be.x,be.y,be.z);return this}transformDirection(t){for(let e=0,n=this.count;e<n;e++)be.fromBufferAttribute(this,e),be.transformDirection(t),this.setXYZ(e,be.x,be.y,be.z);return this}set(t,e=0){return this.array.set(t,e),this}getComponent(t,e){let n=this.array[t*this.itemSize+e];return this.normalized&&(n=Us(n,this.array)),n}setComponent(t,e,n){return this.normalized&&(n=Ve(n,this.array)),this.array[t*this.itemSize+e]=n,this}getX(t){let e=this.array[t*this.itemSize];return this.normalized&&(e=Us(e,this.array)),e}setX(t,e){return this.normalized&&(e=Ve(e,this.array)),this.array[t*this.itemSize]=e,this}getY(t){let e=this.array[t*this.itemSize+1];return this.normalized&&(e=Us(e,this.array)),e}setY(t,e){return this.normalized&&(e=Ve(e,this.array)),this.array[t*this.itemSize+1]=e,this}getZ(t){let e=this.array[t*this.itemSize+2];return this.normalized&&(e=Us(e,this.array)),e}setZ(t,e){return this.normalized&&(e=Ve(e,this.array)),this.array[t*this.itemSize+2]=e,this}getW(t){let e=this.array[t*this.itemSize+3];return this.normalized&&(e=Us(e,this.array)),e}setW(t,e){return this.normalized&&(e=Ve(e,this.array)),this.array[t*this.itemSize+3]=e,this}setXY(t,e,n){return t*=this.itemSize,this.normalized&&(e=Ve(e,this.array),n=Ve(n,this.array)),this.array[t+0]=e,this.array[t+1]=n,this}setXYZ(t,e,n,s){return t*=this.itemSize,this.normalized&&(e=Ve(e,this.array),n=Ve(n,this.array),s=Ve(s,this.array)),this.array[t+0]=e,this.array[t+1]=n,this.array[t+2]=s,this}setXYZW(t,e,n,s,r){return t*=this.itemSize,this.normalized&&(e=Ve(e,this.array),n=Ve(n,this.array),s=Ve(s,this.array),r=Ve(r,this.array)),this.array[t+0]=e,this.array[t+1]=n,this.array[t+2]=s,this.array[t+3]=r,this}onUpload(t){return this.onUploadCallback=t,this}clone(){return new this.constructor(this.array,this.itemSize).copy(this)}toJSON(){let t={itemSize:this.itemSize,type:this.array.constructor.name,array:Array.from(this.array),normalized:this.normalized};return this.name!==""&&(t.name=this.name),this.usage!==Bl&&(t.usage=this.usage),t}dispose(){this.dispatchEvent({type:"dispose"})}};var Ys=class extends Je{constructor(t,e,n){super(new Uint16Array(t),e,n)}};var Zs=class extends Je{constructor(t,e,n){super(new Uint32Array(t),e,n)}};var Fe=class extends Je{constructor(t,e,n){super(new Float32Array(t),e,n)}},Du=new fi,Bs=new k,wl=new k,Vi=class{constructor(t=new k,e=-1){this.isSphere=!0,this.center=t,this.radius=e}set(t,e){return this.center.copy(t),this.radius=e,this}setFromPoints(t,e){let n=this.center;e!==void 0?n.copy(e):Du.setFromPoints(t).getCenter(n);let s=0;for(let r=0,a=t.length;r<a;r++)s=Math.max(s,n.distanceToSquared(t[r]));return this.radius=Math.sqrt(s),this}copy(t){return this.center.copy(t.center),this.radius=t.radius,this}isEmpty(){return this.radius<0}makeEmpty(){return this.center.set(0,0,0),this.radius=-1,this}containsPoint(t){return t.distanceToSquared(this.center)<=this.radius*this.radius}distanceToPoint(t){return t.distanceTo(this.center)-this.radius}intersectsSphere(t){let e=this.radius+t.radius;return t.center.distanceToSquared(this.center)<=e*e}intersectsBox(t){return t.intersectsSphere(this)}intersectsPlane(t){return Math.abs(t.distanceToPoint(this.center))<=this.radius}clampPoint(t,e){let n=this.center.distanceToSquared(t);return e.copy(t),n>this.radius*this.radius&&(e.sub(this.center).normalize(),e.multiplyScalar(this.radius).add(this.center)),e}getBoundingBox(t){return this.isEmpty()?(t.makeEmpty(),t):(t.set(this.center,this.center),t.expandByScalar(this.radius),t)}applyMatrix4(t){return this.center.applyMatrix4(t),this.radius=this.radius*t.getMaxScaleOnAxis(),this}translate(t){return this.center.add(t),this}expandByPoint(t){if(this.isEmpty())return this.center.copy(t),this.radius=0,this;Bs.subVectors(t,this.center);let e=Bs.lengthSq();if(e>this.radius*this.radius){let n=Math.sqrt(e),s=(n-this.radius)*.5;this.center.addScaledVector(Bs,s/n),this.radius+=s}return this}union(t){return t.isEmpty()?this:this.isEmpty()?(this.copy(t),this):(this.center.equals(t.center)===!0?this.radius=Math.max(this.radius,t.radius):(wl.subVectors(t.center,this.center).setLength(t.radius),this.expandByPoint(Bs.copy(t.center).add(wl)),this.expandByPoint(Bs.copy(t.center).sub(wl))),this)}equals(t){return t.center.equals(this.center)&&t.radius===this.radius}clone(){return new this.constructor().copy(this)}toJSON(){return{radius:this.radius,center:this.center.toArray()}}fromJSON(t){return this.radius=t.radius,this.center.fromArray(t.center),this}},Nu=0,ln=new fe,Al=new Oe,os=new k,Ye=new fi,ks=new fi,Ce=new k,cn=class i extends Dn{constructor(){super(),this.isBufferGeometry=!0,Object.defineProperty(this,"id",{value:Nu++}),this.uuid=gr(),this.name="",this.type="BufferGeometry",this.index=null,this.indirect=null,this.indirectOffset=0,this.attributes={},this.morphAttributes={},this.morphTargetsRelative=!1,this.groups=[],this.boundingBox=null,this.boundingSphere=null,this.drawRange={start:0,count:1/0},this.userData={}}getIndex(){return this.index}setIndex(t){return Array.isArray(t)?this.index=new(xu(t)?Zs:Ys)(t,1):this.index=t,this}setIndirect(t,e=0){return this.indirect=t,this.indirectOffset=e,this}getIndirect(){return this.indirect}getAttribute(t){return this.attributes[t]}setAttribute(t,e){return this.attributes[t]=e,this}deleteAttribute(t){return delete this.attributes[t],this}hasAttribute(t){return this.attributes[t]!==void 0}addGroup(t,e,n=0){this.groups.push({start:t,count:e,materialIndex:n})}clearGroups(){this.groups=[]}setDrawRange(t,e){this.drawRange.start=t,this.drawRange.count=e}applyMatrix4(t){let e=this.attributes.position;e!==void 0&&(e.applyMatrix4(t),e.needsUpdate=!0);let n=this.attributes.normal;if(n!==void 0){let r=new Pt().getNormalMatrix(t);n.applyNormalMatrix(r),n.needsUpdate=!0}let s=this.attributes.tangent;return s!==void 0&&(s.transformDirection(t),s.needsUpdate=!0),this.boundingBox!==null&&this.computeBoundingBox(),this.boundingSphere!==null&&this.computeBoundingSphere(),this}applyQuaternion(t){return ln.makeRotationFromQuaternion(t),this.applyMatrix4(ln),this}rotateX(t){return ln.makeRotationX(t),this.applyMatrix4(ln),this}rotateY(t){return ln.makeRotationY(t),this.applyMatrix4(ln),this}rotateZ(t){return ln.makeRotationZ(t),this.applyMatrix4(ln),this}translate(t,e,n){return ln.makeTranslation(t,e,n),this.applyMatrix4(ln),this}scale(t,e,n){return ln.makeScale(t,e,n),this.applyMatrix4(ln),this}lookAt(t){return Al.lookAt(t),Al.updateMatrix(),this.applyMatrix4(Al.matrix),this}center(){return this.computeBoundingBox(),this.boundingBox.getCenter(os).negate(),this.translate(os.x,os.y,os.z),this}setFromPoints(t){let e=this.getAttribute("position");if(e===void 0){let n=[];for(let s=0,r=t.length;s<r;s++){let a=t[s];n.push(a.x,a.y,a.z||0)}this.setAttribute("position",new Fe(n,3))}else{let n=Math.min(t.length,e.count);for(let s=0;s<n;s++){let r=t[s];e.setXYZ(s,r.x,r.y,r.z||0)}t.length>e.count&&Et("BufferGeometry: Buffer size too small for points data. Use .dispose() and create a new geometry."),e.needsUpdate=!0}return this}computeBoundingBox(){this.boundingBox===null&&(this.boundingBox=new fi);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){Ct("BufferGeometry.computeBoundingBox(): GLBufferAttribute requires a manual bounding box.",this),this.boundingBox.set(new k(-1/0,-1/0,-1/0),new k(1/0,1/0,1/0));return}if(t!==void 0){if(this.boundingBox.setFromBufferAttribute(t),e)for(let n=0,s=e.length;n<s;n++){let r=e[n];Ye.setFromBufferAttribute(r),this.morphTargetsRelative?(Ce.addVectors(this.boundingBox.min,Ye.min),this.boundingBox.expandByPoint(Ce),Ce.addVectors(this.boundingBox.max,Ye.max),this.boundingBox.expandByPoint(Ce)):(this.boundingBox.expandByPoint(Ye.min),this.boundingBox.expandByPoint(Ye.max))}}else this.boundingBox.makeEmpty();(isNaN(this.boundingBox.min.x)||isNaN(this.boundingBox.min.y)||isNaN(this.boundingBox.min.z))&&Ct('BufferGeometry.computeBoundingBox(): Computed min/max have NaN values. The "position" attribute is likely to have NaN values.',this)}computeBoundingSphere(){this.boundingSphere===null&&(this.boundingSphere=new Vi);let t=this.attributes.position,e=this.morphAttributes.position;if(t&&t.isGLBufferAttribute){Ct("BufferGeometry.computeBoundingSphere(): GLBufferAttribute requires a manual bounding sphere.",this),this.boundingSphere.set(new k,1/0);return}if(t){let n=this.boundingSphere.center;if(Ye.setFromBufferAttribute(t),e)for(let r=0,a=e.length;r<a;r++){let o=e[r];ks.setFromBufferAttribute(o),this.morphTargetsRelative?(Ce.addVectors(Ye.min,ks.min),Ye.expandByPoint(Ce),Ce.addVectors(Ye.max,ks.max),Ye.expandByPoint(Ce)):(Ye.expandByPoint(ks.min),Ye.expandByPoint(ks.max))}Ye.getCenter(n);let s=0;for(let r=0,a=t.count;r<a;r++)Ce.fromBufferAttribute(t,r),s=Math.max(s,n.distanceToSquared(Ce));if(e)for(let r=0,a=e.length;r<a;r++){let o=e[r],l=this.morphTargetsRelative;for(let c=0,d=o.count;c<d;c++)Ce.fromBufferAttribute(o,c),l&&(os.fromBufferAttribute(t,c),Ce.add(os)),s=Math.max(s,n.distanceToSquared(Ce))}this.boundingSphere.radius=Math.sqrt(s),isNaN(this.boundingSphere.radius)&&Ct('BufferGeometry.computeBoundingSphere(): Computed radius is NaN. The "position" attribute is likely to have NaN values.',this)}}computeTangents(){let t=this.index,e=this.attributes;if(t===null||e.position===void 0||e.normal===void 0||e.uv===void 0){Ct("BufferGeometry: .computeTangents() failed. Missing required attributes (index, position, normal or uv)");return}let n=e.position,s=e.normal,r=e.uv;this.hasAttribute("tangent")===!1&&this.setAttribute("tangent",new Je(new Float32Array(4*n.count),4));let a=this.getAttribute("tangent"),o=[],l=[];for(let v=0;v<n.count;v++)o[v]=new k,l[v]=new k;let c=new k,d=new k,u=new k,h=new jt,p=new jt,_=new jt,y=new k,m=new k;function f(v,A,D){c.fromBufferAttribute(n,v),d.fromBufferAttribute(n,A),u.fromBufferAttribute(n,D),h.fromBufferAttribute(r,v),p.fromBufferAttribute(r,A),_.fromBufferAttribute(r,D),d.sub(c),u.sub(c),p.sub(h),_.sub(h);let T=1/(p.x*_.y-_.x*p.y);isFinite(T)&&(y.copy(d).multiplyScalar(_.y).addScaledVector(u,-p.y).multiplyScalar(T),m.copy(u).multiplyScalar(p.x).addScaledVector(d,-_.x).multiplyScalar(T),o[v].add(y),o[A].add(y),o[D].add(y),l[v].add(m),l[A].add(m),l[D].add(m))}let S=this.groups;S.length===0&&(S=[{start:0,count:t.count}]);for(let v=0,A=S.length;v<A;++v){let D=S[v],T=D.start,O=D.count;for(let H=T,$=T+O;H<$;H+=3)f(t.getX(H+0),t.getX(H+1),t.getX(H+2))}let M=new k,E=new k,R=new k,w=new k;function I(v){R.fromBufferAttribute(s,v),w.copy(R);let A=o[v];M.copy(A),M.sub(R.multiplyScalar(R.dot(A))).normalize(),E.crossVectors(w,A);let T=E.dot(l[v])<0?-1:1;a.setXYZW(v,M.x,M.y,M.z,T)}for(let v=0,A=S.length;v<A;++v){let D=S[v],T=D.start,O=D.count;for(let H=T,$=T+O;H<$;H+=3)I(t.getX(H+0)),I(t.getX(H+1)),I(t.getX(H+2))}}computeVertexNormals(){let t=this.index,e=this.getAttribute("position");if(e!==void 0){let n=this.getAttribute("normal");if(n===void 0)n=new Je(new Float32Array(e.count*3),3),this.setAttribute("normal",n);else for(let h=0,p=n.count;h<p;h++)n.setXYZ(h,0,0,0);let s=new k,r=new k,a=new k,o=new k,l=new k,c=new k,d=new k,u=new k;if(t)for(let h=0,p=t.count;h<p;h+=3){let _=t.getX(h+0),y=t.getX(h+1),m=t.getX(h+2);s.fromBufferAttribute(e,_),r.fromBufferAttribute(e,y),a.fromBufferAttribute(e,m),d.subVectors(a,r),u.subVectors(s,r),d.cross(u),o.fromBufferAttribute(n,_),l.fromBufferAttribute(n,y),c.fromBufferAttribute(n,m),o.add(d),l.add(d),c.add(d),n.setXYZ(_,o.x,o.y,o.z),n.setXYZ(y,l.x,l.y,l.z),n.setXYZ(m,c.x,c.y,c.z)}else for(let h=0,p=e.count;h<p;h+=3)s.fromBufferAttribute(e,h+0),r.fromBufferAttribute(e,h+1),a.fromBufferAttribute(e,h+2),d.subVectors(a,r),u.subVectors(s,r),d.cross(u),n.setXYZ(h+0,d.x,d.y,d.z),n.setXYZ(h+1,d.x,d.y,d.z),n.setXYZ(h+2,d.x,d.y,d.z);this.normalizeNormals(),n.needsUpdate=!0}}normalizeNormals(){let t=this.attributes.normal;for(let e=0,n=t.count;e<n;e++)Ce.fromBufferAttribute(t,e),Ce.normalize(),t.setXYZ(e,Ce.x,Ce.y,Ce.z)}toNonIndexed(){function t(o,l){let c=o.array,d=o.itemSize,u=o.normalized,h=new c.constructor(l.length*d),p=0,_=0;for(let y=0,m=l.length;y<m;y++){o.isInterleavedBufferAttribute?p=l[y]*o.data.stride+o.offset:p=l[y]*d;for(let f=0;f<d;f++)h[_++]=c[p++]}return new Je(h,d,u)}if(this.index===null)return Et("BufferGeometry.toNonIndexed(): BufferGeometry is already non-indexed."),this;let e=new i,n=this.index.array,s=this.attributes;for(let o in s){let l=s[o],c=t(l,n);e.setAttribute(o,c)}let r=this.morphAttributes;for(let o in r){let l=[],c=r[o];for(let d=0,u=c.length;d<u;d++){let h=c[d],p=t(h,n);l.push(p)}e.morphAttributes[o]=l}e.morphTargetsRelative=this.morphTargetsRelative;let a=this.groups;for(let o=0,l=a.length;o<l;o++){let c=a[o];e.addGroup(c.start,c.count,c.materialIndex)}return e}toJSON(){let t={metadata:{version:4.7,type:"BufferGeometry",generator:"BufferGeometry.toJSON"}};if(t.uuid=this.uuid,t.type=this.type,this.name!==""&&(t.name=this.name),Object.keys(this.userData).length>0&&(t.userData=this.userData),this.parameters!==void 0){let l=this.parameters;for(let c in l)l[c]!==void 0&&(t[c]=l[c]);return t}t.data={attributes:{}};let e=this.index;e!==null&&(t.data.index={type:e.array.constructor.name,array:Array.prototype.slice.call(e.array)});let n=this.attributes;for(let l in n){let c=n[l];t.data.attributes[l]=c.toJSON(t.data)}let s={},r=!1;for(let l in this.morphAttributes){let c=this.morphAttributes[l],d=[];for(let u=0,h=c.length;u<h;u++){let p=c[u];d.push(p.toJSON(t.data))}d.length>0&&(s[l]=d,r=!0)}r&&(t.data.morphAttributes=s,t.data.morphTargetsRelative=this.morphTargetsRelative);let a=this.groups;a.length>0&&(t.data.groups=JSON.parse(JSON.stringify(a)));let o=this.boundingSphere;return o!==null&&(t.data.boundingSphere=o.toJSON()),t}clone(){return new this.constructor().copy(this)}copy(t){this.index=null,this.attributes={},this.morphAttributes={},this.groups=[],this.boundingBox=null,this.boundingSphere=null;let e={};this.name=t.name;let n=t.index;n!==null&&this.setIndex(n.clone());let s=t.attributes;for(let c in s){let d=s[c];this.setAttribute(c,d.clone(e))}let r=t.morphAttributes;for(let c in r){let d=[],u=r[c];for(let h=0,p=u.length;h<p;h++)d.push(u[h].clone(e));this.morphAttributes[c]=d}this.morphTargetsRelative=t.morphTargetsRelative;let a=t.groups;for(let c=0,d=a.length;c<d;c++){let u=a[c];this.addGroup(u.start,u.count,u.materialIndex)}let o=t.boundingBox;o!==null&&(this.boundingBox=o.clone());let l=t.boundingSphere;return l!==null&&(this.boundingSphere=l.clone()),this.drawRange.start=t.drawRange.start,this.drawRange.count=t.drawRange.count,this.userData=t.userData,this}dispose(){this.dispatchEvent({type:"dispose"})}};var Uu=0,Kn=class extends Dn{constructor(){super(),this.isMaterial=!0,Object.defineProperty(this,"id",{value:Uu++}),this.uuid=gr(),this.name="",this.type="Material",this.blending=ki,this.side=Zn,this.vertexColors=!1,this.opacity=1,this.transparent=!1,this.alphaHash=!1,this.blendSrc=sa,this.blendDst=ra,this.blendEquation=ui,this.blendSrcAlpha=null,this.blendDstAlpha=null,this.blendEquationAlpha=null,this.blendColor=new zt(0,0,0),this.blendAlpha=0,this.depthFunc=zi,this.depthTest=!0,this.depthWrite=!0,this.stencilWriteMask=255,this.stencilFunc=Ol,this.stencilRef=0,this.stencilFuncMask=255,this.stencilFail=Oi,this.stencilZFail=Oi,this.stencilZPass=Oi,this.stencilWrite=!1,this.clippingPlanes=null,this.clipIntersection=!1,this.clipShadows=!1,this.shadowSide=null,this.colorWrite=!0,this.precision=null,this.polygonOffset=!1,this.polygonOffsetFactor=0,this.polygonOffsetUnits=0,this.dithering=!1,this.alphaToCoverage=!1,this.premultipliedAlpha=!1,this.forceSinglePass=!1,this.allowOverride=!0,this.visible=!0,this.toneMapped=!0,this.userData={},this.version=0,this._alphaTest=0}get alphaTest(){return this._alphaTest}set alphaTest(t){this._alphaTest>0!=t>0&&this.version++,this._alphaTest=t}onBeforeRender(){}onBeforeCompile(){}customProgramCacheKey(){return this.onBeforeCompile.toString()}setValues(t){if(t!==void 0)for(let e in t){let n=t[e];if(n===void 0){Et(`Material: parameter '${e}' has value of undefined.`);continue}let s=this[e];if(s===void 0){Et(`Material: '${e}' is not a property of THREE.${this.type}.`);continue}s&&s.isColor?s.set(n):s&&s.isVector3&&n&&n.isVector3?s.copy(n):this[e]=n}}toJSON(t){let e=t===void 0||typeof t=="string";e&&(t={textures:{},images:{}});let n={metadata:{version:4.7,type:"Material",generator:"Material.toJSON"}};n.uuid=this.uuid,n.type=this.type,this.name!==""&&(n.name=this.name),this.color&&this.color.isColor&&(n.color=this.color.getHex()),this.roughness!==void 0&&(n.roughness=this.roughness),this.metalness!==void 0&&(n.metalness=this.metalness),this.sheen!==void 0&&(n.sheen=this.sheen),this.sheenColor&&this.sheenColor.isColor&&(n.sheenColor=this.sheenColor.getHex()),this.sheenRoughness!==void 0&&(n.sheenRoughness=this.sheenRoughness),this.emissive&&this.emissive.isColor&&(n.emissive=this.emissive.getHex()),this.emissiveIntensity!==void 0&&this.emissiveIntensity!==1&&(n.emissiveIntensity=this.emissiveIntensity),this.specular&&this.specular.isColor&&(n.specular=this.specular.getHex()),this.specularIntensity!==void 0&&(n.specularIntensity=this.specularIntensity),this.specularColor&&this.specularColor.isColor&&(n.specularColor=this.specularColor.getHex()),this.shininess!==void 0&&(n.shininess=this.shininess),this.clearcoat!==void 0&&(n.clearcoat=this.clearcoat),this.clearcoatRoughness!==void 0&&(n.clearcoatRoughness=this.clearcoatRoughness),this.clearcoatMap&&this.clearcoatMap.isTexture&&(n.clearcoatMap=this.clearcoatMap.toJSON(t).uuid),this.clearcoatRoughnessMap&&this.clearcoatRoughnessMap.isTexture&&(n.clearcoatRoughnessMap=this.clearcoatRoughnessMap.toJSON(t).uuid),this.clearcoatNormalMap&&this.clearcoatNormalMap.isTexture&&(n.clearcoatNormalMap=this.clearcoatNormalMap.toJSON(t).uuid,n.clearcoatNormalScale=this.clearcoatNormalScale.toArray()),this.sheenColorMap&&this.sheenColorMap.isTexture&&(n.sheenColorMap=this.sheenColorMap.toJSON(t).uuid),this.sheenRoughnessMap&&this.sheenRoughnessMap.isTexture&&(n.sheenRoughnessMap=this.sheenRoughnessMap.toJSON(t).uuid),this.dispersion!==void 0&&(n.dispersion=this.dispersion),this.iridescence!==void 0&&(n.iridescence=this.iridescence),this.iridescenceIOR!==void 0&&(n.iridescenceIOR=this.iridescenceIOR),this.iridescenceThicknessRange!==void 0&&(n.iridescenceThicknessRange=this.iridescenceThicknessRange),this.iridescenceMap&&this.iridescenceMap.isTexture&&(n.iridescenceMap=this.iridescenceMap.toJSON(t).uuid),this.iridescenceThicknessMap&&this.iridescenceThicknessMap.isTexture&&(n.iridescenceThicknessMap=this.iridescenceThicknessMap.toJSON(t).uuid),this.anisotropy!==void 0&&(n.anisotropy=this.anisotropy),this.anisotropyRotation!==void 0&&(n.anisotropyRotation=this.anisotropyRotation),this.anisotropyMap&&this.anisotropyMap.isTexture&&(n.anisotropyMap=this.anisotropyMap.toJSON(t).uuid),this.map&&this.map.isTexture&&(n.map=this.map.toJSON(t).uuid),this.matcap&&this.matcap.isTexture&&(n.matcap=this.matcap.toJSON(t).uuid),this.alphaMap&&this.alphaMap.isTexture&&(n.alphaMap=this.alphaMap.toJSON(t).uuid),this.lightMap&&this.lightMap.isTexture&&(n.lightMap=this.lightMap.toJSON(t).uuid,n.lightMapIntensity=this.lightMapIntensity),this.aoMap&&this.aoMap.isTexture&&(n.aoMap=this.aoMap.toJSON(t).uuid,n.aoMapIntensity=this.aoMapIntensity),this.bumpMap&&this.bumpMap.isTexture&&(n.bumpMap=this.bumpMap.toJSON(t).uuid,n.bumpScale=this.bumpScale),this.normalMap&&this.normalMap.isTexture&&(n.normalMap=this.normalMap.toJSON(t).uuid,n.normalMapType=this.normalMapType,n.normalScale=this.normalScale.toArray()),this.displacementMap&&this.displacementMap.isTexture&&(n.displacementMap=this.displacementMap.toJSON(t).uuid,n.displacementScale=this.displacementScale,n.displacementBias=this.displacementBias),this.roughnessMap&&this.roughnessMap.isTexture&&(n.roughnessMap=this.roughnessMap.toJSON(t).uuid),this.metalnessMap&&this.metalnessMap.isTexture&&(n.metalnessMap=this.metalnessMap.toJSON(t).uuid),this.emissiveMap&&this.emissiveMap.isTexture&&(n.emissiveMap=this.emissiveMap.toJSON(t).uuid),this.specularMap&&this.specularMap.isTexture&&(n.specularMap=this.specularMap.toJSON(t).uuid),this.specularIntensityMap&&this.specularIntensityMap.isTexture&&(n.specularIntensityMap=this.specularIntensityMap.toJSON(t).uuid),this.specularColorMap&&this.specularColorMap.isTexture&&(n.specularColorMap=this.specularColorMap.toJSON(t).uuid),this.envMap&&this.envMap.isTexture&&(n.envMap=this.envMap.toJSON(t).uuid,this.combine!==void 0&&(n.combine=this.combine)),this.envMapRotation!==void 0&&(n.envMapRotation=this.envMapRotation.toArray()),this.envMapIntensity!==void 0&&(n.envMapIntensity=this.envMapIntensity),this.reflectivity!==void 0&&(n.reflectivity=this.reflectivity),this.refractionRatio!==void 0&&(n.refractionRatio=this.refractionRatio),this.gradientMap&&this.gradientMap.isTexture&&(n.gradientMap=this.gradientMap.toJSON(t).uuid),this.transmission!==void 0&&(n.transmission=this.transmission),this.transmissionMap&&this.transmissionMap.isTexture&&(n.transmissionMap=this.transmissionMap.toJSON(t).uuid),this.thickness!==void 0&&(n.thickness=this.thickness),this.thicknessMap&&this.thicknessMap.isTexture&&(n.thicknessMap=this.thicknessMap.toJSON(t).uuid),this.attenuationDistance!==void 0&&this.attenuationDistance!==1/0&&(n.attenuationDistance=this.attenuationDistance),this.attenuationColor!==void 0&&(n.attenuationColor=this.attenuationColor.getHex()),this.size!==void 0&&(n.size=this.size),this.shadowSide!==null&&(n.shadowSide=this.shadowSide),this.sizeAttenuation!==void 0&&(n.sizeAttenuation=this.sizeAttenuation),this.blending!==ki&&(n.blending=this.blending),this.side!==Zn&&(n.side=this.side),this.vertexColors===!0&&(n.vertexColors=!0),this.opacity<1&&(n.opacity=this.opacity),this.transparent===!0&&(n.transparent=!0),this.blendSrc!==sa&&(n.blendSrc=this.blendSrc),this.blendDst!==ra&&(n.blendDst=this.blendDst),this.blendEquation!==ui&&(n.blendEquation=this.blendEquation),this.blendSrcAlpha!==null&&(n.blendSrcAlpha=this.blendSrcAlpha),this.blendDstAlpha!==null&&(n.blendDstAlpha=this.blendDstAlpha),this.blendEquationAlpha!==null&&(n.blendEquationAlpha=this.blendEquationAlpha),this.blendColor&&this.blendColor.isColor&&(n.blendColor=this.blendColor.getHex()),this.blendAlpha!==0&&(n.blendAlpha=this.blendAlpha),this.depthFunc!==zi&&(n.depthFunc=this.depthFunc),this.depthTest===!1&&(n.depthTest=this.depthTest),this.depthWrite===!1&&(n.depthWrite=this.depthWrite),this.colorWrite===!1&&(n.colorWrite=this.colorWrite),this.stencilWriteMask!==255&&(n.stencilWriteMask=this.stencilWriteMask),this.stencilFunc!==Ol&&(n.stencilFunc=this.stencilFunc),this.stencilRef!==0&&(n.stencilRef=this.stencilRef),this.stencilFuncMask!==255&&(n.stencilFuncMask=this.stencilFuncMask),this.stencilFail!==Oi&&(n.stencilFail=this.stencilFail),this.stencilZFail!==Oi&&(n.stencilZFail=this.stencilZFail),this.stencilZPass!==Oi&&(n.stencilZPass=this.stencilZPass),this.stencilWrite===!0&&(n.stencilWrite=this.stencilWrite),this.rotation!==void 0&&this.rotation!==0&&(n.rotation=this.rotation),this.polygonOffset===!0&&(n.polygonOffset=!0),this.polygonOffsetFactor!==0&&(n.polygonOffsetFactor=this.polygonOffsetFactor),this.polygonOffsetUnits!==0&&(n.polygonOffsetUnits=this.polygonOffsetUnits),this.linewidth!==void 0&&this.linewidth!==1&&(n.linewidth=this.linewidth),this.dashSize!==void 0&&(n.dashSize=this.dashSize),this.gapSize!==void 0&&(n.gapSize=this.gapSize),this.scale!==void 0&&(n.scale=this.scale),this.dithering===!0&&(n.dithering=!0),this.alphaTest>0&&(n.alphaTest=this.alphaTest),this.alphaHash===!0&&(n.alphaHash=!0),this.alphaToCoverage===!0&&(n.alphaToCoverage=!0),this.premultipliedAlpha===!0&&(n.premultipliedAlpha=!0),this.forceSinglePass===!0&&(n.forceSinglePass=!0),this.allowOverride===!1&&(n.allowOverride=!1),this.wireframe===!0&&(n.wireframe=!0),this.wireframeLinewidth>1&&(n.wireframeLinewidth=this.wireframeLinewidth),this.wireframeLinecap!=="round"&&(n.wireframeLinecap=this.wireframeLinecap),this.wireframeLinejoin!=="round"&&(n.wireframeLinejoin=this.wireframeLinejoin),this.flatShading===!0&&(n.flatShading=!0),this.visible===!1&&(n.visible=!1),this.toneMapped===!1&&(n.toneMapped=!1),this.fog===!1&&(n.fog=!1),Object.keys(this.userData).length>0&&(n.userData=this.userData);function s(r){let a=[];for(let o in r){let l=r[o];delete l.metadata,a.push(l)}return a}if(e){let r=s(t.textures),a=s(t.images);r.length>0&&(n.textures=r),a.length>0&&(n.images=a)}return n}clone(){return new this.constructor().copy(this)}copy(t){this.name=t.name,this.blending=t.blending,this.side=t.side,this.vertexColors=t.vertexColors,this.opacity=t.opacity,this.transparent=t.transparent,this.blendSrc=t.blendSrc,this.blendDst=t.blendDst,this.blendEquation=t.blendEquation,this.blendSrcAlpha=t.blendSrcAlpha,this.blendDstAlpha=t.blendDstAlpha,this.blendEquationAlpha=t.blendEquationAlpha,this.blendColor.copy(t.blendColor),this.blendAlpha=t.blendAlpha,this.depthFunc=t.depthFunc,this.depthTest=t.depthTest,this.depthWrite=t.depthWrite,this.stencilWriteMask=t.stencilWriteMask,this.stencilFunc=t.stencilFunc,this.stencilRef=t.stencilRef,this.stencilFuncMask=t.stencilFuncMask,this.stencilFail=t.stencilFail,this.stencilZFail=t.stencilZFail,this.stencilZPass=t.stencilZPass,this.stencilWrite=t.stencilWrite;let e=t.clippingPlanes,n=null;if(e!==null){let s=e.length;n=new Array(s);for(let r=0;r!==s;++r)n[r]=e[r].clone()}return this.clippingPlanes=n,this.clipIntersection=t.clipIntersection,this.clipShadows=t.clipShadows,this.shadowSide=t.shadowSide,this.colorWrite=t.colorWrite,this.precision=t.precision,this.polygonOffset=t.polygonOffset,this.polygonOffsetFactor=t.polygonOffsetFactor,this.polygonOffsetUnits=t.polygonOffsetUnits,this.dithering=t.dithering,this.alphaTest=t.alphaTest,this.alphaHash=t.alphaHash,this.alphaToCoverage=t.alphaToCoverage,this.premultipliedAlpha=t.premultipliedAlpha,this.forceSinglePass=t.forceSinglePass,this.allowOverride=t.allowOverride,this.visible=t.visible,this.toneMapped=t.toneMapped,this.userData=JSON.parse(JSON.stringify(t.userData)),this}dispose(){this.dispatchEvent({type:"dispose"})}set needsUpdate(t){t===!0&&this.version++}};var qn=new k,El=new k,Gr=new k,ci=new k,Tl=new k,Hr=new k,Cl=new k,Js=class{constructor(t=new k,e=new k(0,0,-1)){this.origin=t,this.direction=e}set(t,e){return this.origin.copy(t),this.direction.copy(e),this}copy(t){return this.origin.copy(t.origin),this.direction.copy(t.direction),this}at(t,e){return e.copy(this.origin).addScaledVector(this.direction,t)}lookAt(t){return this.direction.copy(t).sub(this.origin).normalize(),this}recast(t){return this.origin.copy(this.at(t,qn)),this}closestPointToPoint(t,e){e.subVectors(t,this.origin);let n=e.dot(this.direction);return n<0?e.copy(this.origin):e.copy(this.origin).addScaledVector(this.direction,n)}distanceToPoint(t){return Math.sqrt(this.distanceSqToPoint(t))}distanceSqToPoint(t){let e=qn.subVectors(t,this.origin).dot(this.direction);return e<0?this.origin.distanceToSquared(t):(qn.copy(this.origin).addScaledVector(this.direction,e),qn.distanceToSquared(t))}distanceSqToSegment(t,e,n,s){El.copy(t).add(e).multiplyScalar(.5),Gr.copy(e).sub(t).normalize(),ci.copy(this.origin).sub(El);let r=t.distanceTo(e)*.5,a=-this.direction.dot(Gr),o=ci.dot(this.direction),l=-ci.dot(Gr),c=ci.lengthSq(),d=Math.abs(1-a*a),u,h,p,_;if(d>0)if(u=a*l-o,h=a*o-l,_=r*d,u>=0)if(h>=-_)if(h<=_){let y=1/d;u*=y,h*=y,p=u*(u+a*h+2*o)+h*(a*u+h+2*l)+c}else h=r,u=Math.max(0,-(a*h+o)),p=-u*u+h*(h+2*l)+c;else h=-r,u=Math.max(0,-(a*h+o)),p=-u*u+h*(h+2*l)+c;else h<=-_?(u=Math.max(0,-(-a*r+o)),h=u>0?-r:Math.min(Math.max(-r,-l),r),p=-u*u+h*(h+2*l)+c):h<=_?(u=0,h=Math.min(Math.max(-r,-l),r),p=h*(h+2*l)+c):(u=Math.max(0,-(a*r+o)),h=u>0?r:Math.min(Math.max(-r,-l),r),p=-u*u+h*(h+2*l)+c);else h=a>0?-r:r,u=Math.max(0,-(a*h+o)),p=-u*u+h*(h+2*l)+c;return n&&n.copy(this.origin).addScaledVector(this.direction,u),s&&s.copy(El).addScaledVector(Gr,h),p}intersectSphere(t,e){qn.subVectors(t.center,this.origin);let n=qn.dot(this.direction),s=qn.dot(qn)-n*n,r=t.radius*t.radius;if(s>r)return null;let a=Math.sqrt(r-s),o=n-a,l=n+a;return l<0?null:o<0?this.at(l,e):this.at(o,e)}intersectsSphere(t){return t.radius<0?!1:this.distanceSqToPoint(t.center)<=t.radius*t.radius}distanceToPlane(t){let e=t.normal.dot(this.direction);if(e===0)return t.distanceToPoint(this.origin)===0?0:null;let n=-(this.origin.dot(t.normal)+t.constant)/e;return n>=0?n:null}intersectPlane(t,e){let n=this.distanceToPlane(t);return n===null?null:this.at(n,e)}intersectsPlane(t){let e=t.distanceToPoint(this.origin);return e===0||t.normal.dot(this.direction)*e<0}intersectBox(t,e){let n,s,r,a,o,l,c=1/this.direction.x,d=1/this.direction.y,u=1/this.direction.z,h=this.origin;return c>=0?(n=(t.min.x-h.x)*c,s=(t.max.x-h.x)*c):(n=(t.max.x-h.x)*c,s=(t.min.x-h.x)*c),d>=0?(r=(t.min.y-h.y)*d,a=(t.max.y-h.y)*d):(r=(t.max.y-h.y)*d,a=(t.min.y-h.y)*d),n>a||r>s||((r>n||isNaN(n))&&(n=r),(a<s||isNaN(s))&&(s=a),u>=0?(o=(t.min.z-h.z)*u,l=(t.max.z-h.z)*u):(o=(t.max.z-h.z)*u,l=(t.min.z-h.z)*u),n>l||o>s)||((o>n||n!==n)&&(n=o),(l<s||s!==s)&&(s=l),s<0)?null:this.at(n>=0?n:s,e)}intersectsBox(t){return this.intersectBox(t,qn)!==null}intersectTriangle(t,e,n,s,r){Tl.subVectors(e,t),Hr.subVectors(n,t),Cl.crossVectors(Tl,Hr);let a=this.direction.dot(Cl),o;if(a>0){if(s)return null;o=1}else if(a<0)o=-1,a=-a;else return null;ci.subVectors(this.origin,t);let l=o*this.direction.dot(Hr.crossVectors(ci,Hr));if(l<0)return null;let c=o*this.direction.dot(Tl.cross(ci));if(c<0||l+c>a)return null;let d=-o*ci.dot(Cl);return d<0?null:this.at(d/a,r)}applyMatrix4(t){return this.origin.applyMatrix4(t),this.direction.transformDirection(t),this}equals(t){return t.origin.equals(this.origin)&&t.direction.equals(this.direction)}clone(){return new this.constructor().copy(this)}},Ks=class extends Kn{constructor(t){super(),this.isMeshBasicMaterial=!0,this.type="MeshBasicMaterial",this.color=new zt(16777215),this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.specularMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new Jn,this.combine=ql,this.reflectivity=1,this.refractionRatio=.98,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.specularMap=t.specularMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.combine=t.combine,this.reflectivity=t.reflectivity,this.refractionRatio=t.refractionRatio,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.fog=t.fog,this}},xh=new fe,Ui=new Js,Wr=new Vi,yh=new k,$r=new k,Xr=new k,qr=new k,Rl=new k,Yr=new k,bh=new k,Zr=new k,ke=class extends Oe{constructor(t=new cn,e=new Ks){super(),this.isMesh=!0,this.type="Mesh",this.geometry=t,this.material=e,this.morphTargetDictionary=void 0,this.morphTargetInfluences=void 0,this.count=1,this.updateMorphTargets()}copy(t,e){return super.copy(t,e),t.morphTargetInfluences!==void 0&&(this.morphTargetInfluences=t.morphTargetInfluences.slice()),t.morphTargetDictionary!==void 0&&(this.morphTargetDictionary=Object.assign({},t.morphTargetDictionary)),this.material=Array.isArray(t.material)?t.material.slice():t.material,this.geometry=t.geometry,this}updateMorphTargets(){let e=this.geometry.morphAttributes,n=Object.keys(e);if(n.length>0){let s=e[n[0]];if(s!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let r=0,a=s.length;r<a;r++){let o=s[r].name||String(r);this.morphTargetInfluences.push(0),this.morphTargetDictionary[o]=r}}}}getVertexPosition(t,e){let n=this.geometry,s=n.attributes.position,r=n.morphAttributes.position,a=n.morphTargetsRelative;e.fromBufferAttribute(s,t);let o=this.morphTargetInfluences;if(r&&o){Yr.set(0,0,0);for(let l=0,c=r.length;l<c;l++){let d=o[l],u=r[l];d!==0&&(Rl.fromBufferAttribute(u,t),a?Yr.addScaledVector(Rl,d):Yr.addScaledVector(Rl.sub(e),d))}e.add(Yr)}return e}raycast(t,e){let n=this.geometry,s=this.material,r=this.matrixWorld;s!==void 0&&(n.boundingSphere===null&&n.computeBoundingSphere(),Wr.copy(n.boundingSphere),Wr.applyMatrix4(r),Ui.copy(t.ray).recast(t.near),!(Wr.containsPoint(Ui.origin)===!1&&(Ui.intersectSphere(Wr,yh)===null||Ui.origin.distanceToSquared(yh)>(t.far-t.near)**2))&&(xh.copy(r).invert(),Ui.copy(t.ray).applyMatrix4(xh),!(n.boundingBox!==null&&Ui.intersectsBox(n.boundingBox)===!1)&&this._computeIntersections(t,e,Ui)))}_computeIntersections(t,e,n){let s,r=this.geometry,a=this.material,o=r.index,l=r.attributes.position,c=r.attributes.uv,d=r.attributes.uv1,u=r.attributes.normal,h=r.groups,p=r.drawRange;if(o!==null)if(Array.isArray(a))for(let _=0,y=h.length;_<y;_++){let m=h[_],f=a[m.materialIndex],S=Math.max(m.start,p.start),M=Math.min(o.count,Math.min(m.start+m.count,p.start+p.count));for(let E=S,R=M;E<R;E+=3){let w=o.getX(E),I=o.getX(E+1),v=o.getX(E+2);s=Jr(this,f,t,n,c,d,u,w,I,v),s&&(s.faceIndex=Math.floor(E/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let _=Math.max(0,p.start),y=Math.min(o.count,p.start+p.count);for(let m=_,f=y;m<f;m+=3){let S=o.getX(m),M=o.getX(m+1),E=o.getX(m+2);s=Jr(this,a,t,n,c,d,u,S,M,E),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}else if(l!==void 0)if(Array.isArray(a))for(let _=0,y=h.length;_<y;_++){let m=h[_],f=a[m.materialIndex],S=Math.max(m.start,p.start),M=Math.min(l.count,Math.min(m.start+m.count,p.start+p.count));for(let E=S,R=M;E<R;E+=3){let w=E,I=E+1,v=E+2;s=Jr(this,f,t,n,c,d,u,w,I,v),s&&(s.faceIndex=Math.floor(E/3),s.face.materialIndex=m.materialIndex,e.push(s))}}else{let _=Math.max(0,p.start),y=Math.min(l.count,p.start+p.count);for(let m=_,f=y;m<f;m+=3){let S=m,M=m+1,E=m+2;s=Jr(this,a,t,n,c,d,u,S,M,E),s&&(s.faceIndex=Math.floor(m/3),e.push(s))}}}};function Fu(i,t,e,n,s,r,a,o){let l;if(t.side===ze?l=n.intersectTriangle(a,r,s,!0,o):l=n.intersectTriangle(s,r,a,t.side===Zn,o),l===null)return null;Zr.copy(o),Zr.applyMatrix4(i.matrixWorld);let c=e.ray.origin.distanceTo(Zr);return c<e.near||c>e.far?null:{distance:c,point:Zr.clone(),object:i}}function Jr(i,t,e,n,s,r,a,o,l,c){i.getVertexPosition(o,$r),i.getVertexPosition(l,Xr),i.getVertexPosition(c,qr);let d=Fu(i,t,e,n,$r,Xr,qr,bh);if(d){let u=new k;di.getBarycoord(bh,$r,Xr,qr,u),s&&(d.uv=di.getInterpolatedAttribute(s,o,l,c,u,new jt)),r&&(d.uv1=di.getInterpolatedAttribute(r,o,l,c,u,new jt)),a&&(d.normal=di.getInterpolatedAttribute(a,o,l,c,u,new k),d.normal.dot(n.direction)>0&&d.normal.multiplyScalar(-1));let h={a:o,b:l,c,normal:new k,materialIndex:0};di.getNormal($r,Xr,qr,h.normal),d.face=h,d.barycoord=u}return d}var ba=class extends Ge{constructor(t=null,e=1,n=1,s,r,a,o,l,c=Re,d=Re,u,h){super(null,a,o,l,c,d,s,r,u,h),this.isDataTexture=!0,this.image={data:t,width:e,height:n},this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}};var Il=new k,Ou=new k,Bu=new Pt,In=class{constructor(t=new k(1,0,0),e=0){this.isPlane=!0,this.normal=t,this.constant=e}set(t,e){return this.normal.copy(t),this.constant=e,this}setComponents(t,e,n,s){return this.normal.set(t,e,n),this.constant=s,this}setFromNormalAndCoplanarPoint(t,e){return this.normal.copy(t),this.constant=-e.dot(this.normal),this}setFromCoplanarPoints(t,e,n){let s=Il.subVectors(n,e).cross(Ou.subVectors(t,e)).normalize();return this.setFromNormalAndCoplanarPoint(s,t),this}copy(t){return this.normal.copy(t.normal),this.constant=t.constant,this}normalize(){let t=1/this.normal.length();return this.normal.multiplyScalar(t),this.constant*=t,this}negate(){return this.constant*=-1,this.normal.negate(),this}distanceToPoint(t){return this.normal.dot(t)+this.constant}distanceToSphere(t){return this.distanceToPoint(t.center)-t.radius}projectPoint(t,e){return e.copy(t).addScaledVector(this.normal,-this.distanceToPoint(t))}intersectLine(t,e,n=!0){let s=t.delta(Il),r=this.normal.dot(s);if(r===0)return this.distanceToPoint(t.start)===0?e.copy(t.start):null;let a=-(t.start.dot(this.normal)+this.constant)/r;return n===!0&&(a<0||a>1)?null:e.copy(t.start).addScaledVector(s,a)}intersectsLine(t){let e=this.distanceToPoint(t.start),n=this.distanceToPoint(t.end);return e<0&&n>0||n<0&&e>0}intersectsBox(t){return t.intersectsPlane(this)}intersectsSphere(t){return t.intersectsPlane(this)}coplanarPoint(t){return t.copy(this.normal).multiplyScalar(-this.constant)}applyMatrix4(t,e){let n=e||Bu.getNormalMatrix(t),s=this.coplanarPoint(Il).applyMatrix4(t),r=this.normal.applyMatrix3(n).normalize();return this.constant=-s.dot(r),this}translate(t){return this.constant-=t.dot(this.normal),this}equals(t){return t.normal.equals(this.normal)&&t.constant===this.constant}clone(){return new this.constructor().copy(this)}},Fi=new Vi,ku=new jt(.5,.5),Kr=new k,ms=class{constructor(t=new In,e=new In,n=new In,s=new In,r=new In,a=new In){this.planes=[t,e,n,s,r,a]}set(t,e,n,s,r,a){let o=this.planes;return o[0].copy(t),o[1].copy(e),o[2].copy(n),o[3].copy(s),o[4].copy(r),o[5].copy(a),this}copy(t){let e=this.planes;for(let n=0;n<6;n++)e[n].copy(t.planes[n]);return this}setFromProjectionMatrix(t,e=vn,n=!1){let s=this.planes,r=t.elements,a=r[0],o=r[1],l=r[2],c=r[3],d=r[4],u=r[5],h=r[6],p=r[7],_=r[8],y=r[9],m=r[10],f=r[11],S=r[12],M=r[13],E=r[14],R=r[15];if(s[0].setComponents(c-a,p-d,f-_,R-S).normalize(),s[1].setComponents(c+a,p+d,f+_,R+S).normalize(),s[2].setComponents(c+o,p+u,f+y,R+M).normalize(),s[3].setComponents(c-o,p-u,f-y,R-M).normalize(),n)s[4].setComponents(l,h,m,E).normalize(),s[5].setComponents(c-l,p-h,f-m,R-E).normalize();else if(s[4].setComponents(c-l,p-h,f-m,R-E).normalize(),e===vn)s[5].setComponents(c+l,p+h,f+m,R+E).normalize();else if(e===ds)s[5].setComponents(l,h,m,E).normalize();else throw new Error("THREE.Frustum.setFromProjectionMatrix(): Invalid coordinate system: "+e);return this}intersectsObject(t){if(t.boundingSphere!==void 0)t.boundingSphere===null&&t.computeBoundingSphere(),Fi.copy(t.boundingSphere).applyMatrix4(t.matrixWorld);else{let e=t.geometry;e.boundingSphere===null&&e.computeBoundingSphere(),Fi.copy(e.boundingSphere).applyMatrix4(t.matrixWorld)}return this.intersectsSphere(Fi)}intersectsSprite(t){Fi.center.set(0,0,0);let e=ku.distanceTo(t.center);return Fi.radius=.7071067811865476+e,Fi.applyMatrix4(t.matrixWorld),this.intersectsSphere(Fi)}intersectsSphere(t){let e=this.planes,n=t.center,s=-t.radius;for(let r=0;r<6;r++)if(e[r].distanceToPoint(n)<s)return!1;return!0}intersectsBox(t){let e=this.planes;for(let n=0;n<6;n++){let s=e[n];if(Kr.x=s.normal.x>0?t.max.x:t.min.x,Kr.y=s.normal.y>0?t.max.y:t.min.y,Kr.z=s.normal.z>0?t.max.z:t.min.z,s.distanceToPoint(Kr)<0)return!1}return!0}containsPoint(t){let e=this.planes;for(let n=0;n<6;n++)if(e[n].distanceToPoint(t)<0)return!1;return!0}clone(){return new this.constructor().copy(this)}};var js=class extends Kn{constructor(t){super(),this.isLineBasicMaterial=!0,this.type="LineBasicMaterial",this.color=new zt(16777215),this.map=null,this.linewidth=1,this.linecap="round",this.linejoin="round",this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.color.copy(t.color),this.map=t.map,this.linewidth=t.linewidth,this.linecap=t.linecap,this.linejoin=t.linejoin,this.fog=t.fog,this}},Sa=new k,Ma=new k,Sh=new fe,zs=new Js,jr=new Vi,Pl=new k,Mh=new k,wa=class extends Oe{constructor(t=new cn,e=new js){super(),this.isLine=!0,this.type="Line",this.geometry=t,this.material=e,this.morphTargetDictionary=void 0,this.morphTargetInfluences=void 0,this.updateMorphTargets()}copy(t,e){return super.copy(t,e),this.material=Array.isArray(t.material)?t.material.slice():t.material,this.geometry=t.geometry,this}computeLineDistances(){let t=this.geometry;if(t.index===null){let e=t.attributes.position,n=[0];for(let s=1,r=e.count;s<r;s++)Sa.fromBufferAttribute(e,s-1),Ma.fromBufferAttribute(e,s),n[s]=n[s-1],n[s]+=Sa.distanceTo(Ma);t.setAttribute("lineDistance",new Fe(n,1))}else Et("Line.computeLineDistances(): Computation only possible with non-indexed BufferGeometry.");return this}raycast(t,e){let n=this.geometry,s=this.matrixWorld,r=t.params.Line.threshold,a=n.drawRange;if(n.boundingSphere===null&&n.computeBoundingSphere(),jr.copy(n.boundingSphere),jr.applyMatrix4(s),jr.radius+=r,t.ray.intersectsSphere(jr)===!1)return;Sh.copy(s).invert(),zs.copy(t.ray).applyMatrix4(Sh);let o=r/((this.scale.x+this.scale.y+this.scale.z)/3),l=o*o,c=this.isLineSegments?2:1,d=n.index,h=n.attributes.position;if(d!==null){let p=Math.max(0,a.start),_=Math.min(d.count,a.start+a.count);for(let y=p,m=_-1;y<m;y+=c){let f=d.getX(y),S=d.getX(y+1),M=Qr(this,t,zs,l,f,S,y);M&&e.push(M)}if(this.isLineLoop){let y=d.getX(_-1),m=d.getX(p),f=Qr(this,t,zs,l,y,m,_-1);f&&e.push(f)}}else{let p=Math.max(0,a.start),_=Math.min(h.count,a.start+a.count);for(let y=p,m=_-1;y<m;y+=c){let f=Qr(this,t,zs,l,y,y+1,y);f&&e.push(f)}if(this.isLineLoop){let y=Qr(this,t,zs,l,_-1,p,_-1);y&&e.push(y)}}}updateMorphTargets(){let e=this.geometry.morphAttributes,n=Object.keys(e);if(n.length>0){let s=e[n[0]];if(s!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let r=0,a=s.length;r<a;r++){let o=s[r].name||String(r);this.morphTargetInfluences.push(0),this.morphTargetDictionary[o]=r}}}}};function Qr(i,t,e,n,s,r,a){let o=i.geometry.attributes.position;if(Sa.fromBufferAttribute(o,s),Ma.fromBufferAttribute(o,r),e.distanceSqToSegment(Sa,Ma,Pl,Mh)>n)return;Pl.applyMatrix4(i.matrixWorld);let c=t.ray.origin.distanceTo(Pl);if(!(c<t.near||c>t.far))return{distance:c,point:Mh.clone().applyMatrix4(i.matrixWorld),index:a,face:null,faceIndex:null,barycoord:null,object:i}}var wh=new k,Ah=new k,Aa=class extends wa{constructor(t,e){super(t,e),this.isLineSegments=!0,this.type="LineSegments"}computeLineDistances(){let t=this.geometry;if(t.index===null){let e=t.attributes.position,n=[];for(let s=0,r=e.count;s<r;s+=2)wh.fromBufferAttribute(e,s),Ah.fromBufferAttribute(e,s+1),n[s]=s===0?0:n[s-1],n[s+1]=n[s]+wh.distanceTo(Ah);t.setAttribute("lineDistance",new Fe(n,1))}else Et("LineSegments.computeLineDistances(): Computation only possible with non-indexed BufferGeometry.");return this}};var Qs=class extends Ge{constructor(t=[],e=vi,n,s,r,a,o,l,c,d){super(t,e,n,s,r,a,o,l,c,d),this.isCubeTexture=!0,this.flipY=!1}get images(){return this.image}set images(t){this.image=t}};var jn=class extends Ge{constructor(t,e,n=yn,s,r,a,o=Re,l=Re,c,d=Ln,u=1){if(d!==Ln&&d!==yi)throw new Error("DepthTexture format must be either THREE.DepthFormat or THREE.DepthStencilFormat");let h={width:t,height:e,depth:u};super(h,s,r,a,o,l,d,n,c),this.isDepthTexture=!0,this.flipY=!1,this.generateMipmaps=!1,this.compareFunction=null}copy(t){return super.copy(t),this.source=new fs(Object.assign({},t.image)),this.compareFunction=t.compareFunction,this}toJSON(t){let e=super.toJSON(t);return this.compareFunction!==null&&(e.compareFunction=this.compareFunction),e}},Ea=class extends jn{constructor(t,e=yn,n=vi,s,r,a=Re,o=Re,l,c=Ln){let d={width:t,height:t,depth:1},u=[d,d,d,d,d,d];super(t,t,e,n,s,r,a,o,l,c),this.image=u,this.isCubeDepthTexture=!0,this.isCubeTexture=!0}get images(){return this.image}set images(t){this.image=t}},tr=class extends Ge{constructor(t=null){super(),this.sourceTexture=t,this.isExternalTexture=!0}copy(t){return super.copy(t),this.sourceTexture=t.sourceTexture,this}},pi=class i extends cn{constructor(t=1,e=1,n=1,s=1,r=1,a=1){super(),this.type="BoxGeometry",this.parameters={width:t,height:e,depth:n,widthSegments:s,heightSegments:r,depthSegments:a};let o=this;s=Math.floor(s),r=Math.floor(r),a=Math.floor(a);let l=[],c=[],d=[],u=[],h=0,p=0;_("z","y","x",-1,-1,n,e,t,a,r,0),_("z","y","x",1,-1,n,e,-t,a,r,1),_("x","z","y",1,1,t,n,e,s,a,2),_("x","z","y",1,-1,t,n,-e,s,a,3),_("x","y","z",1,-1,t,e,n,s,r,4),_("x","y","z",-1,-1,t,e,-n,s,r,5),this.setIndex(l),this.setAttribute("position",new Fe(c,3)),this.setAttribute("normal",new Fe(d,3)),this.setAttribute("uv",new Fe(u,2));function _(y,m,f,S,M,E,R,w,I,v,A){let D=E/I,T=R/v,O=E/2,H=R/2,$=w/2,U=I+1,L=v+1,B=0,K=0,tt=new k;for(let ct=0;ct<L;ct++){let bt=ct*T-H;for(let vt=0;vt<U;vt++){let Gt=vt*D-O;tt[y]=Gt*S,tt[m]=bt*M,tt[f]=$,c.push(tt.x,tt.y,tt.z),tt[y]=0,tt[m]=0,tt[f]=w>0?1:-1,d.push(tt.x,tt.y,tt.z),u.push(vt/I),u.push(1-ct/v),B+=1}}for(let ct=0;ct<v;ct++)for(let bt=0;bt<I;bt++){let vt=h+bt+U*ct,Gt=h+bt+U*(ct+1),Qt=h+(bt+1)+U*(ct+1),Ut=h+(bt+1)+U*ct;l.push(vt,Gt,Ut),l.push(Gt,Qt,Ut),K+=6}o.addGroup(p,K,A),p+=K,h+=B}}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.width,t.height,t.depth,t.widthSegments,t.heightSegments,t.depthSegments)}};var Gi=class i extends cn{constructor(t=1,e=1,n=1,s=1){super(),this.type="PlaneGeometry",this.parameters={width:t,height:e,widthSegments:n,heightSegments:s};let r=t/2,a=e/2,o=Math.floor(n),l=Math.floor(s),c=o+1,d=l+1,u=t/o,h=e/l,p=[],_=[],y=[],m=[];for(let f=0;f<d;f++){let S=f*h-a;for(let M=0;M<c;M++){let E=M*u-r;_.push(E,-S,0),y.push(0,0,1),m.push(M/o),m.push(1-f/l)}}for(let f=0;f<l;f++)for(let S=0;S<o;S++){let M=S+c*f,E=S+c*(f+1),R=S+1+c*(f+1),w=S+1+c*f;p.push(M,E,w),p.push(E,R,w)}this.setIndex(p),this.setAttribute("position",new Fe(_,3)),this.setAttribute("normal",new Fe(y,3)),this.setAttribute("uv",new Fe(m,2))}copy(t){return super.copy(t),this.parameters=Object.assign({},t.parameters),this}static fromJSON(t){return new i(t.width,t.height,t.widthSegments,t.heightSegments)}};function Wi(i){let t={};for(let e in i){t[e]={};for(let n in i[e]){let s=i[e][n];if(Eh(s))s.isRenderTargetTexture?(Et("UniformsUtils: Textures of render targets cannot be cloned via cloneUniforms() or mergeUniforms()."),t[e][n]=null):t[e][n]=s.clone();else if(Array.isArray(s))if(Eh(s[0])){let r=[];for(let a=0,o=s.length;a<o;a++)r[a]=s[a].clone();t[e][n]=r}else t[e][n]=s.slice();else t[e][n]=s}}return t}function Be(i){let t={};for(let e=0;e<i.length;e++){let n=Wi(i[e]);for(let s in n)t[s]=n[s]}return t}function Eh(i){return i&&(i.isColor||i.isMatrix3||i.isMatrix4||i.isVector2||i.isVector3||i.isVector4||i.isTexture||i.isQuaternion)}function zu(i){let t=[];for(let e=0;e<i.length;e++)t.push(i[e].clone());return t}function dc(i){let t=i.getRenderTarget();return t===null?i.outputColorSpace:t.isXRRenderTarget===!0?t.texture.colorSpace:$t.workingColorSpace}var pd={clone:Wi,merge:Be},Vu=`void main() {
	gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}`,Gu=`void main() {
	gl_FragColor = vec4( 1.0, 0.0, 0.0, 1.0 );
}`,je=class extends Kn{constructor(t){super(),this.isShaderMaterial=!0,this.type="ShaderMaterial",this.defines={},this.uniforms={},this.uniformsGroups=[],this.vertexShader=Vu,this.fragmentShader=Gu,this.linewidth=1,this.wireframe=!1,this.wireframeLinewidth=1,this.fog=!1,this.lights=!1,this.clipping=!1,this.forceSinglePass=!0,this.extensions={clipCullDistance:!1,multiDraw:!1},this.defaultAttributeValues={color:[1,1,1],uv:[0,0],uv1:[0,0]},this.index0AttributeName=void 0,this.uniformsNeedUpdate=!1,this.glslVersion=null,t!==void 0&&this.setValues(t)}copy(t){return super.copy(t),this.fragmentShader=t.fragmentShader,this.vertexShader=t.vertexShader,this.uniforms=Wi(t.uniforms),this.uniformsGroups=zu(t.uniformsGroups),this.defines=Object.assign({},t.defines),this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.fog=t.fog,this.lights=t.lights,this.clipping=t.clipping,this.extensions=Object.assign({},t.extensions),this.glslVersion=t.glslVersion,this.defaultAttributeValues=Object.assign({},t.defaultAttributeValues),this.index0AttributeName=t.index0AttributeName,this.uniformsNeedUpdate=t.uniformsNeedUpdate,this}toJSON(t){let e=super.toJSON(t);e.glslVersion=this.glslVersion,e.uniforms={};for(let s in this.uniforms){let a=this.uniforms[s].value;a&&a.isTexture?e.uniforms[s]={type:"t",value:a.toJSON(t).uuid}:a&&a.isColor?e.uniforms[s]={type:"c",value:a.getHex()}:a&&a.isVector2?e.uniforms[s]={type:"v2",value:a.toArray()}:a&&a.isVector3?e.uniforms[s]={type:"v3",value:a.toArray()}:a&&a.isVector4?e.uniforms[s]={type:"v4",value:a.toArray()}:a&&a.isMatrix3?e.uniforms[s]={type:"m3",value:a.toArray()}:a&&a.isMatrix4?e.uniforms[s]={type:"m4",value:a.toArray()}:e.uniforms[s]={value:a}}Object.keys(this.defines).length>0&&(e.defines=this.defines),e.vertexShader=this.vertexShader,e.fragmentShader=this.fragmentShader,e.lights=this.lights,e.clipping=this.clipping;let n={};for(let s in this.extensions)this.extensions[s]===!0&&(n[s]=!0);return Object.keys(n).length>0&&(e.extensions=n),e}},Ta=class extends je{constructor(t){super(t),this.isRawShaderMaterial=!0,this.type="RawShaderMaterial"}},gs=class extends Kn{constructor(t){super(),this.isMeshStandardMaterial=!0,this.type="MeshStandardMaterial",this.defines={STANDARD:""},this.color=new zt(16777215),this.roughness=1,this.metalness=0,this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.emissive=new zt(0),this.emissiveIntensity=1,this.emissiveMap=null,this.bumpMap=null,this.bumpScale=1,this.normalMap=null,this.normalMapType=Co,this.normalScale=new jt(1,1),this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.roughnessMap=null,this.metalnessMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new Jn,this.envMapIntensity=1,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.flatShading=!1,this.fog=!0,this.setValues(t)}copy(t){return super.copy(t),this.defines={STANDARD:""},this.color.copy(t.color),this.roughness=t.roughness,this.metalness=t.metalness,this.map=t.map,this.lightMap=t.lightMap,this.lightMapIntensity=t.lightMapIntensity,this.aoMap=t.aoMap,this.aoMapIntensity=t.aoMapIntensity,this.emissive.copy(t.emissive),this.emissiveMap=t.emissiveMap,this.emissiveIntensity=t.emissiveIntensity,this.bumpMap=t.bumpMap,this.bumpScale=t.bumpScale,this.normalMap=t.normalMap,this.normalMapType=t.normalMapType,this.normalScale.copy(t.normalScale),this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.roughnessMap=t.roughnessMap,this.metalnessMap=t.metalnessMap,this.alphaMap=t.alphaMap,this.envMap=t.envMap,this.envMapRotation.copy(t.envMapRotation),this.envMapIntensity=t.envMapIntensity,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this.wireframeLinecap=t.wireframeLinecap,this.wireframeLinejoin=t.wireframeLinejoin,this.flatShading=t.flatShading,this.fog=t.fog,this}};var Ca=class extends Kn{constructor(t){super(),this.isMeshDepthMaterial=!0,this.type="MeshDepthMaterial",this.depthPacking=ed,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.wireframe=!1,this.wireframeLinewidth=1,this.setValues(t)}copy(t){return super.copy(t),this.depthPacking=t.depthPacking,this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this.wireframe=t.wireframe,this.wireframeLinewidth=t.wireframeLinewidth,this}},Ra=class extends Kn{constructor(t){super(),this.isMeshDistanceMaterial=!0,this.type="MeshDistanceMaterial",this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.setValues(t)}copy(t){return super.copy(t),this.map=t.map,this.alphaMap=t.alphaMap,this.displacementMap=t.displacementMap,this.displacementScale=t.displacementScale,this.displacementBias=t.displacementBias,this}};function ta(i,t){return!i||i.constructor===t?i:typeof t.BYTES_PER_ELEMENT=="number"?new t(i):Array.prototype.slice.call(i)}var mi=class{constructor(t,e,n,s){this.parameterPositions=t,this._cachedIndex=0,this.resultBuffer=s!==void 0?s:new e.constructor(n),this.sampleValues=e,this.valueSize=n,this.settings=null,this.DefaultSettings_={}}evaluate(t){let e=this.parameterPositions,n=this._cachedIndex,s=e[n],r=e[n-1];n:{t:{let a;e:{i:if(!(t<s)){for(let o=n+2;;){if(s===void 0){if(t<r)break i;return n=e.length,this._cachedIndex=n,this.copySampleValue_(n-1)}if(n===o)break;if(r=s,s=e[++n],t<s)break t}a=e.length;break e}if(!(t>=r)){let o=e[1];t<o&&(n=2,r=o);for(let l=n-2;;){if(r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(n===l)break;if(s=r,r=e[--n-1],t>=r)break t}a=n,n=0;break e}break n}for(;n<a;){let o=n+a>>>1;t<e[o]?a=o:n=o+1}if(s=e[n],r=e[n-1],r===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(s===void 0)return n=e.length,this._cachedIndex=n,this.copySampleValue_(n-1)}this._cachedIndex=n,this.intervalChanged_(n,r,s)}return this.interpolate_(n,r,t,s)}getSettings_(){return this.settings||this.DefaultSettings_}copySampleValue_(t){let e=this.resultBuffer,n=this.sampleValues,s=this.valueSize,r=t*s;for(let a=0;a!==s;++a)e[a]=n[r+a];return e}interpolate_(){throw new Error("call to abstract method")}intervalChanged_(){}},Ia=class extends mi{constructor(t,e,n,s){super(t,e,n,s),this._weightPrev=-0,this._offsetPrev=-0,this._weightNext=-0,this._offsetNext=-0,this.DefaultSettings_={endingStart:Nl,endingEnd:Nl}}intervalChanged_(t,e,n){let s=this.parameterPositions,r=t-2,a=t+1,o=s[r],l=s[a];if(o===void 0)switch(this.getSettings_().endingStart){case Ul:r=t,o=2*e-n;break;case Fl:r=s.length-2,o=e+s[r]-s[r+1];break;default:r=t,o=n}if(l===void 0)switch(this.getSettings_().endingEnd){case Ul:a=t,l=2*n-e;break;case Fl:a=1,l=n+s[1]-s[0];break;default:a=t-1,l=e}let c=(n-e)*.5,d=this.valueSize;this._weightPrev=c/(e-o),this._weightNext=c/(l-n),this._offsetPrev=r*d,this._offsetNext=a*d}interpolate_(t,e,n,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=t*o,c=l-o,d=this._offsetPrev,u=this._offsetNext,h=this._weightPrev,p=this._weightNext,_=(n-e)/(s-e),y=_*_,m=y*_,f=-h*m+2*h*y-h*_,S=(1+h)*m+(-1.5-2*h)*y+(-.5+h)*_+1,M=(-1-p)*m+(1.5+p)*y+.5*_,E=p*m-p*y;for(let R=0;R!==o;++R)r[R]=f*a[d+R]+S*a[c+R]+M*a[l+R]+E*a[u+R];return r}},Pa=class extends mi{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t,e,n,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=t*o,c=l-o,d=(n-e)/(s-e),u=1-d;for(let h=0;h!==o;++h)r[h]=a[c+h]*u+a[l+h]*d;return r}},La=class extends mi{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t){return this.copySampleValue_(t-1)}},Da=class extends mi{interpolate_(t,e,n,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=t*o,c=l-o,d=this.settings||this.DefaultSettings_,u=d.inTangents,h=d.outTangents;if(!u||!h){let y=(n-e)/(s-e),m=1-y;for(let f=0;f!==o;++f)r[f]=a[c+f]*m+a[l+f]*y;return r}let p=o*2,_=t-1;for(let y=0;y!==o;++y){let m=a[c+y],f=a[l+y],S=_*p+y*2,M=h[S],E=h[S+1],R=t*p+y*2,w=u[R],I=u[R+1],v=(n-e)/(s-e),A,D,T,O,H;for(let $=0;$<8;$++){A=v*v,D=A*v,T=1-v,O=T*T,H=O*T;let L=H*e+3*O*v*M+3*T*A*w+D*s-n;if(Math.abs(L)<1e-10)break;let B=3*O*(M-e)+6*T*v*(w-M)+3*A*(s-w);if(Math.abs(B)<1e-10)break;v=v-L/B,v=Math.max(0,Math.min(1,v))}r[y]=H*m+3*O*v*E+3*T*A*I+D*f}return r}},Qe=class{constructor(t,e,n,s){if(t===void 0)throw new Error("THREE.KeyframeTrack: track name is undefined");if(e===void 0||e.length===0)throw new Error("THREE.KeyframeTrack: no keyframes in track named "+t);this.name=t,this.times=ta(e,this.TimeBufferType),this.values=ta(n,this.ValueBufferType),this.setInterpolation(s||this.DefaultInterpolation)}static toJSON(t){let e=t.constructor,n;if(e.toJSON!==this.toJSON)n=e.toJSON(t);else{n={name:t.name,times:ta(t.times,Array),values:ta(t.values,Array)};let s=t.getInterpolation();s!==t.DefaultInterpolation&&(n.interpolation=s)}return n.type=t.ValueTypeName,n}InterpolantFactoryMethodDiscrete(t){return new La(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodLinear(t){return new Pa(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodSmooth(t){return new Ia(this.times,this.values,this.getValueSize(),t)}InterpolantFactoryMethodBezier(t){let e=new Da(this.times,this.values,this.getValueSize(),t);return this.settings&&(e.settings=this.settings),e}setInterpolation(t){let e;switch(t){case Vs:e=this.InterpolantFactoryMethodDiscrete;break;case ma:e=this.InterpolantFactoryMethodLinear;break;case ia:e=this.InterpolantFactoryMethodSmooth;break;case Dl:e=this.InterpolantFactoryMethodBezier;break}if(e===void 0){let n="unsupported interpolation for "+this.ValueTypeName+" keyframe track named "+this.name;if(this.createInterpolant===void 0)if(t!==this.DefaultInterpolation)this.setInterpolation(this.DefaultInterpolation);else throw new Error(n);return Et("KeyframeTrack:",n),this}return this.createInterpolant=e,this}getInterpolation(){switch(this.createInterpolant){case this.InterpolantFactoryMethodDiscrete:return Vs;case this.InterpolantFactoryMethodLinear:return ma;case this.InterpolantFactoryMethodSmooth:return ia;case this.InterpolantFactoryMethodBezier:return Dl}}getValueSize(){return this.values.length/this.times.length}shift(t){if(t!==0){let e=this.times;for(let n=0,s=e.length;n!==s;++n)e[n]+=t}return this}scale(t){if(t!==1){let e=this.times;for(let n=0,s=e.length;n!==s;++n)e[n]*=t}return this}trim(t,e){let n=this.times,s=n.length,r=0,a=s-1;for(;r!==s&&n[r]<t;)++r;for(;a!==-1&&n[a]>e;)--a;if(++a,r!==0||a!==s){r>=a&&(a=Math.max(a,1),r=a-1);let o=this.getValueSize();this.times=n.slice(r,a),this.values=this.values.slice(r*o,a*o)}return this}validate(){let t=!0,e=this.getValueSize();e-Math.floor(e)!==0&&(Ct("KeyframeTrack: Invalid value size in track.",this),t=!1);let n=this.times,s=this.values,r=n.length;r===0&&(Ct("KeyframeTrack: Track is empty.",this),t=!1);let a=null;for(let o=0;o!==r;o++){let l=n[o];if(typeof l=="number"&&isNaN(l)){Ct("KeyframeTrack: Time is not a valid number.",this,o,l),t=!1;break}if(a!==null&&a>l){Ct("KeyframeTrack: Out of order keys.",this,o,l,a),t=!1;break}a=l}if(s!==void 0&&yu(s))for(let o=0,l=s.length;o!==l;++o){let c=s[o];if(isNaN(c)){Ct("KeyframeTrack: Value is not a valid number.",this,o,c),t=!1;break}}return t}optimize(){let t=this.times.slice(),e=this.values.slice(),n=this.getValueSize(),s=this.getInterpolation()===ia,r=t.length-1,a=1;for(let o=1;o<r;++o){let l=!1,c=t[o],d=t[o+1];if(c!==d&&(o!==1||c!==t[0]))if(s)l=!0;else{let u=o*n,h=u-n,p=u+n;for(let _=0;_!==n;++_){let y=e[u+_];if(y!==e[h+_]||y!==e[p+_]){l=!0;break}}}if(l){if(o!==a){t[a]=t[o];let u=o*n,h=a*n;for(let p=0;p!==n;++p)e[h+p]=e[u+p]}++a}}if(r>0){t[a]=t[r];for(let o=r*n,l=a*n,c=0;c!==n;++c)e[l+c]=e[o+c];++a}return a!==t.length?(this.times=t.slice(0,a),this.values=e.slice(0,a*n)):(this.times=t,this.values=e),this}clone(){let t=this.times.slice(),e=this.values.slice(),n=this.constructor,s=new n(this.name,t,e);return s.createInterpolant=this.createInterpolant,s}};Qe.prototype.ValueTypeName="";Qe.prototype.TimeBufferType=Float32Array;Qe.prototype.ValueBufferType=Float32Array;Qe.prototype.DefaultInterpolation=ma;var gi=class extends Qe{constructor(t,e,n){super(t,e,n)}};gi.prototype.ValueTypeName="bool";gi.prototype.ValueBufferType=Array;gi.prototype.DefaultInterpolation=Vs;gi.prototype.InterpolantFactoryMethodLinear=void 0;gi.prototype.InterpolantFactoryMethodSmooth=void 0;var Na=class extends Qe{constructor(t,e,n,s){super(t,e,n,s)}};Na.prototype.ValueTypeName="color";var Ua=class extends Qe{constructor(t,e,n,s){super(t,e,n,s)}};Ua.prototype.ValueTypeName="number";var Fa=class extends mi{constructor(t,e,n,s){super(t,e,n,s)}interpolate_(t,e,n,s){let r=this.resultBuffer,a=this.sampleValues,o=this.valueSize,l=(n-e)/(s-e),c=t*o;for(let d=c+o;c!==d;c+=4)Nn.slerpFlat(r,0,a,c-o,a,c,l);return r}},er=class extends Qe{constructor(t,e,n,s){super(t,e,n,s)}InterpolantFactoryMethodLinear(t){return new Fa(this.times,this.values,this.getValueSize(),t)}};er.prototype.ValueTypeName="quaternion";er.prototype.InterpolantFactoryMethodSmooth=void 0;var _i=class extends Qe{constructor(t,e,n){super(t,e,n)}};_i.prototype.ValueTypeName="string";_i.prototype.ValueBufferType=Array;_i.prototype.DefaultInterpolation=Vs;_i.prototype.InterpolantFactoryMethodLinear=void 0;_i.prototype.InterpolantFactoryMethodSmooth=void 0;var Oa=class extends Qe{constructor(t,e,n,s){super(t,e,n,s)}};Oa.prototype.ValueTypeName="vector";var Ba=class{constructor(t,e,n){let s=this,r=!1,a=0,o=0,l,c=[];this.onStart=void 0,this.onLoad=t,this.onProgress=e,this.onError=n,this._abortController=null,this.itemStart=function(d){o++,r===!1&&s.onStart!==void 0&&s.onStart(d,a,o),r=!0},this.itemEnd=function(d){a++,s.onProgress!==void 0&&s.onProgress(d,a,o),a===o&&(r=!1,s.onLoad!==void 0&&s.onLoad())},this.itemError=function(d){s.onError!==void 0&&s.onError(d)},this.resolveURL=function(d){return l?l(d):d},this.setURLModifier=function(d){return l=d,this},this.addHandler=function(d,u){return c.push(d,u),this},this.removeHandler=function(d){let u=c.indexOf(d);return u!==-1&&c.splice(u,2),this},this.getHandler=function(d){for(let u=0,h=c.length;u<h;u+=2){let p=c[u],_=c[u+1];if(p.global&&(p.lastIndex=0),p.test(d))return _}return null},this.abort=function(){return this.abortController.abort(),this._abortController=null,this}}get abortController(){return this._abortController||(this._abortController=new AbortController),this._abortController}},md=new Ba,ka=class{constructor(t){this.manager=t!==void 0?t:md,this.crossOrigin="anonymous",this.withCredentials=!1,this.path="",this.resourcePath="",this.requestHeader={},typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}load(){}loadAsync(t,e){let n=this;return new Promise(function(s,r){n.load(t,s,e,r)})}parse(){}setCrossOrigin(t){return this.crossOrigin=t,this}setWithCredentials(t){return this.withCredentials=t,this}setPath(t){return this.path=t,this}setResourcePath(t){return this.resourcePath=t,this}setRequestHeader(t){return this.requestHeader=t,this}abort(){return this}};ka.DEFAULT_MATERIAL_NAME="__DEFAULT";var nr=class extends Oe{constructor(t,e=1){super(),this.isLight=!0,this.type="Light",this.color=new zt(t),this.intensity=e}dispose(){this.dispatchEvent({type:"dispose"})}copy(t,e){return super.copy(t,e),this.color.copy(t.color),this.intensity=t.intensity,this}toJSON(t){let e=super.toJSON(t);return e.object.color=this.color.getHex(),e.object.intensity=this.intensity,e}};var Ll=new fe,Th=new k,Ch=new k,kl=class{constructor(t){this.camera=t,this.intensity=1,this.bias=0,this.biasNode=null,this.normalBias=0,this.radius=1,this.blurSamples=8,this.mapSize=new jt(512,512),this.mapType=He,this.map=null,this.mapPass=null,this.matrix=new fe,this.autoUpdate=!0,this.needsUpdate=!1,this._frustum=new ms,this._frameExtents=new jt(1,1),this._viewportCount=1,this._viewports=[new pe(0,0,1,1)]}getViewportCount(){return this._viewportCount}getFrustum(){return this._frustum}updateMatrices(t){let e=this.camera,n=this.matrix;Th.setFromMatrixPosition(t.matrixWorld),e.position.copy(Th),Ch.setFromMatrixPosition(t.target.matrixWorld),e.lookAt(Ch),e.updateMatrixWorld(),Ll.multiplyMatrices(e.projectionMatrix,e.matrixWorldInverse),this._frustum.setFromProjectionMatrix(Ll,e.coordinateSystem,e.reversedDepth),e.coordinateSystem===ds||e.reversedDepth?n.set(.5,0,0,.5,0,.5,0,.5,0,0,1,0,0,0,0,1):n.set(.5,0,0,.5,0,.5,0,.5,0,0,.5,.5,0,0,0,1),n.multiply(Ll)}getViewport(t){return this._viewports[t]}getFrameExtents(){return this._frameExtents}dispose(){this.map&&this.map.dispose(),this.mapPass&&this.mapPass.dispose()}copy(t){return this.camera=t.camera.clone(),this.intensity=t.intensity,this.bias=t.bias,this.radius=t.radius,this.autoUpdate=t.autoUpdate,this.needsUpdate=t.needsUpdate,this.normalBias=t.normalBias,this.blurSamples=t.blurSamples,this.mapSize.copy(t.mapSize),this.biasNode=t.biasNode,this}clone(){return new this.constructor().copy(this)}toJSON(){let t={};return this.intensity!==1&&(t.intensity=this.intensity),this.bias!==0&&(t.bias=this.bias),this.normalBias!==0&&(t.normalBias=this.normalBias),this.radius!==1&&(t.radius=this.radius),(this.mapSize.x!==512||this.mapSize.y!==512)&&(t.mapSize=this.mapSize.toArray()),t.camera=this.camera.toJSON(!1).object,delete t.camera.matrix,t}},ea=new k,na=new Nn,Rn=new k,ir=class extends Oe{constructor(){super(),this.isCamera=!0,this.type="Camera",this.matrixWorldInverse=new fe,this.projectionMatrix=new fe,this.projectionMatrixInverse=new fe,this.coordinateSystem=vn,this._reversedDepth=!1}get reversedDepth(){return this._reversedDepth}copy(t,e){return super.copy(t,e),this.matrixWorldInverse.copy(t.matrixWorldInverse),this.projectionMatrix.copy(t.projectionMatrix),this.projectionMatrixInverse.copy(t.projectionMatrixInverse),this.coordinateSystem=t.coordinateSystem,this}getWorldDirection(t){return super.getWorldDirection(t).negate()}updateMatrixWorld(t){super.updateMatrixWorld(t),this.matrixWorld.decompose(ea,na,Rn),Rn.x===1&&Rn.y===1&&Rn.z===1?this.matrixWorldInverse.copy(this.matrixWorld).invert():this.matrixWorldInverse.compose(ea,na,Rn.set(1,1,1)).invert()}updateWorldMatrix(t,e){super.updateWorldMatrix(t,e),this.matrixWorld.decompose(ea,na,Rn),Rn.x===1&&Rn.y===1&&Rn.z===1?this.matrixWorldInverse.copy(this.matrixWorld).invert():this.matrixWorldInverse.compose(ea,na,Rn.set(1,1,1)).invert()}clone(){return new this.constructor().copy(this)}},hi=new k,Rh=new jt,Ih=new jt,Ue=class extends ir{constructor(t=50,e=1,n=.1,s=2e3){super(),this.isPerspectiveCamera=!0,this.type="PerspectiveCamera",this.fov=t,this.zoom=1,this.near=n,this.far=s,this.focus=10,this.aspect=e,this.view=null,this.filmGauge=35,this.filmOffset=0,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.fov=t.fov,this.zoom=t.zoom,this.near=t.near,this.far=t.far,this.focus=t.focus,this.aspect=t.aspect,this.view=t.view===null?null:Object.assign({},t.view),this.filmGauge=t.filmGauge,this.filmOffset=t.filmOffset,this}setFocalLength(t){let e=.5*this.getFilmHeight()/t;this.fov=_a*2*Math.atan(e),this.updateProjectionMatrix()}getFocalLength(){let t=Math.tan(ll*.5*this.fov);return .5*this.getFilmHeight()/t}getEffectiveFOV(){return _a*2*Math.atan(Math.tan(ll*.5*this.fov)/this.zoom)}getFilmWidth(){return this.filmGauge*Math.min(this.aspect,1)}getFilmHeight(){return this.filmGauge/Math.max(this.aspect,1)}getViewBounds(t,e,n){hi.set(-1,-1,.5).applyMatrix4(this.projectionMatrixInverse),e.set(hi.x,hi.y).multiplyScalar(-t/hi.z),hi.set(1,1,.5).applyMatrix4(this.projectionMatrixInverse),n.set(hi.x,hi.y).multiplyScalar(-t/hi.z)}getViewSize(t,e){return this.getViewBounds(t,Rh,Ih),e.subVectors(Ih,Rh)}setViewOffset(t,e,n,s,r,a){this.aspect=t/e,this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=n,this.view.offsetY=s,this.view.width=r,this.view.height=a,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=this.near,e=t*Math.tan(ll*.5*this.fov)/this.zoom,n=2*e,s=this.aspect*n,r=-.5*s,a=this.view;if(this.view!==null&&this.view.enabled){let l=a.fullWidth,c=a.fullHeight;r+=a.offsetX*s/l,e-=a.offsetY*n/c,s*=a.width/l,n*=a.height/c}let o=this.filmOffset;o!==0&&(r+=t*o/this.getFilmWidth()),this.projectionMatrix.makePerspective(r,r+s,e,e-n,t,this.far,this.coordinateSystem,this.reversedDepth),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.fov=this.fov,e.object.zoom=this.zoom,e.object.near=this.near,e.object.far=this.far,e.object.focus=this.focus,e.object.aspect=this.aspect,this.view!==null&&(e.object.view=Object.assign({},this.view)),e.object.filmGauge=this.filmGauge,e.object.filmOffset=this.filmOffset,e}};var _s=class extends ir{constructor(t=-1,e=1,n=1,s=-1,r=.1,a=2e3){super(),this.isOrthographicCamera=!0,this.type="OrthographicCamera",this.zoom=1,this.view=null,this.left=t,this.right=e,this.top=n,this.bottom=s,this.near=r,this.far=a,this.updateProjectionMatrix()}copy(t,e){return super.copy(t,e),this.left=t.left,this.right=t.right,this.top=t.top,this.bottom=t.bottom,this.near=t.near,this.far=t.far,this.zoom=t.zoom,this.view=t.view===null?null:Object.assign({},t.view),this}setViewOffset(t,e,n,s,r,a){this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=t,this.view.fullHeight=e,this.view.offsetX=n,this.view.offsetY=s,this.view.width=r,this.view.height=a,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){let t=(this.right-this.left)/(2*this.zoom),e=(this.top-this.bottom)/(2*this.zoom),n=(this.right+this.left)/2,s=(this.top+this.bottom)/2,r=n-t,a=n+t,o=s+e,l=s-e;if(this.view!==null&&this.view.enabled){let c=(this.right-this.left)/this.view.fullWidth/this.zoom,d=(this.top-this.bottom)/this.view.fullHeight/this.zoom;r+=c*this.view.offsetX,a=r+c*this.view.width,o-=d*this.view.offsetY,l=o-d*this.view.height}this.projectionMatrix.makeOrthographic(r,a,o,l,this.near,this.far,this.coordinateSystem,this.reversedDepth),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(t){let e=super.toJSON(t);return e.object.zoom=this.zoom,e.object.left=this.left,e.object.right=this.right,e.object.top=this.top,e.object.bottom=this.bottom,e.object.near=this.near,e.object.far=this.far,this.view!==null&&(e.object.view=Object.assign({},this.view)),e}},zl=class extends kl{constructor(){super(new _s(-5,5,5,-5,.5,500)),this.isDirectionalLightShadow=!0}},sr=class extends nr{constructor(t,e){super(t,e),this.isDirectionalLight=!0,this.type="DirectionalLight",this.position.copy(Oe.DEFAULT_UP),this.updateMatrix(),this.target=new Oe,this.shadow=new zl}dispose(){super.dispose(),this.shadow.dispose()}copy(t){return super.copy(t),this.target=t.target.clone(),this.shadow=t.shadow.clone(),this}toJSON(t){let e=super.toJSON(t);return e.object.shadow=this.shadow.toJSON(),e.object.target=this.target.uuid,e}},rr=class extends nr{constructor(t,e){super(t,e),this.isAmbientLight=!0,this.type="AmbientLight"}};var ls=-90,cs=1,za=class extends Oe{constructor(t,e,n){super(),this.type="CubeCamera",this.renderTarget=n,this.coordinateSystem=null,this.activeMipmapLevel=0;let s=new Ue(ls,cs,t,e);s.layers=this.layers,this.add(s);let r=new Ue(ls,cs,t,e);r.layers=this.layers,this.add(r);let a=new Ue(ls,cs,t,e);a.layers=this.layers,this.add(a);let o=new Ue(ls,cs,t,e);o.layers=this.layers,this.add(o);let l=new Ue(ls,cs,t,e);l.layers=this.layers,this.add(l);let c=new Ue(ls,cs,t,e);c.layers=this.layers,this.add(c)}updateCoordinateSystem(){let t=this.coordinateSystem,e=this.children.concat(),[n,s,r,a,o,l]=e;for(let c of e)this.remove(c);if(t===vn)n.up.set(0,1,0),n.lookAt(1,0,0),s.up.set(0,1,0),s.lookAt(-1,0,0),r.up.set(0,0,-1),r.lookAt(0,1,0),a.up.set(0,0,1),a.lookAt(0,-1,0),o.up.set(0,1,0),o.lookAt(0,0,1),l.up.set(0,1,0),l.lookAt(0,0,-1);else if(t===ds)n.up.set(0,-1,0),n.lookAt(-1,0,0),s.up.set(0,-1,0),s.lookAt(1,0,0),r.up.set(0,0,1),r.lookAt(0,1,0),a.up.set(0,0,-1),a.lookAt(0,-1,0),o.up.set(0,-1,0),o.lookAt(0,0,1),l.up.set(0,-1,0),l.lookAt(0,0,-1);else throw new Error("THREE.CubeCamera.updateCoordinateSystem(): Invalid coordinate system: "+t);for(let c of e)this.add(c),c.updateMatrixWorld()}update(t,e){this.parent===null&&this.updateMatrixWorld();let{renderTarget:n,activeMipmapLevel:s}=this;this.coordinateSystem!==t.coordinateSystem&&(this.coordinateSystem=t.coordinateSystem,this.updateCoordinateSystem());let[r,a,o,l,c,d]=this.children,u=t.getRenderTarget(),h=t.getActiveCubeFace(),p=t.getActiveMipmapLevel(),_=t.xr.enabled;t.xr.enabled=!1;let y=n.texture.generateMipmaps;n.texture.generateMipmaps=!1;let m=!1;t.isWebGLRenderer===!0?m=t.state.buffers.depth.getReversed():m=t.reversedDepthBuffer,t.setRenderTarget(n,0,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,r),t.setRenderTarget(n,1,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,a),t.setRenderTarget(n,2,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,o),t.setRenderTarget(n,3,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,l),t.setRenderTarget(n,4,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,c),n.texture.generateMipmaps=y,t.setRenderTarget(n,5,s),m&&t.autoClear===!1&&t.clearDepth(),t.render(e,d),t.setRenderTarget(u,h,p),t.xr.enabled=_,n.texture.needsPMREMUpdate=!0}},Va=class extends Ue{constructor(t=[]){super(),this.isArrayCamera=!0,this.isMultiViewCamera=!1,this.cameras=t}};var uc="\\[\\]\\.:\\/",Hu=new RegExp("["+uc+"]","g"),fc="[^"+uc+"]",Wu="[^"+uc.replace("\\.","")+"]",$u=/((?:WC+[\/:])*)/.source.replace("WC",fc),Xu=/(WCOD+)?/.source.replace("WCOD",Wu),qu=/(?:\.(WC+)(?:\[(.+)\])?)?/.source.replace("WC",fc),Yu=/\.(WC+)(?:\[(.+)\])?/.source.replace("WC",fc),Zu=new RegExp("^"+$u+Xu+qu+Yu+"$"),Ju=["material","materials","bones","map"],Vl=class{constructor(t,e,n){let s=n||he.parseTrackName(e);this._targetGroup=t,this._bindings=t.subscribe_(e,s)}getValue(t,e){this.bind();let n=this._targetGroup.nCachedObjects_,s=this._bindings[n];s!==void 0&&s.getValue(t,e)}setValue(t,e){let n=this._bindings;for(let s=this._targetGroup.nCachedObjects_,r=n.length;s!==r;++s)n[s].setValue(t,e)}bind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,n=t.length;e!==n;++e)t[e].bind()}unbind(){let t=this._bindings;for(let e=this._targetGroup.nCachedObjects_,n=t.length;e!==n;++e)t[e].unbind()}},he=class i{constructor(t,e,n){this.path=e,this.parsedPath=n||i.parseTrackName(e),this.node=i.findNode(t,this.parsedPath.nodeName),this.rootNode=t,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}static create(t,e,n){return t&&t.isAnimationObjectGroup?new i.Composite(t,e,n):new i(t,e,n)}static sanitizeNodeName(t){return t.replace(/\s/g,"_").replace(Hu,"")}static parseTrackName(t){let e=Zu.exec(t);if(e===null)throw new Error("PropertyBinding: Cannot parse trackName: "+t);let n={nodeName:e[2],objectName:e[3],objectIndex:e[4],propertyName:e[5],propertyIndex:e[6]},s=n.nodeName&&n.nodeName.lastIndexOf(".");if(s!==void 0&&s!==-1){let r=n.nodeName.substring(s+1);Ju.indexOf(r)!==-1&&(n.nodeName=n.nodeName.substring(0,s),n.objectName=r)}if(n.propertyName===null||n.propertyName.length===0)throw new Error("PropertyBinding: can not parse propertyName from trackName: "+t);return n}static findNode(t,e){if(e===void 0||e===""||e==="."||e===-1||e===t.name||e===t.uuid)return t;if(t.skeleton){let n=t.skeleton.getBoneByName(e);if(n!==void 0)return n}if(t.children){let n=function(r){for(let a=0;a<r.length;a++){let o=r[a];if(o.name===e||o.uuid===e)return o;let l=n(o.children);if(l)return l}return null},s=n(t.children);if(s)return s}return null}_getValue_unavailable(){}_setValue_unavailable(){}_getValue_direct(t,e){t[e]=this.targetObject[this.propertyName]}_getValue_array(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)t[e++]=n[s]}_getValue_arrayElement(t,e){t[e]=this.resolvedProperty[this.propertyIndex]}_getValue_toArray(t,e){this.resolvedProperty.toArray(t,e)}_setValue_direct(t,e){this.targetObject[this.propertyName]=t[e]}_setValue_direct_setNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.needsUpdate=!0}_setValue_direct_setMatrixWorldNeedsUpdate(t,e){this.targetObject[this.propertyName]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_array(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++]}_setValue_array_setNeedsUpdate(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++];this.targetObject.needsUpdate=!0}_setValue_array_setMatrixWorldNeedsUpdate(t,e){let n=this.resolvedProperty;for(let s=0,r=n.length;s!==r;++s)n[s]=t[e++];this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_arrayElement(t,e){this.resolvedProperty[this.propertyIndex]=t[e]}_setValue_arrayElement_setNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.needsUpdate=!0}_setValue_arrayElement_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty[this.propertyIndex]=t[e],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_fromArray(t,e){this.resolvedProperty.fromArray(t,e)}_setValue_fromArray_setNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.needsUpdate=!0}_setValue_fromArray_setMatrixWorldNeedsUpdate(t,e){this.resolvedProperty.fromArray(t,e),this.targetObject.matrixWorldNeedsUpdate=!0}_getValue_unbound(t,e){this.bind(),this.getValue(t,e)}_setValue_unbound(t,e){this.bind(),this.setValue(t,e)}bind(){let t=this.node,e=this.parsedPath,n=e.objectName,s=e.propertyName,r=e.propertyIndex;if(t||(t=i.findNode(this.rootNode,e.nodeName),this.node=t),this.getValue=this._getValue_unavailable,this.setValue=this._setValue_unavailable,!t){Et("PropertyBinding: No target node found for track: "+this.path+".");return}if(n){let c=e.objectIndex;switch(n){case"materials":if(!t.material){Ct("PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.materials){Ct("PropertyBinding: Can not bind to material.materials as node.material does not have a materials array.",this);return}t=t.material.materials;break;case"bones":if(!t.skeleton){Ct("PropertyBinding: Can not bind to bones as node does not have a skeleton.",this);return}t=t.skeleton.bones;for(let d=0;d<t.length;d++)if(t[d].name===c){c=d;break}break;case"map":if("map"in t){t=t.map;break}if(!t.material){Ct("PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!t.material.map){Ct("PropertyBinding: Can not bind to material.map as node.material does not have a map.",this);return}t=t.material.map;break;default:if(t[n]===void 0){Ct("PropertyBinding: Can not bind to objectName of node undefined.",this);return}t=t[n]}if(c!==void 0){if(t[c]===void 0){Ct("PropertyBinding: Trying to bind to objectIndex of objectName, but is undefined.",this,t);return}t=t[c]}}let a=t[s];if(a===void 0){let c=e.nodeName;Ct("PropertyBinding: Trying to update property for track: "+c+"."+s+" but it wasn't found.",t);return}let o=this.Versioning.None;this.targetObject=t,t.isMaterial===!0?o=this.Versioning.NeedsUpdate:t.isObject3D===!0&&(o=this.Versioning.MatrixWorldNeedsUpdate);let l=this.BindingType.Direct;if(r!==void 0){if(s==="morphTargetInfluences"){if(!t.geometry){Ct("PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.",this);return}if(!t.geometry.morphAttributes){Ct("PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.morphAttributes.",this);return}t.morphTargetDictionary[r]!==void 0&&(r=t.morphTargetDictionary[r])}l=this.BindingType.ArrayElement,this.resolvedProperty=a,this.propertyIndex=r}else a.fromArray!==void 0&&a.toArray!==void 0?(l=this.BindingType.HasFromToArray,this.resolvedProperty=a):Array.isArray(a)?(l=this.BindingType.EntireArray,this.resolvedProperty=a):this.propertyName=s;this.getValue=this.GetterByBindingType[l],this.setValue=this.SetterByBindingTypeAndVersioning[l][o]}unbind(){this.node=null,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}};he.Composite=Vl;he.prototype.BindingType={Direct:0,EntireArray:1,ArrayElement:2,HasFromToArray:3};he.prototype.Versioning={None:0,NeedsUpdate:1,MatrixWorldNeedsUpdate:2};he.prototype.GetterByBindingType=[he.prototype._getValue_direct,he.prototype._getValue_array,he.prototype._getValue_arrayElement,he.prototype._getValue_toArray];he.prototype.SetterByBindingTypeAndVersioning=[[he.prototype._setValue_direct,he.prototype._setValue_direct_setNeedsUpdate,he.prototype._setValue_direct_setMatrixWorldNeedsUpdate],[he.prototype._setValue_array,he.prototype._setValue_array_setNeedsUpdate,he.prototype._setValue_array_setMatrixWorldNeedsUpdate],[he.prototype._setValue_arrayElement,he.prototype._setValue_arrayElement_setNeedsUpdate,he.prototype._setValue_arrayElement_setMatrixWorldNeedsUpdate],[he.prototype._setValue_fromArray,he.prototype._setValue_fromArray_setNeedsUpdate,he.prototype._setValue_fromArray_setMatrixWorldNeedsUpdate]];var Kv=new Float32Array(1);var Gl=class i{static{i.prototype.isMatrix2=!0}constructor(t,e,n,s){this.elements=[1,0,0,1],t!==void 0&&this.set(t,e,n,s)}identity(){return this.set(1,0,0,1),this}fromArray(t,e=0){for(let n=0;n<4;n++)this.elements[n]=t[n+e];return this}set(t,e,n,s){let r=this.elements;return r[0]=t,r[2]=e,r[1]=n,r[3]=s,this}};var ar=class extends Aa{constructor(t=10,e=10,n=4473924,s=8947848){n=new zt(n),s=new zt(s);let r=e/2,a=t/e,o=t/2,l=[],c=[];for(let h=0,p=0,_=-o;h<=e;h++,_+=a){l.push(-o,0,_,o,0,_),l.push(_,0,-o,_,0,o);let y=h===r?n:s;y.toArray(c,p),p+=3,y.toArray(c,p),p+=3,y.toArray(c,p),p+=3,y.toArray(c,p),p+=3}let d=new cn;d.setAttribute("position",new Fe(l,3)),d.setAttribute("color",new Fe(c,3));let u=new js({vertexColors:!0,toneMapped:!1});super(d,u),this.type="GridHelper"}dispose(){this.geometry.dispose(),this.material.dispose()}};function pc(i,t,e,n){let s=Ku(n);switch(e){case ac:return i*t;case lc:return i*t/s.components*s.byteLength;case Ya:return i*t/s.components*s.byteLength;case bi:return i*t*2/s.components*s.byteLength;case Za:return i*t*2/s.components*s.byteLength;case oc:return i*t*3/s.components*s.byteLength;case hn:return i*t*4/s.components*s.byteLength;case Ja:return i*t*4/s.components*s.byteLength;case hr:case dr:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*8;case ur:case fr:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case ja:case to:return Math.max(i,16)*Math.max(t,8)/4;case Ka:case Qa:return Math.max(i,8)*Math.max(t,8)/2;case eo:case no:case so:case ro:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*8;case io:case pr:case ao:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case oo:return Math.floor((i+3)/4)*Math.floor((t+3)/4)*16;case lo:return Math.floor((i+4)/5)*Math.floor((t+3)/4)*16;case co:return Math.floor((i+4)/5)*Math.floor((t+4)/5)*16;case ho:return Math.floor((i+5)/6)*Math.floor((t+4)/5)*16;case uo:return Math.floor((i+5)/6)*Math.floor((t+5)/6)*16;case fo:return Math.floor((i+7)/8)*Math.floor((t+4)/5)*16;case po:return Math.floor((i+7)/8)*Math.floor((t+5)/6)*16;case mo:return Math.floor((i+7)/8)*Math.floor((t+7)/8)*16;case go:return Math.floor((i+9)/10)*Math.floor((t+4)/5)*16;case _o:return Math.floor((i+9)/10)*Math.floor((t+5)/6)*16;case vo:return Math.floor((i+9)/10)*Math.floor((t+7)/8)*16;case xo:return Math.floor((i+9)/10)*Math.floor((t+9)/10)*16;case yo:return Math.floor((i+11)/12)*Math.floor((t+9)/10)*16;case bo:return Math.floor((i+11)/12)*Math.floor((t+11)/12)*16;case So:case Mo:case wo:return Math.ceil(i/4)*Math.ceil(t/4)*16;case Ao:case Eo:return Math.ceil(i/4)*Math.ceil(t/4)*8;case mr:case To:return Math.ceil(i/4)*Math.ceil(t/4)*16}throw new Error(`Unable to determine texture byte length for ${e} format.`)}function Ku(i){switch(i){case He:case nc:return{byteLength:1,components:1};case xs:case ic:case On:return{byteLength:2,components:1};case Xa:case qa:return{byteLength:2,components:4};case yn:case $a:case bn:return{byteLength:4,components:1};case sc:case rc:return{byteLength:4,components:3}}throw new Error(`Unknown texture type ${i}.`)}typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("register",{detail:{revision:"184"}}));typeof window<"u"&&(window.__THREE__?Et("WARNING: Multiple instances of Three.js being imported."):window.__THREE__="184");function Bd(){let i=null,t=!1,e=null,n=null;function s(r,a){e(r,a),n=i.requestAnimationFrame(s)}return{start:function(){t!==!0&&e!==null&&i!==null&&(n=i.requestAnimationFrame(s),t=!0)},stop:function(){i!==null&&i.cancelAnimationFrame(n),t=!1},setAnimationLoop:function(r){e=r},setContext:function(r){i=r}}}function Qu(i){let t=new WeakMap;function e(o,l){let c=o.array,d=o.usage,u=c.byteLength,h=i.createBuffer();i.bindBuffer(l,h),i.bufferData(l,c,d),o.onUploadCallback();let p;if(c instanceof Float32Array)p=i.FLOAT;else if(typeof Float16Array<"u"&&c instanceof Float16Array)p=i.HALF_FLOAT;else if(c instanceof Uint16Array)o.isFloat16BufferAttribute?p=i.HALF_FLOAT:p=i.UNSIGNED_SHORT;else if(c instanceof Int16Array)p=i.SHORT;else if(c instanceof Uint32Array)p=i.UNSIGNED_INT;else if(c instanceof Int32Array)p=i.INT;else if(c instanceof Int8Array)p=i.BYTE;else if(c instanceof Uint8Array)p=i.UNSIGNED_BYTE;else if(c instanceof Uint8ClampedArray)p=i.UNSIGNED_BYTE;else throw new Error("THREE.WebGLAttributes: Unsupported buffer data format: "+c);return{buffer:h,type:p,bytesPerElement:c.BYTES_PER_ELEMENT,version:o.version,size:u}}function n(o,l,c){let d=l.array,u=l.updateRanges;if(i.bindBuffer(c,o),u.length===0)i.bufferSubData(c,0,d);else{u.sort((p,_)=>p.start-_.start);let h=0;for(let p=1;p<u.length;p++){let _=u[h],y=u[p];y.start<=_.start+_.count+1?_.count=Math.max(_.count,y.start+y.count-_.start):(++h,u[h]=y)}u.length=h+1;for(let p=0,_=u.length;p<_;p++){let y=u[p];i.bufferSubData(c,y.start*d.BYTES_PER_ELEMENT,d,y.start,y.count)}l.clearUpdateRanges()}l.onUploadCallback()}function s(o){return o.isInterleavedBufferAttribute&&(o=o.data),t.get(o)}function r(o){o.isInterleavedBufferAttribute&&(o=o.data);let l=t.get(o);l&&(i.deleteBuffer(l.buffer),t.delete(o))}function a(o,l){if(o.isInterleavedBufferAttribute&&(o=o.data),o.isGLBufferAttribute){let d=t.get(o);(!d||d.version<o.version)&&t.set(o,{buffer:o.buffer,type:o.type,bytesPerElement:o.elementSize,version:o.version});return}let c=t.get(o);if(c===void 0)t.set(o,e(o,l));else if(c.version<o.version){if(c.size!==o.array.byteLength)throw new Error("THREE.WebGLAttributes: The size of the buffer attribute's array buffer does not match the original size. Resizing buffer attributes is not supported.");n(c.buffer,o,l),c.version=o.version}}return{get:s,remove:r,update:a}}var tf=`#ifdef USE_ALPHAHASH
	if ( diffuseColor.a < getAlphaHashThreshold( vPosition ) ) discard;
#endif`,ef=`#ifdef USE_ALPHAHASH
	const float ALPHA_HASH_SCALE = 0.05;
	float hash2D( vec2 value ) {
		return fract( 1.0e4 * sin( 17.0 * value.x + 0.1 * value.y ) * ( 0.1 + abs( sin( 13.0 * value.y + value.x ) ) ) );
	}
	float hash3D( vec3 value ) {
		return hash2D( vec2( hash2D( value.xy ), value.z ) );
	}
	float getAlphaHashThreshold( vec3 position ) {
		float maxDeriv = max(
			length( dFdx( position.xyz ) ),
			length( dFdy( position.xyz ) )
		);
		float pixScale = 1.0 / ( ALPHA_HASH_SCALE * maxDeriv );
		vec2 pixScales = vec2(
			exp2( floor( log2( pixScale ) ) ),
			exp2( ceil( log2( pixScale ) ) )
		);
		vec2 alpha = vec2(
			hash3D( floor( pixScales.x * position.xyz ) ),
			hash3D( floor( pixScales.y * position.xyz ) )
		);
		float lerpFactor = fract( log2( pixScale ) );
		float x = ( 1.0 - lerpFactor ) * alpha.x + lerpFactor * alpha.y;
		float a = min( lerpFactor, 1.0 - lerpFactor );
		vec3 cases = vec3(
			x * x / ( 2.0 * a * ( 1.0 - a ) ),
			( x - 0.5 * a ) / ( 1.0 - a ),
			1.0 - ( ( 1.0 - x ) * ( 1.0 - x ) / ( 2.0 * a * ( 1.0 - a ) ) )
		);
		float threshold = ( x < ( 1.0 - a ) )
			? ( ( x < a ) ? cases.x : cases.y )
			: cases.z;
		return clamp( threshold , 1.0e-6, 1.0 );
	}
#endif`,nf=`#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, vAlphaMapUv ).g;
#endif`,sf=`#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,rf=`#ifdef USE_ALPHATEST
	#ifdef ALPHA_TO_COVERAGE
	diffuseColor.a = smoothstep( alphaTest, alphaTest + fwidth( diffuseColor.a ), diffuseColor.a );
	if ( diffuseColor.a == 0.0 ) discard;
	#else
	if ( diffuseColor.a < alphaTest ) discard;
	#endif
#endif`,af=`#ifdef USE_ALPHATEST
	uniform float alphaTest;
#endif`,of=`#ifdef USE_AOMAP
	float ambientOcclusion = ( texture2D( aoMap, vAoMapUv ).r - 1.0 ) * aoMapIntensity + 1.0;
	reflectedLight.indirectDiffuse *= ambientOcclusion;
	#if defined( USE_CLEARCOAT ) 
		clearcoatSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_SHEEN ) 
		sheenSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD )
		float dotNV = saturate( dot( geometryNormal, geometryViewDir ) );
		reflectedLight.indirectSpecular *= computeSpecularOcclusion( dotNV, ambientOcclusion, material.roughness );
	#endif
#endif`,lf=`#ifdef USE_AOMAP
	uniform sampler2D aoMap;
	uniform float aoMapIntensity;
#endif`,cf=`#ifdef USE_BATCHING
	#if ! defined( GL_ANGLE_multi_draw )
	#define gl_DrawID _gl_DrawID
	uniform int _gl_DrawID;
	#endif
	uniform highp sampler2D batchingTexture;
	uniform highp usampler2D batchingIdTexture;
	mat4 getBatchingMatrix( const in float i ) {
		int size = textureSize( batchingTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( batchingTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( batchingTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( batchingTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( batchingTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
	float getIndirectIndex( const in int i ) {
		int size = textureSize( batchingIdTexture, 0 ).x;
		int x = i % size;
		int y = i / size;
		return float( texelFetch( batchingIdTexture, ivec2( x, y ), 0 ).r );
	}
#endif
#ifdef USE_BATCHING_COLOR
	uniform sampler2D batchingColorTexture;
	vec4 getBatchingColor( const in float i ) {
		int size = textureSize( batchingColorTexture, 0 ).x;
		int j = int( i );
		int x = j % size;
		int y = j / size;
		return texelFetch( batchingColorTexture, ivec2( x, y ), 0 );
	}
#endif`,hf=`#ifdef USE_BATCHING
	mat4 batchingMatrix = getBatchingMatrix( getIndirectIndex( gl_DrawID ) );
#endif`,df=`vec3 transformed = vec3( position );
#ifdef USE_ALPHAHASH
	vPosition = vec3( position );
#endif`,uf=`vec3 objectNormal = vec3( normal );
#ifdef USE_TANGENT
	vec3 objectTangent = vec3( tangent.xyz );
#endif`,ff=`float G_BlinnPhong_Implicit( ) {
	return 0.25;
}
float D_BlinnPhong( const in float shininess, const in float dotNH ) {
	return RECIPROCAL_PI * ( shininess * 0.5 + 1.0 ) * pow( dotNH, shininess );
}
vec3 BRDF_BlinnPhong( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in vec3 specularColor, const in float shininess ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( specularColor, 1.0, dotVH );
	float G = G_BlinnPhong_Implicit( );
	float D = D_BlinnPhong( shininess, dotNH );
	return F * ( G * D );
} // validated`,pf=`#ifdef USE_IRIDESCENCE
	const mat3 XYZ_TO_REC709 = mat3(
		 3.2404542, -0.9692660,  0.0556434,
		-1.5371385,  1.8760108, -0.2040259,
		-0.4985314,  0.0415560,  1.0572252
	);
	vec3 Fresnel0ToIor( vec3 fresnel0 ) {
		vec3 sqrtF0 = sqrt( fresnel0 );
		return ( vec3( 1.0 ) + sqrtF0 ) / ( vec3( 1.0 ) - sqrtF0 );
	}
	vec3 IorToFresnel0( vec3 transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - vec3( incidentIor ) ) / ( transmittedIor + vec3( incidentIor ) ) );
	}
	float IorToFresnel0( float transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - incidentIor ) / ( transmittedIor + incidentIor ));
	}
	vec3 evalSensitivity( float OPD, vec3 shift ) {
		float phase = 2.0 * PI * OPD * 1.0e-9;
		vec3 val = vec3( 5.4856e-13, 4.4201e-13, 5.2481e-13 );
		vec3 pos = vec3( 1.6810e+06, 1.7953e+06, 2.2084e+06 );
		vec3 var = vec3( 4.3278e+09, 9.3046e+09, 6.6121e+09 );
		vec3 xyz = val * sqrt( 2.0 * PI * var ) * cos( pos * phase + shift ) * exp( - pow2( phase ) * var );
		xyz.x += 9.7470e-14 * sqrt( 2.0 * PI * 4.5282e+09 ) * cos( 2.2399e+06 * phase + shift[ 0 ] ) * exp( - 4.5282e+09 * pow2( phase ) );
		xyz /= 1.0685e-7;
		vec3 rgb = XYZ_TO_REC709 * xyz;
		return rgb;
	}
	vec3 evalIridescence( float outsideIOR, float eta2, float cosTheta1, float thinFilmThickness, vec3 baseF0 ) {
		vec3 I;
		float iridescenceIOR = mix( outsideIOR, eta2, smoothstep( 0.0, 0.03, thinFilmThickness ) );
		float sinTheta2Sq = pow2( outsideIOR / iridescenceIOR ) * ( 1.0 - pow2( cosTheta1 ) );
		float cosTheta2Sq = 1.0 - sinTheta2Sq;
		if ( cosTheta2Sq < 0.0 ) {
			return vec3( 1.0 );
		}
		float cosTheta2 = sqrt( cosTheta2Sq );
		float R0 = IorToFresnel0( iridescenceIOR, outsideIOR );
		float R12 = F_Schlick( R0, 1.0, cosTheta1 );
		float T121 = 1.0 - R12;
		float phi12 = 0.0;
		if ( iridescenceIOR < outsideIOR ) phi12 = PI;
		float phi21 = PI - phi12;
		vec3 baseIOR = Fresnel0ToIor( clamp( baseF0, 0.0, 0.9999 ) );		vec3 R1 = IorToFresnel0( baseIOR, iridescenceIOR );
		vec3 R23 = F_Schlick( R1, 1.0, cosTheta2 );
		vec3 phi23 = vec3( 0.0 );
		if ( baseIOR[ 0 ] < iridescenceIOR ) phi23[ 0 ] = PI;
		if ( baseIOR[ 1 ] < iridescenceIOR ) phi23[ 1 ] = PI;
		if ( baseIOR[ 2 ] < iridescenceIOR ) phi23[ 2 ] = PI;
		float OPD = 2.0 * iridescenceIOR * thinFilmThickness * cosTheta2;
		vec3 phi = vec3( phi21 ) + phi23;
		vec3 R123 = clamp( R12 * R23, 1e-5, 0.9999 );
		vec3 r123 = sqrt( R123 );
		vec3 Rs = pow2( T121 ) * R23 / ( vec3( 1.0 ) - R123 );
		vec3 C0 = R12 + Rs;
		I = C0;
		vec3 Cm = Rs - T121;
		for ( int m = 1; m <= 2; ++ m ) {
			Cm *= r123;
			vec3 Sm = 2.0 * evalSensitivity( float( m ) * OPD, float( m ) * phi );
			I += Cm * Sm;
		}
		return max( I, vec3( 0.0 ) );
	}
#endif`,mf=`#ifdef USE_BUMPMAP
	uniform sampler2D bumpMap;
	uniform float bumpScale;
	vec2 dHdxy_fwd() {
		vec2 dSTdx = dFdx( vBumpMapUv );
		vec2 dSTdy = dFdy( vBumpMapUv );
		float Hll = bumpScale * texture2D( bumpMap, vBumpMapUv ).x;
		float dBx = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdx ).x - Hll;
		float dBy = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdy ).x - Hll;
		return vec2( dBx, dBy );
	}
	vec3 perturbNormalArb( vec3 surf_pos, vec3 surf_norm, vec2 dHdxy, float faceDirection ) {
		vec3 vSigmaX = normalize( dFdx( surf_pos.xyz ) );
		vec3 vSigmaY = normalize( dFdy( surf_pos.xyz ) );
		vec3 vN = surf_norm;
		vec3 R1 = cross( vSigmaY, vN );
		vec3 R2 = cross( vN, vSigmaX );
		float fDet = dot( vSigmaX, R1 ) * faceDirection;
		vec3 vGrad = sign( fDet ) * ( dHdxy.x * R1 + dHdxy.y * R2 );
		return normalize( abs( fDet ) * surf_norm - vGrad );
	}
#endif`,gf=`#if NUM_CLIPPING_PLANES > 0
	vec4 plane;
	#ifdef ALPHA_TO_COVERAGE
		float distanceToPlane, distanceGradient;
		float clipOpacity = 1.0;
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
			distanceGradient = fwidth( distanceToPlane ) / 2.0;
			clipOpacity *= smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			if ( clipOpacity == 0.0 ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			float unionClipOpacity = 1.0;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
				distanceGradient = fwidth( distanceToPlane ) / 2.0;
				unionClipOpacity *= 1.0 - smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			}
			#pragma unroll_loop_end
			clipOpacity *= 1.0 - unionClipOpacity;
		#endif
		diffuseColor.a *= clipOpacity;
		if ( diffuseColor.a == 0.0 ) discard;
	#else
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			if ( dot( vClipPosition, plane.xyz ) > plane.w ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			bool clipped = true;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				clipped = ( dot( vClipPosition, plane.xyz ) > plane.w ) && clipped;
			}
			#pragma unroll_loop_end
			if ( clipped ) discard;
		#endif
	#endif
#endif`,_f=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
	uniform vec4 clippingPlanes[ NUM_CLIPPING_PLANES ];
#endif`,vf=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
#endif`,xf=`#if NUM_CLIPPING_PLANES > 0
	vClipPosition = - mvPosition.xyz;
#endif`,yf=`#if defined( USE_COLOR ) || defined( USE_COLOR_ALPHA )
	diffuseColor *= vColor;
#endif`,bf=`#if defined( USE_COLOR ) || defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#endif`,Sf=`#if defined( USE_COLOR ) || defined( USE_COLOR_ALPHA ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	varying vec4 vColor;
#endif`,Mf=`#if defined( USE_COLOR ) || defined( USE_COLOR_ALPHA ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	vColor = vec4( 1.0 );
#endif
#ifdef USE_COLOR_ALPHA
	vColor *= color;
#elif defined( USE_COLOR )
	vColor.rgb *= color;
#endif
#ifdef USE_INSTANCING_COLOR
	vColor.rgb *= instanceColor.rgb;
#endif
#ifdef USE_BATCHING_COLOR
	vColor *= getBatchingColor( getIndirectIndex( gl_DrawID ) );
#endif`,wf=`#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6
#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
#define whiteComplement( a ) ( 1.0 - saturate( a ) )
float pow2( const in float x ) { return x*x; }
vec3 pow2( const in vec3 x ) { return x*x; }
float pow3( const in float x ) { return x*x*x; }
float pow4( const in float x ) { float x2 = x*x; return x2*x2; }
float max3( const in vec3 v ) { return max( max( v.x, v.y ), v.z ); }
float average( const in vec3 v ) { return dot( v, vec3( 0.3333333 ) ); }
highp float rand( const in vec2 uv ) {
	const highp float a = 12.9898, b = 78.233, c = 43758.5453;
	highp float dt = dot( uv.xy, vec2( a,b ) ), sn = mod( dt, PI );
	return fract( sin( sn ) * c );
}
#ifdef HIGH_PRECISION
	float precisionSafeLength( vec3 v ) { return length( v ); }
#else
	float precisionSafeLength( vec3 v ) {
		float maxComponent = max3( abs( v ) );
		return length( v / maxComponent ) * maxComponent;
	}
#endif
struct IncidentLight {
	vec3 color;
	vec3 direction;
	bool visible;
};
struct ReflectedLight {
	vec3 directDiffuse;
	vec3 directSpecular;
	vec3 indirectDiffuse;
	vec3 indirectSpecular;
};
#ifdef USE_ALPHAHASH
	varying vec3 vPosition;
#endif
vec3 transformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );
}
vec3 inverseTransformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( vec4( dir, 0.0 ) * matrix ).xyz );
}
bool isPerspectiveMatrix( mat4 m ) {
	return m[ 2 ][ 3 ] == - 1.0;
}
vec2 equirectUv( in vec3 dir ) {
	float u = atan( dir.z, dir.x ) * RECIPROCAL_PI2 + 0.5;
	float v = asin( clamp( dir.y, - 1.0, 1.0 ) ) * RECIPROCAL_PI + 0.5;
	return vec2( u, v );
}
vec3 BRDF_Lambert( const in vec3 diffuseColor ) {
	return RECIPROCAL_PI * diffuseColor;
}
vec3 F_Schlick( const in vec3 f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
}
float F_Schlick( const in float f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
} // validated`,Af=`#ifdef ENVMAP_TYPE_CUBE_UV
	#define cubeUV_minMipLevel 4.0
	#define cubeUV_minTileSize 16.0
	float getFace( vec3 direction ) {
		vec3 absDirection = abs( direction );
		float face = - 1.0;
		if ( absDirection.x > absDirection.z ) {
			if ( absDirection.x > absDirection.y )
				face = direction.x > 0.0 ? 0.0 : 3.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		} else {
			if ( absDirection.z > absDirection.y )
				face = direction.z > 0.0 ? 2.0 : 5.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		}
		return face;
	}
	vec2 getUV( vec3 direction, float face ) {
		vec2 uv;
		if ( face == 0.0 ) {
			uv = vec2( direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 1.0 ) {
			uv = vec2( - direction.x, - direction.z ) / abs( direction.y );
		} else if ( face == 2.0 ) {
			uv = vec2( - direction.x, direction.y ) / abs( direction.z );
		} else if ( face == 3.0 ) {
			uv = vec2( - direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 4.0 ) {
			uv = vec2( - direction.x, direction.z ) / abs( direction.y );
		} else {
			uv = vec2( direction.x, direction.y ) / abs( direction.z );
		}
		return 0.5 * ( uv + 1.0 );
	}
	vec3 bilinearCubeUV( sampler2D envMap, vec3 direction, float mipInt ) {
		float face = getFace( direction );
		float filterInt = max( cubeUV_minMipLevel - mipInt, 0.0 );
		mipInt = max( mipInt, cubeUV_minMipLevel );
		float faceSize = exp2( mipInt );
		highp vec2 uv = getUV( direction, face ) * ( faceSize - 2.0 ) + 1.0;
		if ( face > 2.0 ) {
			uv.y += faceSize;
			face -= 3.0;
		}
		uv.x += face * faceSize;
		uv.x += filterInt * 3.0 * cubeUV_minTileSize;
		uv.y += 4.0 * ( exp2( CUBEUV_MAX_MIP ) - faceSize );
		uv.x *= CUBEUV_TEXEL_WIDTH;
		uv.y *= CUBEUV_TEXEL_HEIGHT;
		#ifdef texture2DGradEXT
			return texture2DGradEXT( envMap, uv, vec2( 0.0 ), vec2( 0.0 ) ).rgb;
		#else
			return texture2D( envMap, uv ).rgb;
		#endif
	}
	#define cubeUV_r0 1.0
	#define cubeUV_m0 - 2.0
	#define cubeUV_r1 0.8
	#define cubeUV_m1 - 1.0
	#define cubeUV_r4 0.4
	#define cubeUV_m4 2.0
	#define cubeUV_r5 0.305
	#define cubeUV_m5 3.0
	#define cubeUV_r6 0.21
	#define cubeUV_m6 4.0
	float roughnessToMip( float roughness ) {
		float mip = 0.0;
		if ( roughness >= cubeUV_r1 ) {
			mip = ( cubeUV_r0 - roughness ) * ( cubeUV_m1 - cubeUV_m0 ) / ( cubeUV_r0 - cubeUV_r1 ) + cubeUV_m0;
		} else if ( roughness >= cubeUV_r4 ) {
			mip = ( cubeUV_r1 - roughness ) * ( cubeUV_m4 - cubeUV_m1 ) / ( cubeUV_r1 - cubeUV_r4 ) + cubeUV_m1;
		} else if ( roughness >= cubeUV_r5 ) {
			mip = ( cubeUV_r4 - roughness ) * ( cubeUV_m5 - cubeUV_m4 ) / ( cubeUV_r4 - cubeUV_r5 ) + cubeUV_m4;
		} else if ( roughness >= cubeUV_r6 ) {
			mip = ( cubeUV_r5 - roughness ) * ( cubeUV_m6 - cubeUV_m5 ) / ( cubeUV_r5 - cubeUV_r6 ) + cubeUV_m5;
		} else {
			mip = - 2.0 * log2( 1.16 * roughness );		}
		return mip;
	}
	vec4 textureCubeUV( sampler2D envMap, vec3 sampleDir, float roughness ) {
		float mip = clamp( roughnessToMip( roughness ), cubeUV_m0, CUBEUV_MAX_MIP );
		float mipF = fract( mip );
		float mipInt = floor( mip );
		vec3 color0 = bilinearCubeUV( envMap, sampleDir, mipInt );
		if ( mipF == 0.0 ) {
			return vec4( color0, 1.0 );
		} else {
			vec3 color1 = bilinearCubeUV( envMap, sampleDir, mipInt + 1.0 );
			return vec4( mix( color0, color1, mipF ), 1.0 );
		}
	}
#endif`,Ef=`vec3 transformedNormal = objectNormal;
#ifdef USE_TANGENT
	vec3 transformedTangent = objectTangent;
#endif
#ifdef USE_BATCHING
	mat3 bm = mat3( batchingMatrix );
	transformedNormal /= vec3( dot( bm[ 0 ], bm[ 0 ] ), dot( bm[ 1 ], bm[ 1 ] ), dot( bm[ 2 ], bm[ 2 ] ) );
	transformedNormal = bm * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = bm * transformedTangent;
	#endif
#endif
#ifdef USE_INSTANCING
	mat3 im = mat3( instanceMatrix );
	transformedNormal /= vec3( dot( im[ 0 ], im[ 0 ] ), dot( im[ 1 ], im[ 1 ] ), dot( im[ 2 ], im[ 2 ] ) );
	transformedNormal = im * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = im * transformedTangent;
	#endif
#endif
transformedNormal = normalMatrix * transformedNormal;
#ifdef FLIP_SIDED
	transformedNormal = - transformedNormal;
#endif
#ifdef USE_TANGENT
	transformedTangent = ( modelViewMatrix * vec4( transformedTangent, 0.0 ) ).xyz;
	#ifdef FLIP_SIDED
		transformedTangent = - transformedTangent;
	#endif
#endif`,Tf=`#ifdef USE_DISPLACEMENTMAP
	uniform sampler2D displacementMap;
	uniform float displacementScale;
	uniform float displacementBias;
#endif`,Cf=`#ifdef USE_DISPLACEMENTMAP
	transformed += normalize( objectNormal ) * ( texture2D( displacementMap, vDisplacementMapUv ).x * displacementScale + displacementBias );
#endif`,Rf=`#ifdef USE_EMISSIVEMAP
	vec4 emissiveColor = texture2D( emissiveMap, vEmissiveMapUv );
	#ifdef DECODE_VIDEO_TEXTURE_EMISSIVE
		emissiveColor = sRGBTransferEOTF( emissiveColor );
	#endif
	totalEmissiveRadiance *= emissiveColor.rgb;
#endif`,If=`#ifdef USE_EMISSIVEMAP
	uniform sampler2D emissiveMap;
#endif`,Pf="gl_FragColor = linearToOutputTexel( gl_FragColor );",Lf=`vec4 LinearTransferOETF( in vec4 value ) {
	return value;
}
vec4 sRGBTransferEOTF( in vec4 value ) {
	return vec4( mix( pow( value.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), value.rgb * 0.0773993808, vec3( lessThanEqual( value.rgb, vec3( 0.04045 ) ) ) ), value.a );
}
vec4 sRGBTransferOETF( in vec4 value ) {
	return vec4( mix( pow( value.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 ), value.rgb * 12.92, vec3( lessThanEqual( value.rgb, vec3( 0.0031308 ) ) ) ), value.a );
}`,Df=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vec3 cameraToFrag;
		if ( isOrthographic ) {
			cameraToFrag = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToFrag = normalize( vWorldPosition - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vec3 reflectVec = reflect( cameraToFrag, worldNormal );
		#else
			vec3 reflectVec = refract( cameraToFrag, worldNormal, refractionRatio );
		#endif
	#else
		vec3 reflectVec = vReflect;
	#endif
	#ifdef ENVMAP_TYPE_CUBE
		vec4 envColor = textureCube( envMap, envMapRotation * reflectVec );
		#ifdef ENVMAP_BLENDING_MULTIPLY
			outgoingLight = mix( outgoingLight, outgoingLight * envColor.xyz, specularStrength * reflectivity );
		#elif defined( ENVMAP_BLENDING_MIX )
			outgoingLight = mix( outgoingLight, envColor.xyz, specularStrength * reflectivity );
		#elif defined( ENVMAP_BLENDING_ADD )
			outgoingLight += envColor.xyz * specularStrength * reflectivity;
		#endif
	#endif
#endif`,Nf=`#ifdef USE_ENVMAP
	uniform float envMapIntensity;
	uniform mat3 envMapRotation;
	#ifdef ENVMAP_TYPE_CUBE
		uniform samplerCube envMap;
	#else
		uniform sampler2D envMap;
	#endif
#endif`,Uf=`#ifdef USE_ENVMAP
	uniform float reflectivity;
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		varying vec3 vWorldPosition;
		uniform float refractionRatio;
	#else
		varying vec3 vReflect;
	#endif
#endif`,Ff=`#ifdef USE_ENVMAP
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		
		varying vec3 vWorldPosition;
	#else
		varying vec3 vReflect;
		uniform float refractionRatio;
	#endif
#endif`,Of=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vWorldPosition = worldPosition.xyz;
	#else
		vec3 cameraToVertex;
		if ( isOrthographic ) {
			cameraToVertex = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToVertex = normalize( worldPosition.xyz - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vReflect = reflect( cameraToVertex, worldNormal );
		#else
			vReflect = refract( cameraToVertex, worldNormal, refractionRatio );
		#endif
	#endif
#endif`,Bf=`#ifdef USE_FOG
	vFogDepth = - mvPosition.z;
#endif`,kf=`#ifdef USE_FOG
	varying float vFogDepth;
#endif`,zf=`#ifdef USE_FOG
	#ifdef FOG_EXP2
		float fogFactor = 1.0 - exp( - fogDensity * fogDensity * vFogDepth * vFogDepth );
	#else
		float fogFactor = smoothstep( fogNear, fogFar, vFogDepth );
	#endif
	gl_FragColor.rgb = mix( gl_FragColor.rgb, fogColor, fogFactor );
#endif`,Vf=`#ifdef USE_FOG
	uniform vec3 fogColor;
	varying float vFogDepth;
	#ifdef FOG_EXP2
		uniform float fogDensity;
	#else
		uniform float fogNear;
		uniform float fogFar;
	#endif
#endif`,Gf=`#ifdef USE_GRADIENTMAP
	uniform sampler2D gradientMap;
#endif
vec3 getGradientIrradiance( vec3 normal, vec3 lightDirection ) {
	float dotNL = dot( normal, lightDirection );
	vec2 coord = vec2( dotNL * 0.5 + 0.5, 0.0 );
	#ifdef USE_GRADIENTMAP
		return vec3( texture2D( gradientMap, coord ).r );
	#else
		vec2 fw = fwidth( coord ) * 0.5;
		return mix( vec3( 0.7 ), vec3( 1.0 ), smoothstep( 0.7 - fw.x, 0.7 + fw.x, coord.x ) );
	#endif
}`,Hf=`#ifdef USE_LIGHTMAP
	uniform sampler2D lightMap;
	uniform float lightMapIntensity;
#endif`,Wf=`LambertMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularStrength = specularStrength;`,$f=`varying vec3 vViewPosition;
struct LambertMaterial {
	vec3 diffuseColor;
	float specularStrength;
};
void RE_Direct_Lambert( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Lambert( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Lambert
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Lambert`,Xf=`uniform bool receiveShadow;
uniform vec3 ambientLightColor;
#if defined( USE_LIGHT_PROBES )
	uniform vec3 lightProbe[ 9 ];
#endif
vec3 shGetIrradianceAt( in vec3 normal, in vec3 shCoefficients[ 9 ] ) {
	float x = normal.x, y = normal.y, z = normal.z;
	vec3 result = shCoefficients[ 0 ] * 0.886227;
	result += shCoefficients[ 1 ] * 2.0 * 0.511664 * y;
	result += shCoefficients[ 2 ] * 2.0 * 0.511664 * z;
	result += shCoefficients[ 3 ] * 2.0 * 0.511664 * x;
	result += shCoefficients[ 4 ] * 2.0 * 0.429043 * x * y;
	result += shCoefficients[ 5 ] * 2.0 * 0.429043 * y * z;
	result += shCoefficients[ 6 ] * ( 0.743125 * z * z - 0.247708 );
	result += shCoefficients[ 7 ] * 2.0 * 0.429043 * x * z;
	result += shCoefficients[ 8 ] * 0.429043 * ( x * x - y * y );
	return result;
}
vec3 getLightProbeIrradiance( const in vec3 lightProbe[ 9 ], const in vec3 normal ) {
	vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
	vec3 irradiance = shGetIrradianceAt( worldNormal, lightProbe );
	return irradiance;
}
vec3 getAmbientLightIrradiance( const in vec3 ambientLightColor ) {
	vec3 irradiance = ambientLightColor;
	return irradiance;
}
float getDistanceAttenuation( const in float lightDistance, const in float cutoffDistance, const in float decayExponent ) {
	float distanceFalloff = 1.0 / max( pow( lightDistance, decayExponent ), 0.01 );
	if ( cutoffDistance > 0.0 ) {
		distanceFalloff *= pow2( saturate( 1.0 - pow4( lightDistance / cutoffDistance ) ) );
	}
	return distanceFalloff;
}
float getSpotAttenuation( const in float coneCosine, const in float penumbraCosine, const in float angleCosine ) {
	return smoothstep( coneCosine, penumbraCosine, angleCosine );
}
#if NUM_DIR_LIGHTS > 0
	struct DirectionalLight {
		vec3 direction;
		vec3 color;
	};
	uniform DirectionalLight directionalLights[ NUM_DIR_LIGHTS ];
	void getDirectionalLightInfo( const in DirectionalLight directionalLight, out IncidentLight light ) {
		light.color = directionalLight.color;
		light.direction = directionalLight.direction;
		light.visible = true;
	}
#endif
#if NUM_POINT_LIGHTS > 0
	struct PointLight {
		vec3 position;
		vec3 color;
		float distance;
		float decay;
	};
	uniform PointLight pointLights[ NUM_POINT_LIGHTS ];
	void getPointLightInfo( const in PointLight pointLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = pointLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float lightDistance = length( lVector );
		light.color = pointLight.color;
		light.color *= getDistanceAttenuation( lightDistance, pointLight.distance, pointLight.decay );
		light.visible = ( light.color != vec3( 0.0 ) );
	}
#endif
#if NUM_SPOT_LIGHTS > 0
	struct SpotLight {
		vec3 position;
		vec3 direction;
		vec3 color;
		float distance;
		float decay;
		float coneCos;
		float penumbraCos;
	};
	uniform SpotLight spotLights[ NUM_SPOT_LIGHTS ];
	void getSpotLightInfo( const in SpotLight spotLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = spotLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float angleCos = dot( light.direction, spotLight.direction );
		float spotAttenuation = getSpotAttenuation( spotLight.coneCos, spotLight.penumbraCos, angleCos );
		if ( spotAttenuation > 0.0 ) {
			float lightDistance = length( lVector );
			light.color = spotLight.color * spotAttenuation;
			light.color *= getDistanceAttenuation( lightDistance, spotLight.distance, spotLight.decay );
			light.visible = ( light.color != vec3( 0.0 ) );
		} else {
			light.color = vec3( 0.0 );
			light.visible = false;
		}
	}
#endif
#if NUM_RECT_AREA_LIGHTS > 0
	struct RectAreaLight {
		vec3 color;
		vec3 position;
		vec3 halfWidth;
		vec3 halfHeight;
	};
	uniform sampler2D ltc_1;	uniform sampler2D ltc_2;
	uniform RectAreaLight rectAreaLights[ NUM_RECT_AREA_LIGHTS ];
#endif
#if NUM_HEMI_LIGHTS > 0
	struct HemisphereLight {
		vec3 direction;
		vec3 skyColor;
		vec3 groundColor;
	};
	uniform HemisphereLight hemisphereLights[ NUM_HEMI_LIGHTS ];
	vec3 getHemisphereLightIrradiance( const in HemisphereLight hemiLight, const in vec3 normal ) {
		float dotNL = dot( normal, hemiLight.direction );
		float hemiDiffuseWeight = 0.5 * dotNL + 0.5;
		vec3 irradiance = mix( hemiLight.groundColor, hemiLight.skyColor, hemiDiffuseWeight );
		return irradiance;
	}
#endif
#include <lightprobes_pars_fragment>`,qf=`#ifdef USE_ENVMAP
	vec3 getIBLIrradiance( const in vec3 normal ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * worldNormal, 1.0 );
			return PI * envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	vec3 getIBLRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 reflectVec = reflect( - viewDir, normal );
			reflectVec = normalize( mix( reflectVec, normal, pow4( roughness ) ) );
			reflectVec = inverseTransformDirection( reflectVec, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * reflectVec, roughness );
			return envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	#ifdef USE_ANISOTROPY
		vec3 getIBLAnisotropyRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness, const in vec3 bitangent, const in float anisotropy ) {
			#ifdef ENVMAP_TYPE_CUBE_UV
				vec3 bentNormal = cross( bitangent, viewDir );
				bentNormal = normalize( cross( bentNormal, bitangent ) );
				bentNormal = normalize( mix( bentNormal, normal, pow2( pow2( 1.0 - anisotropy * ( 1.0 - roughness ) ) ) ) );
				return getIBLRadiance( viewDir, bentNormal, roughness );
			#else
				return vec3( 0.0 );
			#endif
		}
	#endif
#endif`,Yf=`ToonMaterial material;
material.diffuseColor = diffuseColor.rgb;`,Zf=`varying vec3 vViewPosition;
struct ToonMaterial {
	vec3 diffuseColor;
};
void RE_Direct_Toon( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	vec3 irradiance = getGradientIrradiance( geometryNormal, directLight.direction ) * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Toon( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Toon
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Toon`,Jf=`BlinnPhongMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularColor = specular;
material.specularShininess = shininess;
material.specularStrength = specularStrength;`,Kf=`varying vec3 vViewPosition;
struct BlinnPhongMaterial {
	vec3 diffuseColor;
	vec3 specularColor;
	float specularShininess;
	float specularStrength;
};
void RE_Direct_BlinnPhong( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
	reflectedLight.directSpecular += irradiance * BRDF_BlinnPhong( directLight.direction, geometryViewDir, geometryNormal, material.specularColor, material.specularShininess ) * material.specularStrength;
}
void RE_IndirectDiffuse_BlinnPhong( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_BlinnPhong
#define RE_IndirectDiffuse		RE_IndirectDiffuse_BlinnPhong`,jf=`PhysicalMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.diffuseContribution = diffuseColor.rgb * ( 1.0 - metalnessFactor );
material.metalness = metalnessFactor;
vec3 dxy = max( abs( dFdx( nonPerturbedNormal ) ), abs( dFdy( nonPerturbedNormal ) ) );
float geometryRoughness = max( max( dxy.x, dxy.y ), dxy.z );
material.roughness = max( roughnessFactor, 0.0525 );material.roughness += geometryRoughness;
material.roughness = min( material.roughness, 1.0 );
#ifdef IOR
	material.ior = ior;
	#ifdef USE_SPECULAR
		float specularIntensityFactor = specularIntensity;
		vec3 specularColorFactor = specularColor;
		#ifdef USE_SPECULAR_COLORMAP
			specularColorFactor *= texture2D( specularColorMap, vSpecularColorMapUv ).rgb;
		#endif
		#ifdef USE_SPECULAR_INTENSITYMAP
			specularIntensityFactor *= texture2D( specularIntensityMap, vSpecularIntensityMapUv ).a;
		#endif
		material.specularF90 = mix( specularIntensityFactor, 1.0, metalnessFactor );
	#else
		float specularIntensityFactor = 1.0;
		vec3 specularColorFactor = vec3( 1.0 );
		material.specularF90 = 1.0;
	#endif
	material.specularColor = min( pow2( ( material.ior - 1.0 ) / ( material.ior + 1.0 ) ) * specularColorFactor, vec3( 1.0 ) ) * specularIntensityFactor;
	material.specularColorBlended = mix( material.specularColor, diffuseColor.rgb, metalnessFactor );
#else
	material.specularColor = vec3( 0.04 );
	material.specularColorBlended = mix( material.specularColor, diffuseColor.rgb, metalnessFactor );
	material.specularF90 = 1.0;
#endif
#ifdef USE_CLEARCOAT
	material.clearcoat = clearcoat;
	material.clearcoatRoughness = clearcoatRoughness;
	material.clearcoatF0 = vec3( 0.04 );
	material.clearcoatF90 = 1.0;
	#ifdef USE_CLEARCOATMAP
		material.clearcoat *= texture2D( clearcoatMap, vClearcoatMapUv ).x;
	#endif
	#ifdef USE_CLEARCOAT_ROUGHNESSMAP
		material.clearcoatRoughness *= texture2D( clearcoatRoughnessMap, vClearcoatRoughnessMapUv ).y;
	#endif
	material.clearcoat = saturate( material.clearcoat );	material.clearcoatRoughness = max( material.clearcoatRoughness, 0.0525 );
	material.clearcoatRoughness += geometryRoughness;
	material.clearcoatRoughness = min( material.clearcoatRoughness, 1.0 );
#endif
#ifdef USE_DISPERSION
	material.dispersion = dispersion;
#endif
#ifdef USE_IRIDESCENCE
	material.iridescence = iridescence;
	material.iridescenceIOR = iridescenceIOR;
	#ifdef USE_IRIDESCENCEMAP
		material.iridescence *= texture2D( iridescenceMap, vIridescenceMapUv ).r;
	#endif
	#ifdef USE_IRIDESCENCE_THICKNESSMAP
		material.iridescenceThickness = (iridescenceThicknessMaximum - iridescenceThicknessMinimum) * texture2D( iridescenceThicknessMap, vIridescenceThicknessMapUv ).g + iridescenceThicknessMinimum;
	#else
		material.iridescenceThickness = iridescenceThicknessMaximum;
	#endif
#endif
#ifdef USE_SHEEN
	material.sheenColor = sheenColor;
	#ifdef USE_SHEEN_COLORMAP
		material.sheenColor *= texture2D( sheenColorMap, vSheenColorMapUv ).rgb;
	#endif
	material.sheenRoughness = clamp( sheenRoughness, 0.0001, 1.0 );
	#ifdef USE_SHEEN_ROUGHNESSMAP
		material.sheenRoughness *= texture2D( sheenRoughnessMap, vSheenRoughnessMapUv ).a;
	#endif
#endif
#ifdef USE_ANISOTROPY
	#ifdef USE_ANISOTROPYMAP
		mat2 anisotropyMat = mat2( anisotropyVector.x, anisotropyVector.y, - anisotropyVector.y, anisotropyVector.x );
		vec3 anisotropyPolar = texture2D( anisotropyMap, vAnisotropyMapUv ).rgb;
		vec2 anisotropyV = anisotropyMat * normalize( 2.0 * anisotropyPolar.rg - vec2( 1.0 ) ) * anisotropyPolar.b;
	#else
		vec2 anisotropyV = anisotropyVector;
	#endif
	material.anisotropy = length( anisotropyV );
	if( material.anisotropy == 0.0 ) {
		anisotropyV = vec2( 1.0, 0.0 );
	} else {
		anisotropyV /= material.anisotropy;
		material.anisotropy = saturate( material.anisotropy );
	}
	material.alphaT = mix( pow2( material.roughness ), 1.0, pow2( material.anisotropy ) );
	material.anisotropyT = tbn[ 0 ] * anisotropyV.x + tbn[ 1 ] * anisotropyV.y;
	material.anisotropyB = tbn[ 1 ] * anisotropyV.x - tbn[ 0 ] * anisotropyV.y;
#endif`,Qf=`uniform sampler2D dfgLUT;
struct PhysicalMaterial {
	vec3 diffuseColor;
	vec3 diffuseContribution;
	vec3 specularColor;
	vec3 specularColorBlended;
	float roughness;
	float metalness;
	float specularF90;
	float dispersion;
	#ifdef USE_CLEARCOAT
		float clearcoat;
		float clearcoatRoughness;
		vec3 clearcoatF0;
		float clearcoatF90;
	#endif
	#ifdef USE_IRIDESCENCE
		float iridescence;
		float iridescenceIOR;
		float iridescenceThickness;
		vec3 iridescenceFresnel;
		vec3 iridescenceF0;
		vec3 iridescenceFresnelDielectric;
		vec3 iridescenceFresnelMetallic;
	#endif
	#ifdef USE_SHEEN
		vec3 sheenColor;
		float sheenRoughness;
	#endif
	#ifdef IOR
		float ior;
	#endif
	#ifdef USE_TRANSMISSION
		float transmission;
		float transmissionAlpha;
		float thickness;
		float attenuationDistance;
		vec3 attenuationColor;
	#endif
	#ifdef USE_ANISOTROPY
		float anisotropy;
		float alphaT;
		vec3 anisotropyT;
		vec3 anisotropyB;
	#endif
};
vec3 clearcoatSpecularDirect = vec3( 0.0 );
vec3 clearcoatSpecularIndirect = vec3( 0.0 );
vec3 sheenSpecularDirect = vec3( 0.0 );
vec3 sheenSpecularIndirect = vec3(0.0 );
vec3 Schlick_to_F0( const in vec3 f, const in float f90, const in float dotVH ) {
    float x = clamp( 1.0 - dotVH, 0.0, 1.0 );
    float x2 = x * x;
    float x5 = clamp( x * x2 * x2, 0.0, 0.9999 );
    return ( f - vec3( f90 ) * x5 ) / ( 1.0 - x5 );
}
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) {
	float a2 = pow2( alpha );
	float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
	float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
	return 0.5 / max( gv + gl, EPSILON );
}
float D_GGX( const in float alpha, const in float dotNH ) {
	float a2 = pow2( alpha );
	float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
	return RECIPROCAL_PI * a2 / pow2( denom );
}
#ifdef USE_ANISOTROPY
	float V_GGX_SmithCorrelated_Anisotropic( const in float alphaT, const in float alphaB, const in float dotTV, const in float dotBV, const in float dotTL, const in float dotBL, const in float dotNV, const in float dotNL ) {
		float gv = dotNL * length( vec3( alphaT * dotTV, alphaB * dotBV, dotNV ) );
		float gl = dotNV * length( vec3( alphaT * dotTL, alphaB * dotBL, dotNL ) );
		return 0.5 / max( gv + gl, EPSILON );
	}
	float D_GGX_Anisotropic( const in float alphaT, const in float alphaB, const in float dotNH, const in float dotTH, const in float dotBH ) {
		float a2 = alphaT * alphaB;
		highp vec3 v = vec3( alphaB * dotTH, alphaT * dotBH, a2 * dotNH );
		highp float v2 = dot( v, v );
		float w2 = a2 / v2;
		return RECIPROCAL_PI * a2 * pow2 ( w2 );
	}
#endif
#ifdef USE_CLEARCOAT
	vec3 BRDF_GGX_Clearcoat( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material) {
		vec3 f0 = material.clearcoatF0;
		float f90 = material.clearcoatF90;
		float roughness = material.clearcoatRoughness;
		float alpha = pow2( roughness );
		vec3 halfDir = normalize( lightDir + viewDir );
		float dotNL = saturate( dot( normal, lightDir ) );
		float dotNV = saturate( dot( normal, viewDir ) );
		float dotNH = saturate( dot( normal, halfDir ) );
		float dotVH = saturate( dot( viewDir, halfDir ) );
		vec3 F = F_Schlick( f0, f90, dotVH );
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
		return F * ( V * D );
	}
#endif
vec3 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
	vec3 f0 = material.specularColorBlended;
	float f90 = material.specularF90;
	float roughness = material.roughness;
	float alpha = pow2( roughness );
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( f0, f90, dotVH );
	#ifdef USE_IRIDESCENCE
		F = mix( F, material.iridescenceFresnel, material.iridescence );
	#endif
	#ifdef USE_ANISOTROPY
		float dotTL = dot( material.anisotropyT, lightDir );
		float dotTV = dot( material.anisotropyT, viewDir );
		float dotTH = dot( material.anisotropyT, halfDir );
		float dotBL = dot( material.anisotropyB, lightDir );
		float dotBV = dot( material.anisotropyB, viewDir );
		float dotBH = dot( material.anisotropyB, halfDir );
		float V = V_GGX_SmithCorrelated_Anisotropic( material.alphaT, alpha, dotTV, dotBV, dotTL, dotBL, dotNV, dotNL );
		float D = D_GGX_Anisotropic( material.alphaT, alpha, dotNH, dotTH, dotBH );
	#else
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
	#endif
	return F * ( V * D );
}
vec2 LTC_Uv( const in vec3 N, const in vec3 V, const in float roughness ) {
	const float LUT_SIZE = 64.0;
	const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
	const float LUT_BIAS = 0.5 / LUT_SIZE;
	float dotNV = saturate( dot( N, V ) );
	vec2 uv = vec2( roughness, sqrt( 1.0 - dotNV ) );
	uv = uv * LUT_SCALE + LUT_BIAS;
	return uv;
}
float LTC_ClippedSphereFormFactor( const in vec3 f ) {
	float l = length( f );
	return max( ( l * l + f.z ) / ( l + 1.0 ), 0.0 );
}
vec3 LTC_EdgeVectorFormFactor( const in vec3 v1, const in vec3 v2 ) {
	float x = dot( v1, v2 );
	float y = abs( x );
	float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
	float b = 3.4175940 + ( 4.1616724 + y ) * y;
	float v = a / b;
	float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;
	return cross( v1, v2 ) * theta_sintheta;
}
vec3 LTC_Evaluate( const in vec3 N, const in vec3 V, const in vec3 P, const in mat3 mInv, const in vec3 rectCoords[ 4 ] ) {
	vec3 v1 = rectCoords[ 1 ] - rectCoords[ 0 ];
	vec3 v2 = rectCoords[ 3 ] - rectCoords[ 0 ];
	vec3 lightNormal = cross( v1, v2 );
	if( dot( lightNormal, P - rectCoords[ 0 ] ) < 0.0 ) return vec3( 0.0 );
	vec3 T1, T2;
	T1 = normalize( V - N * dot( V, N ) );
	T2 = - cross( N, T1 );
	mat3 mat = mInv * transpose( mat3( T1, T2, N ) );
	vec3 coords[ 4 ];
	coords[ 0 ] = mat * ( rectCoords[ 0 ] - P );
	coords[ 1 ] = mat * ( rectCoords[ 1 ] - P );
	coords[ 2 ] = mat * ( rectCoords[ 2 ] - P );
	coords[ 3 ] = mat * ( rectCoords[ 3 ] - P );
	coords[ 0 ] = normalize( coords[ 0 ] );
	coords[ 1 ] = normalize( coords[ 1 ] );
	coords[ 2 ] = normalize( coords[ 2 ] );
	coords[ 3 ] = normalize( coords[ 3 ] );
	vec3 vectorFormFactor = vec3( 0.0 );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 0 ], coords[ 1 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 1 ], coords[ 2 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 2 ], coords[ 3 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 3 ], coords[ 0 ] );
	float result = LTC_ClippedSphereFormFactor( vectorFormFactor );
	return vec3( result );
}
#if defined( USE_SHEEN )
float D_Charlie( float roughness, float dotNH ) {
	float alpha = pow2( roughness );
	float invAlpha = 1.0 / alpha;
	float cos2h = dotNH * dotNH;
	float sin2h = max( 1.0 - cos2h, 0.0078125 );
	return ( 2.0 + invAlpha ) * pow( sin2h, invAlpha * 0.5 ) / ( 2.0 * PI );
}
float V_Neubelt( float dotNV, float dotNL ) {
	return saturate( 1.0 / ( 4.0 * ( dotNL + dotNV - dotNL * dotNV ) ) );
}
vec3 BRDF_Sheen( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, vec3 sheenColor, const in float sheenRoughness ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float D = D_Charlie( sheenRoughness, dotNH );
	float V = V_Neubelt( dotNV, dotNL );
	return sheenColor * ( D * V );
}
#endif
float IBLSheenBRDF( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	float r2 = roughness * roughness;
	float rInv = 1.0 / ( roughness + 0.1 );
	float a = -1.9362 + 1.0678 * roughness + 0.4573 * r2 - 0.8469 * rInv;
	float b = -0.6014 + 0.5538 * roughness - 0.4670 * r2 - 0.1255 * rInv;
	float DG = exp( a * dotNV + b );
	return saturate( DG );
}
vec3 EnvironmentBRDF( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	vec2 fab = texture2D( dfgLUT, vec2( roughness, dotNV ) ).rg;
	return specularColor * fab.x + specularF90 * fab.y;
}
#ifdef USE_IRIDESCENCE
void computeMultiscatteringIridescence( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float iridescence, const in vec3 iridescenceF0, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#else
void computeMultiscattering( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#endif
	float dotNV = saturate( dot( normal, viewDir ) );
	vec2 fab = texture2D( dfgLUT, vec2( roughness, dotNV ) ).rg;
	#ifdef USE_IRIDESCENCE
		vec3 Fr = mix( specularColor, iridescenceF0, iridescence );
	#else
		vec3 Fr = specularColor;
	#endif
	vec3 FssEss = Fr * fab.x + specularF90 * fab.y;
	float Ess = fab.x + fab.y;
	float Ems = 1.0 - Ess;
	vec3 Favg = Fr + ( 1.0 - Fr ) * 0.047619;	vec3 Fms = FssEss * Favg / ( 1.0 - Ems * Favg );
	singleScatter += FssEss;
	multiScatter += Fms * Ems;
}
vec3 BRDF_GGX_Multiscatter( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
	vec3 singleScatter = BRDF_GGX( lightDir, viewDir, normal, material );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	vec2 dfgV = texture2D( dfgLUT, vec2( material.roughness, dotNV ) ).rg;
	vec2 dfgL = texture2D( dfgLUT, vec2( material.roughness, dotNL ) ).rg;
	vec3 FssEss_V = material.specularColorBlended * dfgV.x + material.specularF90 * dfgV.y;
	vec3 FssEss_L = material.specularColorBlended * dfgL.x + material.specularF90 * dfgL.y;
	float Ess_V = dfgV.x + dfgV.y;
	float Ess_L = dfgL.x + dfgL.y;
	float Ems_V = 1.0 - Ess_V;
	float Ems_L = 1.0 - Ess_L;
	vec3 Favg = material.specularColorBlended + ( 1.0 - material.specularColorBlended ) * 0.047619;
	vec3 Fms = FssEss_V * FssEss_L * Favg / ( 1.0 - Ems_V * Ems_L * Favg + EPSILON );
	float compensationFactor = Ems_V * Ems_L;
	vec3 multiScatter = Fms * compensationFactor;
	return singleScatter + multiScatter;
}
#if NUM_RECT_AREA_LIGHTS > 0
	void RE_Direct_RectArea_Physical( const in RectAreaLight rectAreaLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
		vec3 normal = geometryNormal;
		vec3 viewDir = geometryViewDir;
		vec3 position = geometryPosition;
		vec3 lightPos = rectAreaLight.position;
		vec3 halfWidth = rectAreaLight.halfWidth;
		vec3 halfHeight = rectAreaLight.halfHeight;
		vec3 lightColor = rectAreaLight.color;
		float roughness = material.roughness;
		vec3 rectCoords[ 4 ];
		rectCoords[ 0 ] = lightPos + halfWidth - halfHeight;		rectCoords[ 1 ] = lightPos - halfWidth - halfHeight;
		rectCoords[ 2 ] = lightPos - halfWidth + halfHeight;
		rectCoords[ 3 ] = lightPos + halfWidth + halfHeight;
		vec2 uv = LTC_Uv( normal, viewDir, roughness );
		vec4 t1 = texture2D( ltc_1, uv );
		vec4 t2 = texture2D( ltc_2, uv );
		mat3 mInv = mat3(
			vec3( t1.x, 0, t1.y ),
			vec3(    0, 1,    0 ),
			vec3( t1.z, 0, t1.w )
		);
		vec3 fresnel = ( material.specularColorBlended * t2.x + ( material.specularF90 - material.specularColorBlended ) * t2.y );
		reflectedLight.directSpecular += lightColor * fresnel * LTC_Evaluate( normal, viewDir, position, mInv, rectCoords );
		reflectedLight.directDiffuse += lightColor * material.diffuseContribution * LTC_Evaluate( normal, viewDir, position, mat3( 1.0 ), rectCoords );
		#ifdef USE_CLEARCOAT
			vec3 Ncc = geometryClearcoatNormal;
			vec2 uvClearcoat = LTC_Uv( Ncc, viewDir, material.clearcoatRoughness );
			vec4 t1Clearcoat = texture2D( ltc_1, uvClearcoat );
			vec4 t2Clearcoat = texture2D( ltc_2, uvClearcoat );
			mat3 mInvClearcoat = mat3(
				vec3( t1Clearcoat.x, 0, t1Clearcoat.y ),
				vec3(             0, 1,             0 ),
				vec3( t1Clearcoat.z, 0, t1Clearcoat.w )
			);
			vec3 fresnelClearcoat = material.clearcoatF0 * t2Clearcoat.x + ( material.clearcoatF90 - material.clearcoatF0 ) * t2Clearcoat.y;
			clearcoatSpecularDirect += lightColor * fresnelClearcoat * LTC_Evaluate( Ncc, viewDir, position, mInvClearcoat, rectCoords );
		#endif
	}
#endif
void RE_Direct_Physical( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	#ifdef USE_CLEARCOAT
		float dotNLcc = saturate( dot( geometryClearcoatNormal, directLight.direction ) );
		vec3 ccIrradiance = dotNLcc * directLight.color;
		clearcoatSpecularDirect += ccIrradiance * BRDF_GGX_Clearcoat( directLight.direction, geometryViewDir, geometryClearcoatNormal, material );
	#endif
	#ifdef USE_SHEEN
 
 		sheenSpecularDirect += irradiance * BRDF_Sheen( directLight.direction, geometryViewDir, geometryNormal, material.sheenColor, material.sheenRoughness );
 
 		float sheenAlbedoV = IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
 		float sheenAlbedoL = IBLSheenBRDF( geometryNormal, directLight.direction, material.sheenRoughness );
 
 		float sheenEnergyComp = 1.0 - max3( material.sheenColor ) * max( sheenAlbedoV, sheenAlbedoL );
 
 		irradiance *= sheenEnergyComp;
 
 	#endif
	reflectedLight.directSpecular += irradiance * BRDF_GGX_Multiscatter( directLight.direction, geometryViewDir, geometryNormal, material );
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseContribution );
}
void RE_IndirectDiffuse_Physical( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	vec3 diffuse = irradiance * BRDF_Lambert( material.diffuseContribution );
	#ifdef USE_SHEEN
		float sheenAlbedo = IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
		float sheenEnergyComp = 1.0 - max3( material.sheenColor ) * sheenAlbedo;
		diffuse *= sheenEnergyComp;
	#endif
	reflectedLight.indirectDiffuse += diffuse;
}
void RE_IndirectSpecular_Physical( const in vec3 radiance, const in vec3 irradiance, const in vec3 clearcoatRadiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight) {
	#ifdef USE_CLEARCOAT
		clearcoatSpecularIndirect += clearcoatRadiance * EnvironmentBRDF( geometryClearcoatNormal, geometryViewDir, material.clearcoatF0, material.clearcoatF90, material.clearcoatRoughness );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularIndirect += irradiance * material.sheenColor * IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness ) * RECIPROCAL_PI;
 	#endif
	vec3 singleScatteringDielectric = vec3( 0.0 );
	vec3 multiScatteringDielectric = vec3( 0.0 );
	vec3 singleScatteringMetallic = vec3( 0.0 );
	vec3 multiScatteringMetallic = vec3( 0.0 );
	#ifdef USE_IRIDESCENCE
		computeMultiscatteringIridescence( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.iridescence, material.iridescenceFresnelDielectric, material.roughness, singleScatteringDielectric, multiScatteringDielectric );
		computeMultiscatteringIridescence( geometryNormal, geometryViewDir, material.diffuseColor, material.specularF90, material.iridescence, material.iridescenceFresnelMetallic, material.roughness, singleScatteringMetallic, multiScatteringMetallic );
	#else
		computeMultiscattering( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.roughness, singleScatteringDielectric, multiScatteringDielectric );
		computeMultiscattering( geometryNormal, geometryViewDir, material.diffuseColor, material.specularF90, material.roughness, singleScatteringMetallic, multiScatteringMetallic );
	#endif
	vec3 singleScattering = mix( singleScatteringDielectric, singleScatteringMetallic, material.metalness );
	vec3 multiScattering = mix( multiScatteringDielectric, multiScatteringMetallic, material.metalness );
	vec3 totalScatteringDielectric = singleScatteringDielectric + multiScatteringDielectric;
	vec3 diffuse = material.diffuseContribution * ( 1.0 - totalScatteringDielectric );
	vec3 cosineWeightedIrradiance = irradiance * RECIPROCAL_PI;
	vec3 indirectSpecular = radiance * singleScattering;
	indirectSpecular += multiScattering * cosineWeightedIrradiance;
	vec3 indirectDiffuse = diffuse * cosineWeightedIrradiance;
	#ifdef USE_SHEEN
		float sheenAlbedo = IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
		float sheenEnergyComp = 1.0 - max3( material.sheenColor ) * sheenAlbedo;
		indirectSpecular *= sheenEnergyComp;
		indirectDiffuse *= sheenEnergyComp;
	#endif
	reflectedLight.indirectSpecular += indirectSpecular;
	reflectedLight.indirectDiffuse += indirectDiffuse;
}
#define RE_Direct				RE_Direct_Physical
#define RE_Direct_RectArea		RE_Direct_RectArea_Physical
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Physical
#define RE_IndirectSpecular		RE_IndirectSpecular_Physical
float computeSpecularOcclusion( const in float dotNV, const in float ambientOcclusion, const in float roughness ) {
	return saturate( pow( dotNV + ambientOcclusion, exp2( - 16.0 * roughness - 1.0 ) ) - 1.0 + ambientOcclusion );
}`,tp=`
vec3 geometryPosition = - vViewPosition;
vec3 geometryNormal = normal;
vec3 geometryViewDir = ( isOrthographic ) ? vec3( 0, 0, 1 ) : normalize( vViewPosition );
vec3 geometryClearcoatNormal = vec3( 0.0 );
#ifdef USE_CLEARCOAT
	geometryClearcoatNormal = clearcoatNormal;
#endif
#ifdef USE_IRIDESCENCE
	float dotNVi = saturate( dot( normal, geometryViewDir ) );
	if ( material.iridescenceThickness == 0.0 ) {
		material.iridescence = 0.0;
	} else {
		material.iridescence = saturate( material.iridescence );
	}
	if ( material.iridescence > 0.0 ) {
		material.iridescenceFresnelDielectric = evalIridescence( 1.0, material.iridescenceIOR, dotNVi, material.iridescenceThickness, material.specularColor );
		material.iridescenceFresnelMetallic = evalIridescence( 1.0, material.iridescenceIOR, dotNVi, material.iridescenceThickness, material.diffuseColor );
		material.iridescenceFresnel = mix( material.iridescenceFresnelDielectric, material.iridescenceFresnelMetallic, material.metalness );
		material.iridescenceF0 = Schlick_to_F0( material.iridescenceFresnel, 1.0, dotNVi );
	}
#endif
IncidentLight directLight;
#if ( NUM_POINT_LIGHTS > 0 ) && defined( RE_Direct )
	PointLight pointLight;
	#if defined( USE_SHADOWMAP ) && NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHTS; i ++ ) {
		pointLight = pointLights[ i ];
		getPointLightInfo( pointLight, geometryPosition, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_POINT_LIGHT_SHADOWS ) && ( defined( SHADOWMAP_TYPE_PCF ) || defined( SHADOWMAP_TYPE_BASIC ) )
		pointLightShadow = pointLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getPointShadow( pointShadowMap[ i ], pointLightShadow.shadowMapSize, pointLightShadow.shadowIntensity, pointLightShadow.shadowBias, pointLightShadow.shadowRadius, vPointShadowCoord[ i ], pointLightShadow.shadowCameraNear, pointLightShadow.shadowCameraFar ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_SPOT_LIGHTS > 0 ) && defined( RE_Direct )
	SpotLight spotLight;
	vec4 spotColor;
	vec3 spotLightCoord;
	bool inSpotLightMap;
	#if defined( USE_SHADOWMAP ) && NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHTS; i ++ ) {
		spotLight = spotLights[ i ];
		getSpotLightInfo( spotLight, geometryPosition, directLight );
		#if ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#define SPOT_LIGHT_MAP_INDEX UNROLLED_LOOP_INDEX
		#elif ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		#define SPOT_LIGHT_MAP_INDEX NUM_SPOT_LIGHT_MAPS
		#else
		#define SPOT_LIGHT_MAP_INDEX ( UNROLLED_LOOP_INDEX - NUM_SPOT_LIGHT_SHADOWS + NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#endif
		#if ( SPOT_LIGHT_MAP_INDEX < NUM_SPOT_LIGHT_MAPS )
			spotLightCoord = vSpotLightCoord[ i ].xyz / vSpotLightCoord[ i ].w;
			inSpotLightMap = all( lessThan( abs( spotLightCoord * 2. - 1. ), vec3( 1.0 ) ) );
			spotColor = texture2D( spotLightMap[ SPOT_LIGHT_MAP_INDEX ], spotLightCoord.xy );
			directLight.color = inSpotLightMap ? directLight.color * spotColor.rgb : directLight.color;
		#endif
		#undef SPOT_LIGHT_MAP_INDEX
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		spotLightShadow = spotLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( spotShadowMap[ i ], spotLightShadow.shadowMapSize, spotLightShadow.shadowIntensity, spotLightShadow.shadowBias, spotLightShadow.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_DIR_LIGHTS > 0 ) && defined( RE_Direct )
	DirectionalLight directionalLight;
	#if defined( USE_SHADOWMAP ) && NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHTS; i ++ ) {
		directionalLight = directionalLights[ i ];
		getDirectionalLightInfo( directionalLight, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_DIR_LIGHT_SHADOWS )
		directionalLightShadow = directionalLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( directionalShadowMap[ i ], directionalLightShadow.shadowMapSize, directionalLightShadow.shadowIntensity, directionalLightShadow.shadowBias, directionalLightShadow.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_RECT_AREA_LIGHTS > 0 ) && defined( RE_Direct_RectArea )
	RectAreaLight rectAreaLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_RECT_AREA_LIGHTS; i ++ ) {
		rectAreaLight = rectAreaLights[ i ];
		RE_Direct_RectArea( rectAreaLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if defined( RE_IndirectDiffuse )
	vec3 iblIrradiance = vec3( 0.0 );
	vec3 irradiance = getAmbientLightIrradiance( ambientLightColor );
	#if defined( USE_LIGHT_PROBES )
		irradiance += getLightProbeIrradiance( lightProbe, geometryNormal );
	#endif
	#if ( NUM_HEMI_LIGHTS > 0 )
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_HEMI_LIGHTS; i ++ ) {
			irradiance += getHemisphereLightIrradiance( hemisphereLights[ i ], geometryNormal );
		}
		#pragma unroll_loop_end
	#endif
	#ifdef USE_LIGHT_PROBES_GRID
		vec3 probeWorldPos = ( ( vec4( geometryPosition, 1.0 ) - viewMatrix[ 3 ] ) * viewMatrix ).xyz;
		vec3 probeWorldNormal = inverseTransformDirection( geometryNormal, viewMatrix );
		irradiance += getLightProbeGridIrradiance( probeWorldPos, probeWorldNormal );
	#endif
#endif
#if defined( RE_IndirectSpecular )
	vec3 radiance = vec3( 0.0 );
	vec3 clearcoatRadiance = vec3( 0.0 );
#endif`,ep=`#if defined( RE_IndirectDiffuse )
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		vec3 lightMapIrradiance = lightMapTexel.rgb * lightMapIntensity;
		irradiance += lightMapIrradiance;
	#endif
	#if defined( USE_ENVMAP ) && defined( ENVMAP_TYPE_CUBE_UV )
		#if defined( STANDARD ) || defined( LAMBERT ) || defined( PHONG )
			iblIrradiance += getIBLIrradiance( geometryNormal );
		#endif
	#endif
#endif
#if defined( USE_ENVMAP ) && defined( RE_IndirectSpecular )
	#ifdef USE_ANISOTROPY
		radiance += getIBLAnisotropyRadiance( geometryViewDir, geometryNormal, material.roughness, material.anisotropyB, material.anisotropy );
	#else
		radiance += getIBLRadiance( geometryViewDir, geometryNormal, material.roughness );
	#endif
	#ifdef USE_CLEARCOAT
		clearcoatRadiance += getIBLRadiance( geometryViewDir, geometryClearcoatNormal, material.clearcoatRoughness );
	#endif
#endif`,np=`#if defined( RE_IndirectDiffuse )
	#if defined( LAMBERT ) || defined( PHONG )
		irradiance += iblIrradiance;
	#endif
	RE_IndirectDiffuse( irradiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif
#if defined( RE_IndirectSpecular )
	RE_IndirectSpecular( radiance, iblIrradiance, clearcoatRadiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif`,ip=`#ifdef USE_LIGHT_PROBES_GRID
uniform highp sampler3D probesSH;
uniform vec3 probesMin;
uniform vec3 probesMax;
uniform vec3 probesResolution;
vec3 getLightProbeGridIrradiance( vec3 worldPos, vec3 worldNormal ) {
	vec3 res = probesResolution;
	vec3 gridRange = probesMax - probesMin;
	vec3 resMinusOne = res - 1.0;
	vec3 probeSpacing = gridRange / resMinusOne;
	vec3 samplePos = worldPos + worldNormal * probeSpacing * 0.5;
	vec3 uvw = clamp( ( samplePos - probesMin ) / gridRange, 0.0, 1.0 );
	uvw = uvw * resMinusOne / res + 0.5 / res;
	float nz          = res.z;
	float paddedSlices = nz + 2.0;
	float atlasDepth  = 7.0 * paddedSlices;
	float uvZBase     = uvw.z * nz + 1.0;
	vec4 s0 = texture( probesSH, vec3( uvw.xy, ( uvZBase                       ) / atlasDepth ) );
	vec4 s1 = texture( probesSH, vec3( uvw.xy, ( uvZBase +       paddedSlices   ) / atlasDepth ) );
	vec4 s2 = texture( probesSH, vec3( uvw.xy, ( uvZBase + 2.0 * paddedSlices   ) / atlasDepth ) );
	vec4 s3 = texture( probesSH, vec3( uvw.xy, ( uvZBase + 3.0 * paddedSlices   ) / atlasDepth ) );
	vec4 s4 = texture( probesSH, vec3( uvw.xy, ( uvZBase + 4.0 * paddedSlices   ) / atlasDepth ) );
	vec4 s5 = texture( probesSH, vec3( uvw.xy, ( uvZBase + 5.0 * paddedSlices   ) / atlasDepth ) );
	vec4 s6 = texture( probesSH, vec3( uvw.xy, ( uvZBase + 6.0 * paddedSlices   ) / atlasDepth ) );
	vec3 c0 = s0.xyz;
	vec3 c1 = vec3( s0.w, s1.xy );
	vec3 c2 = vec3( s1.zw, s2.x );
	vec3 c3 = s2.yzw;
	vec3 c4 = s3.xyz;
	vec3 c5 = vec3( s3.w, s4.xy );
	vec3 c6 = vec3( s4.zw, s5.x );
	vec3 c7 = s5.yzw;
	vec3 c8 = s6.xyz;
	float x = worldNormal.x, y = worldNormal.y, z = worldNormal.z;
	vec3 result = c0 * 0.886227;
	result += c1 * 2.0 * 0.511664 * y;
	result += c2 * 2.0 * 0.511664 * z;
	result += c3 * 2.0 * 0.511664 * x;
	result += c4 * 2.0 * 0.429043 * x * y;
	result += c5 * 2.0 * 0.429043 * y * z;
	result += c6 * ( 0.743125 * z * z - 0.247708 );
	result += c7 * 2.0 * 0.429043 * x * z;
	result += c8 * 0.429043 * ( x * x - y * y );
	return max( result, vec3( 0.0 ) );
}
#endif`,sp=`#if defined( USE_LOGARITHMIC_DEPTH_BUFFER )
	gl_FragDepth = vIsPerspective == 0.0 ? gl_FragCoord.z : log2( vFragDepth ) * logDepthBufFC * 0.5;
#endif`,rp=`#if defined( USE_LOGARITHMIC_DEPTH_BUFFER )
	uniform float logDepthBufFC;
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,ap=`#ifdef USE_LOGARITHMIC_DEPTH_BUFFER
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,op=`#ifdef USE_LOGARITHMIC_DEPTH_BUFFER
	vFragDepth = 1.0 + gl_Position.w;
	vIsPerspective = float( isPerspectiveMatrix( projectionMatrix ) );
#endif`,lp=`#ifdef USE_MAP
	vec4 sampledDiffuseColor = texture2D( map, vMapUv );
	#ifdef DECODE_VIDEO_TEXTURE
		sampledDiffuseColor = sRGBTransferEOTF( sampledDiffuseColor );
	#endif
	diffuseColor *= sampledDiffuseColor;
#endif`,cp=`#ifdef USE_MAP
	uniform sampler2D map;
#endif`,hp=`#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
	#if defined( USE_POINTS_UV )
		vec2 uv = vUv;
	#else
		vec2 uv = ( uvTransform * vec3( gl_PointCoord.x, 1.0 - gl_PointCoord.y, 1 ) ).xy;
	#endif
#endif
#ifdef USE_MAP
	diffuseColor *= texture2D( map, uv );
#endif
#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, uv ).g;
#endif`,dp=`#if defined( USE_POINTS_UV )
	varying vec2 vUv;
#else
	#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
		uniform mat3 uvTransform;
	#endif
#endif
#ifdef USE_MAP
	uniform sampler2D map;
#endif
#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,up=`float metalnessFactor = metalness;
#ifdef USE_METALNESSMAP
	vec4 texelMetalness = texture2D( metalnessMap, vMetalnessMapUv );
	metalnessFactor *= texelMetalness.b;
#endif`,fp=`#ifdef USE_METALNESSMAP
	uniform sampler2D metalnessMap;
#endif`,pp=`#ifdef USE_INSTANCING_MORPH
	float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	float morphTargetBaseInfluence = texelFetch( morphTexture, ivec2( 0, gl_InstanceID ), 0 ).r;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		morphTargetInfluences[i] =  texelFetch( morphTexture, ivec2( i + 1, gl_InstanceID ), 0 ).r;
	}
#endif`,mp=`#if defined( USE_MORPHCOLORS )
	vColor *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		#if defined( USE_COLOR_ALPHA )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ) * morphTargetInfluences[ i ];
		#elif defined( USE_COLOR )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ).rgb * morphTargetInfluences[ i ];
		#endif
	}
#endif`,gp=`#ifdef USE_MORPHNORMALS
	objectNormal *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) objectNormal += getMorph( gl_VertexID, i, 1 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,_p=`#ifdef USE_MORPHTARGETS
	#ifndef USE_INSTANCING_MORPH
		uniform float morphTargetBaseInfluence;
		uniform float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	#endif
	uniform sampler2DArray morphTargetsTexture;
	uniform ivec2 morphTargetsTextureSize;
	vec4 getMorph( const in int vertexIndex, const in int morphTargetIndex, const in int offset ) {
		int texelIndex = vertexIndex * MORPHTARGETS_TEXTURE_STRIDE + offset;
		int y = texelIndex / morphTargetsTextureSize.x;
		int x = texelIndex - y * morphTargetsTextureSize.x;
		ivec3 morphUV = ivec3( x, y, morphTargetIndex );
		return texelFetch( morphTargetsTexture, morphUV, 0 );
	}
#endif`,vp=`#ifdef USE_MORPHTARGETS
	transformed *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) transformed += getMorph( gl_VertexID, i, 0 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,xp=`float faceDirection = gl_FrontFacing ? 1.0 : - 1.0;
#ifdef FLAT_SHADED
	vec3 fdx = dFdx( vViewPosition );
	vec3 fdy = dFdy( vViewPosition );
	vec3 normal = normalize( cross( fdx, fdy ) );
#else
	vec3 normal = normalize( vNormal );
	#ifdef DOUBLE_SIDED
		normal *= faceDirection;
	#endif
#endif
#if defined( USE_NORMALMAP_TANGENTSPACE ) || defined( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY )
	#ifdef USE_TANGENT
		mat3 tbn = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn = getTangentFrame( - vViewPosition, normal,
		#if defined( USE_NORMALMAP )
			vNormalMapUv
		#elif defined( USE_CLEARCOAT_NORMALMAP )
			vClearcoatNormalMapUv
		#else
			vUv
		#endif
		);
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn[0] *= faceDirection;
		tbn[1] *= faceDirection;
	#endif
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	#ifdef USE_TANGENT
		mat3 tbn2 = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn2 = getTangentFrame( - vViewPosition, normal, vClearcoatNormalMapUv );
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn2[0] *= faceDirection;
		tbn2[1] *= faceDirection;
	#endif
#endif
vec3 nonPerturbedNormal = normal;`,yp=`#ifdef USE_NORMALMAP_OBJECTSPACE
	normal = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	#ifdef FLIP_SIDED
		normal = - normal;
	#endif
	#ifdef DOUBLE_SIDED
		normal = normal * faceDirection;
	#endif
	normal = normalize( normalMatrix * normal );
#elif defined( USE_NORMALMAP_TANGENTSPACE )
	vec3 mapN = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	#if defined( USE_PACKED_NORMALMAP )
		mapN = vec3( mapN.xy, sqrt( saturate( 1.0 - dot( mapN.xy, mapN.xy ) ) ) );
	#endif
	mapN.xy *= normalScale;
	normal = normalize( tbn * mapN );
#elif defined( USE_BUMPMAP )
	normal = perturbNormalArb( - vViewPosition, normal, dHdxy_fwd(), faceDirection );
#endif`,bp=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,Sp=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,Mp=`#ifndef FLAT_SHADED
	vNormal = normalize( transformedNormal );
	#ifdef USE_TANGENT
		vTangent = normalize( transformedTangent );
		vBitangent = normalize( cross( vNormal, vTangent ) * tangent.w );
	#endif
#endif`,wp=`#ifdef USE_NORMALMAP
	uniform sampler2D normalMap;
	uniform vec2 normalScale;
#endif
#ifdef USE_NORMALMAP_OBJECTSPACE
	uniform mat3 normalMatrix;
#endif
#if ! defined ( USE_TANGENT ) && ( defined ( USE_NORMALMAP_TANGENTSPACE ) || defined ( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY ) )
	mat3 getTangentFrame( vec3 eye_pos, vec3 surf_norm, vec2 uv ) {
		vec3 q0 = dFdx( eye_pos.xyz );
		vec3 q1 = dFdy( eye_pos.xyz );
		vec2 st0 = dFdx( uv.st );
		vec2 st1 = dFdy( uv.st );
		vec3 N = surf_norm;
		vec3 q1perp = cross( q1, N );
		vec3 q0perp = cross( N, q0 );
		vec3 T = q1perp * st0.x + q0perp * st1.x;
		vec3 B = q1perp * st0.y + q0perp * st1.y;
		float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );
		return mat3( T * scale, B * scale, N );
	}
#endif`,Ap=`#ifdef USE_CLEARCOAT
	vec3 clearcoatNormal = nonPerturbedNormal;
#endif`,Ep=`#ifdef USE_CLEARCOAT_NORMALMAP
	vec3 clearcoatMapN = texture2D( clearcoatNormalMap, vClearcoatNormalMapUv ).xyz * 2.0 - 1.0;
	clearcoatMapN.xy *= clearcoatNormalScale;
	clearcoatNormal = normalize( tbn2 * clearcoatMapN );
#endif`,Tp=`#ifdef USE_CLEARCOATMAP
	uniform sampler2D clearcoatMap;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform sampler2D clearcoatNormalMap;
	uniform vec2 clearcoatNormalScale;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform sampler2D clearcoatRoughnessMap;
#endif`,Cp=`#ifdef USE_IRIDESCENCEMAP
	uniform sampler2D iridescenceMap;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform sampler2D iridescenceThicknessMap;
#endif`,Rp=`#ifdef OPAQUE
diffuseColor.a = 1.0;
#endif
#ifdef USE_TRANSMISSION
diffuseColor.a *= material.transmissionAlpha;
#endif
gl_FragColor = vec4( outgoingLight, diffuseColor.a );`,Ip=`vec3 packNormalToRGB( const in vec3 normal ) {
	return normalize( normal ) * 0.5 + 0.5;
}
vec3 unpackRGBToNormal( const in vec3 rgb ) {
	return 2.0 * rgb.xyz - 1.0;
}
const float PackUpscale = 256. / 255.;const float UnpackDownscale = 255. / 256.;const float ShiftRight8 = 1. / 256.;
const float Inv255 = 1. / 255.;
const vec4 PackFactors = vec4( 1.0, 256.0, 256.0 * 256.0, 256.0 * 256.0 * 256.0 );
const vec2 UnpackFactors2 = vec2( UnpackDownscale, 1.0 / PackFactors.g );
const vec3 UnpackFactors3 = vec3( UnpackDownscale / PackFactors.rg, 1.0 / PackFactors.b );
const vec4 UnpackFactors4 = vec4( UnpackDownscale / PackFactors.rgb, 1.0 / PackFactors.a );
vec4 packDepthToRGBA( const in float v ) {
	if( v <= 0.0 )
		return vec4( 0., 0., 0., 0. );
	if( v >= 1.0 )
		return vec4( 1., 1., 1., 1. );
	float vuf;
	float af = modf( v * PackFactors.a, vuf );
	float bf = modf( vuf * ShiftRight8, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec4( vuf * Inv255, gf * PackUpscale, bf * PackUpscale, af );
}
vec3 packDepthToRGB( const in float v ) {
	if( v <= 0.0 )
		return vec3( 0., 0., 0. );
	if( v >= 1.0 )
		return vec3( 1., 1., 1. );
	float vuf;
	float bf = modf( v * PackFactors.b, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec3( vuf * Inv255, gf * PackUpscale, bf );
}
vec2 packDepthToRG( const in float v ) {
	if( v <= 0.0 )
		return vec2( 0., 0. );
	if( v >= 1.0 )
		return vec2( 1., 1. );
	float vuf;
	float gf = modf( v * 256., vuf );
	return vec2( vuf * Inv255, gf );
}
float unpackRGBAToDepth( const in vec4 v ) {
	return dot( v, UnpackFactors4 );
}
float unpackRGBToDepth( const in vec3 v ) {
	return dot( v, UnpackFactors3 );
}
float unpackRGToDepth( const in vec2 v ) {
	return v.r * UnpackFactors2.r + v.g * UnpackFactors2.g;
}
vec4 pack2HalfToRGBA( const in vec2 v ) {
	vec4 r = vec4( v.x, fract( v.x * 255.0 ), v.y, fract( v.y * 255.0 ) );
	return vec4( r.x - r.y / 255.0, r.y, r.z - r.w / 255.0, r.w );
}
vec2 unpackRGBATo2Half( const in vec4 v ) {
	return vec2( v.x + ( v.y / 255.0 ), v.z + ( v.w / 255.0 ) );
}
float viewZToOrthographicDepth( const in float viewZ, const in float near, const in float far ) {
	return ( viewZ + near ) / ( near - far );
}
float orthographicDepthToViewZ( const in float depth, const in float near, const in float far ) {
	#ifdef USE_REVERSED_DEPTH_BUFFER
	
		return depth * ( far - near ) - far;
	#else
		return depth * ( near - far ) - near;
	#endif
}
float viewZToPerspectiveDepth( const in float viewZ, const in float near, const in float far ) {
	return ( ( near + viewZ ) * far ) / ( ( far - near ) * viewZ );
}
float perspectiveDepthToViewZ( const in float depth, const in float near, const in float far ) {
	
	#ifdef USE_REVERSED_DEPTH_BUFFER
		return ( near * far ) / ( ( near - far ) * depth - near );
	#else
		return ( near * far ) / ( ( far - near ) * depth - far );
	#endif
}`,Pp=`#ifdef PREMULTIPLIED_ALPHA
	gl_FragColor.rgb *= gl_FragColor.a;
#endif`,Lp=`vec4 mvPosition = vec4( transformed, 1.0 );
#ifdef USE_BATCHING
	mvPosition = batchingMatrix * mvPosition;
#endif
#ifdef USE_INSTANCING
	mvPosition = instanceMatrix * mvPosition;
#endif
mvPosition = modelViewMatrix * mvPosition;
gl_Position = projectionMatrix * mvPosition;`,Dp=`#ifdef DITHERING
	gl_FragColor.rgb = dithering( gl_FragColor.rgb );
#endif`,Np=`#ifdef DITHERING
	vec3 dithering( vec3 color ) {
		float grid_position = rand( gl_FragCoord.xy );
		vec3 dither_shift_RGB = vec3( 0.25 / 255.0, -0.25 / 255.0, 0.25 / 255.0 );
		dither_shift_RGB = mix( 2.0 * dither_shift_RGB, -2.0 * dither_shift_RGB, grid_position );
		return color + dither_shift_RGB;
	}
#endif`,Up=`float roughnessFactor = roughness;
#ifdef USE_ROUGHNESSMAP
	vec4 texelRoughness = texture2D( roughnessMap, vRoughnessMapUv );
	roughnessFactor *= texelRoughness.g;
#endif`,Fp=`#ifdef USE_ROUGHNESSMAP
	uniform sampler2D roughnessMap;
#endif`,Op=`#if NUM_SPOT_LIGHT_COORDS > 0
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#if NUM_SPOT_LIGHT_MAPS > 0
	uniform sampler2D spotLightMap[ NUM_SPOT_LIGHT_MAPS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		#if defined( SHADOWMAP_TYPE_PCF )
			uniform sampler2DShadow directionalShadowMap[ NUM_DIR_LIGHT_SHADOWS ];
		#else
			uniform sampler2D directionalShadowMap[ NUM_DIR_LIGHT_SHADOWS ];
		#endif
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		#if defined( SHADOWMAP_TYPE_PCF )
			uniform sampler2DShadow spotShadowMap[ NUM_SPOT_LIGHT_SHADOWS ];
		#else
			uniform sampler2D spotShadowMap[ NUM_SPOT_LIGHT_SHADOWS ];
		#endif
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		#if defined( SHADOWMAP_TYPE_PCF )
			uniform samplerCubeShadow pointShadowMap[ NUM_POINT_LIGHT_SHADOWS ];
		#elif defined( SHADOWMAP_TYPE_BASIC )
			uniform samplerCube pointShadowMap[ NUM_POINT_LIGHT_SHADOWS ];
		#endif
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
	#if defined( SHADOWMAP_TYPE_PCF )
		float interleavedGradientNoise( vec2 position ) {
			return fract( 52.9829189 * fract( dot( position, vec2( 0.06711056, 0.00583715 ) ) ) );
		}
		vec2 vogelDiskSample( int sampleIndex, int samplesCount, float phi ) {
			const float goldenAngle = 2.399963229728653;
			float r = sqrt( ( float( sampleIndex ) + 0.5 ) / float( samplesCount ) );
			float theta = float( sampleIndex ) * goldenAngle + phi;
			return vec2( cos( theta ), sin( theta ) ) * r;
		}
	#endif
	#if defined( SHADOWMAP_TYPE_PCF )
		float getShadow( sampler2DShadow shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
			float shadow = 1.0;
			shadowCoord.xyz /= shadowCoord.w;
			shadowCoord.z += shadowBias;
			bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
			bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
			if ( frustumTest ) {
				vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
				float radius = shadowRadius * texelSize.x;
				float phi = interleavedGradientNoise( gl_FragCoord.xy ) * PI2;
				shadow = (
					texture( shadowMap, vec3( shadowCoord.xy + vogelDiskSample( 0, 5, phi ) * radius, shadowCoord.z ) ) +
					texture( shadowMap, vec3( shadowCoord.xy + vogelDiskSample( 1, 5, phi ) * radius, shadowCoord.z ) ) +
					texture( shadowMap, vec3( shadowCoord.xy + vogelDiskSample( 2, 5, phi ) * radius, shadowCoord.z ) ) +
					texture( shadowMap, vec3( shadowCoord.xy + vogelDiskSample( 3, 5, phi ) * radius, shadowCoord.z ) ) +
					texture( shadowMap, vec3( shadowCoord.xy + vogelDiskSample( 4, 5, phi ) * radius, shadowCoord.z ) )
				) * 0.2;
			}
			return mix( 1.0, shadow, shadowIntensity );
		}
	#elif defined( SHADOWMAP_TYPE_VSM )
		float getShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
			float shadow = 1.0;
			shadowCoord.xyz /= shadowCoord.w;
			#ifdef USE_REVERSED_DEPTH_BUFFER
				shadowCoord.z -= shadowBias;
			#else
				shadowCoord.z += shadowBias;
			#endif
			bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
			bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
			if ( frustumTest ) {
				vec2 distribution = texture2D( shadowMap, shadowCoord.xy ).rg;
				float mean = distribution.x;
				float variance = distribution.y * distribution.y;
				#ifdef USE_REVERSED_DEPTH_BUFFER
					float hard_shadow = step( mean, shadowCoord.z );
				#else
					float hard_shadow = step( shadowCoord.z, mean );
				#endif
				
				if ( hard_shadow == 1.0 ) {
					shadow = 1.0;
				} else {
					variance = max( variance, 0.0000001 );
					float d = shadowCoord.z - mean;
					float p_max = variance / ( variance + d * d );
					p_max = clamp( ( p_max - 0.3 ) / 0.65, 0.0, 1.0 );
					shadow = max( hard_shadow, p_max );
				}
			}
			return mix( 1.0, shadow, shadowIntensity );
		}
	#else
		float getShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
			float shadow = 1.0;
			shadowCoord.xyz /= shadowCoord.w;
			#ifdef USE_REVERSED_DEPTH_BUFFER
				shadowCoord.z -= shadowBias;
			#else
				shadowCoord.z += shadowBias;
			#endif
			bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
			bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
			if ( frustumTest ) {
				float depth = texture2D( shadowMap, shadowCoord.xy ).r;
				#ifdef USE_REVERSED_DEPTH_BUFFER
					shadow = step( depth, shadowCoord.z );
				#else
					shadow = step( shadowCoord.z, depth );
				#endif
			}
			return mix( 1.0, shadow, shadowIntensity );
		}
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
	#if defined( SHADOWMAP_TYPE_PCF )
	float getPointShadow( samplerCubeShadow shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord, float shadowCameraNear, float shadowCameraFar ) {
		float shadow = 1.0;
		vec3 lightToPosition = shadowCoord.xyz;
		vec3 bd3D = normalize( lightToPosition );
		vec3 absVec = abs( lightToPosition );
		float viewSpaceZ = max( max( absVec.x, absVec.y ), absVec.z );
		if ( viewSpaceZ - shadowCameraFar <= 0.0 && viewSpaceZ - shadowCameraNear >= 0.0 ) {
			#ifdef USE_REVERSED_DEPTH_BUFFER
				float dp = ( shadowCameraNear * ( shadowCameraFar - viewSpaceZ ) ) / ( viewSpaceZ * ( shadowCameraFar - shadowCameraNear ) );
				dp -= shadowBias;
			#else
				float dp = ( shadowCameraFar * ( viewSpaceZ - shadowCameraNear ) ) / ( viewSpaceZ * ( shadowCameraFar - shadowCameraNear ) );
				dp += shadowBias;
			#endif
			float texelSize = shadowRadius / shadowMapSize.x;
			vec3 absDir = abs( bd3D );
			vec3 tangent = absDir.x > absDir.z ? vec3( 0.0, 1.0, 0.0 ) : vec3( 1.0, 0.0, 0.0 );
			tangent = normalize( cross( bd3D, tangent ) );
			vec3 bitangent = cross( bd3D, tangent );
			float phi = interleavedGradientNoise( gl_FragCoord.xy ) * PI2;
			vec2 sample0 = vogelDiskSample( 0, 5, phi );
			vec2 sample1 = vogelDiskSample( 1, 5, phi );
			vec2 sample2 = vogelDiskSample( 2, 5, phi );
			vec2 sample3 = vogelDiskSample( 3, 5, phi );
			vec2 sample4 = vogelDiskSample( 4, 5, phi );
			shadow = (
				texture( shadowMap, vec4( bd3D + ( tangent * sample0.x + bitangent * sample0.y ) * texelSize, dp ) ) +
				texture( shadowMap, vec4( bd3D + ( tangent * sample1.x + bitangent * sample1.y ) * texelSize, dp ) ) +
				texture( shadowMap, vec4( bd3D + ( tangent * sample2.x + bitangent * sample2.y ) * texelSize, dp ) ) +
				texture( shadowMap, vec4( bd3D + ( tangent * sample3.x + bitangent * sample3.y ) * texelSize, dp ) ) +
				texture( shadowMap, vec4( bd3D + ( tangent * sample4.x + bitangent * sample4.y ) * texelSize, dp ) )
			) * 0.2;
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
	#elif defined( SHADOWMAP_TYPE_BASIC )
	float getPointShadow( samplerCube shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord, float shadowCameraNear, float shadowCameraFar ) {
		float shadow = 1.0;
		vec3 lightToPosition = shadowCoord.xyz;
		vec3 absVec = abs( lightToPosition );
		float viewSpaceZ = max( max( absVec.x, absVec.y ), absVec.z );
		if ( viewSpaceZ - shadowCameraFar <= 0.0 && viewSpaceZ - shadowCameraNear >= 0.0 ) {
			float dp = ( shadowCameraFar * ( viewSpaceZ - shadowCameraNear ) ) / ( viewSpaceZ * ( shadowCameraFar - shadowCameraNear ) );
			dp += shadowBias;
			vec3 bd3D = normalize( lightToPosition );
			float depth = textureCube( shadowMap, bd3D ).r;
			#ifdef USE_REVERSED_DEPTH_BUFFER
				depth = 1.0 - depth;
			#endif
			shadow = step( dp, depth );
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
	#endif
	#endif
#endif`,Bp=`#if NUM_SPOT_LIGHT_COORDS > 0
	uniform mat4 spotLightMatrix[ NUM_SPOT_LIGHT_COORDS ];
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform mat4 directionalShadowMatrix[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform mat4 pointShadowMatrix[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
#endif`,kp=`#if ( defined( USE_SHADOWMAP ) && ( NUM_DIR_LIGHT_SHADOWS > 0 || NUM_POINT_LIGHT_SHADOWS > 0 ) ) || ( NUM_SPOT_LIGHT_COORDS > 0 )
	#ifdef HAS_NORMAL
		vec3 shadowWorldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
	#else
		vec3 shadowWorldNormal = vec3( 0.0 );
	#endif
	vec4 shadowWorldPosition;
#endif
#if defined( USE_SHADOWMAP )
	#if NUM_DIR_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * directionalLightShadows[ i ].shadowNormalBias, 0 );
			vDirectionalShadowCoord[ i ] = directionalShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * pointLightShadows[ i ].shadowNormalBias, 0 );
			vPointShadowCoord[ i ] = pointShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
#endif
#if NUM_SPOT_LIGHT_COORDS > 0
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_COORDS; i ++ ) {
		shadowWorldPosition = worldPosition;
		#if ( defined( USE_SHADOWMAP ) && UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
			shadowWorldPosition.xyz += shadowWorldNormal * spotLightShadows[ i ].shadowNormalBias;
		#endif
		vSpotLightCoord[ i ] = spotLightMatrix[ i ] * shadowWorldPosition;
	}
	#pragma unroll_loop_end
#endif`,zp=`float getShadowMask() {
	float shadow = 1.0;
	#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
		directionalLight = directionalLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( directionalShadowMap[ i ], directionalLight.shadowMapSize, directionalLight.shadowIntensity, directionalLight.shadowBias, directionalLight.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_SHADOWS; i ++ ) {
		spotLight = spotLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( spotShadowMap[ i ], spotLight.shadowMapSize, spotLight.shadowIntensity, spotLight.shadowBias, spotLight.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0 && ( defined( SHADOWMAP_TYPE_PCF ) || defined( SHADOWMAP_TYPE_BASIC ) )
	PointLightShadow pointLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
		pointLight = pointLightShadows[ i ];
		shadow *= receiveShadow ? getPointShadow( pointShadowMap[ i ], pointLight.shadowMapSize, pointLight.shadowIntensity, pointLight.shadowBias, pointLight.shadowRadius, vPointShadowCoord[ i ], pointLight.shadowCameraNear, pointLight.shadowCameraFar ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#endif
	return shadow;
}`,Vp=`#ifdef USE_SKINNING
	mat4 boneMatX = getBoneMatrix( skinIndex.x );
	mat4 boneMatY = getBoneMatrix( skinIndex.y );
	mat4 boneMatZ = getBoneMatrix( skinIndex.z );
	mat4 boneMatW = getBoneMatrix( skinIndex.w );
#endif`,Gp=`#ifdef USE_SKINNING
	uniform mat4 bindMatrix;
	uniform mat4 bindMatrixInverse;
	uniform highp sampler2D boneTexture;
	mat4 getBoneMatrix( const in float i ) {
		int size = textureSize( boneTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( boneTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( boneTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( boneTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( boneTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
#endif`,Hp=`#ifdef USE_SKINNING
	vec4 skinVertex = bindMatrix * vec4( transformed, 1.0 );
	vec4 skinned = vec4( 0.0 );
	skinned += boneMatX * skinVertex * skinWeight.x;
	skinned += boneMatY * skinVertex * skinWeight.y;
	skinned += boneMatZ * skinVertex * skinWeight.z;
	skinned += boneMatW * skinVertex * skinWeight.w;
	transformed = ( bindMatrixInverse * skinned ).xyz;
#endif`,Wp=`#ifdef USE_SKINNING
	mat4 skinMatrix = mat4( 0.0 );
	skinMatrix += skinWeight.x * boneMatX;
	skinMatrix += skinWeight.y * boneMatY;
	skinMatrix += skinWeight.z * boneMatZ;
	skinMatrix += skinWeight.w * boneMatW;
	skinMatrix = bindMatrixInverse * skinMatrix * bindMatrix;
	objectNormal = vec4( skinMatrix * vec4( objectNormal, 0.0 ) ).xyz;
	#ifdef USE_TANGENT
		objectTangent = vec4( skinMatrix * vec4( objectTangent, 0.0 ) ).xyz;
	#endif
#endif`,$p=`float specularStrength;
#ifdef USE_SPECULARMAP
	vec4 texelSpecular = texture2D( specularMap, vSpecularMapUv );
	specularStrength = texelSpecular.r;
#else
	specularStrength = 1.0;
#endif`,Xp=`#ifdef USE_SPECULARMAP
	uniform sampler2D specularMap;
#endif`,qp=`#if defined( TONE_MAPPING )
	gl_FragColor.rgb = toneMapping( gl_FragColor.rgb );
#endif`,Yp=`#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
uniform float toneMappingExposure;
vec3 LinearToneMapping( vec3 color ) {
	return saturate( toneMappingExposure * color );
}
vec3 ReinhardToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	return saturate( color / ( vec3( 1.0 ) + color ) );
}
vec3 CineonToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	color = max( vec3( 0.0 ), color - 0.004 );
	return pow( ( color * ( 6.2 * color + 0.5 ) ) / ( color * ( 6.2 * color + 1.7 ) + 0.06 ), vec3( 2.2 ) );
}
vec3 RRTAndODTFit( vec3 v ) {
	vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
	vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
	return a / b;
}
vec3 ACESFilmicToneMapping( vec3 color ) {
	const mat3 ACESInputMat = mat3(
		vec3( 0.59719, 0.07600, 0.02840 ),		vec3( 0.35458, 0.90834, 0.13383 ),
		vec3( 0.04823, 0.01566, 0.83777 )
	);
	const mat3 ACESOutputMat = mat3(
		vec3(  1.60475, -0.10208, -0.00327 ),		vec3( -0.53108,  1.10813, -0.07276 ),
		vec3( -0.07367, -0.00605,  1.07602 )
	);
	color *= toneMappingExposure / 0.6;
	color = ACESInputMat * color;
	color = RRTAndODTFit( color );
	color = ACESOutputMat * color;
	return saturate( color );
}
const mat3 LINEAR_REC2020_TO_LINEAR_SRGB = mat3(
	vec3( 1.6605, - 0.1246, - 0.0182 ),
	vec3( - 0.5876, 1.1329, - 0.1006 ),
	vec3( - 0.0728, - 0.0083, 1.1187 )
);
const mat3 LINEAR_SRGB_TO_LINEAR_REC2020 = mat3(
	vec3( 0.6274, 0.0691, 0.0164 ),
	vec3( 0.3293, 0.9195, 0.0880 ),
	vec3( 0.0433, 0.0113, 0.8956 )
);
vec3 agxDefaultContrastApprox( vec3 x ) {
	vec3 x2 = x * x;
	vec3 x4 = x2 * x2;
	return + 15.5 * x4 * x2
		- 40.14 * x4 * x
		+ 31.96 * x4
		- 6.868 * x2 * x
		+ 0.4298 * x2
		+ 0.1191 * x
		- 0.00232;
}
vec3 AgXToneMapping( vec3 color ) {
	const mat3 AgXInsetMatrix = mat3(
		vec3( 0.856627153315983, 0.137318972929847, 0.11189821299995 ),
		vec3( 0.0951212405381588, 0.761241990602591, 0.0767994186031903 ),
		vec3( 0.0482516061458583, 0.101439036467562, 0.811302368396859 )
	);
	const mat3 AgXOutsetMatrix = mat3(
		vec3( 1.1271005818144368, - 0.1413297634984383, - 0.14132976349843826 ),
		vec3( - 0.11060664309660323, 1.157823702216272, - 0.11060664309660294 ),
		vec3( - 0.016493938717834573, - 0.016493938717834257, 1.2519364065950405 )
	);
	const float AgxMinEv = - 12.47393;	const float AgxMaxEv = 4.026069;
	color *= toneMappingExposure;
	color = LINEAR_SRGB_TO_LINEAR_REC2020 * color;
	color = AgXInsetMatrix * color;
	color = max( color, 1e-10 );	color = log2( color );
	color = ( color - AgxMinEv ) / ( AgxMaxEv - AgxMinEv );
	color = clamp( color, 0.0, 1.0 );
	color = agxDefaultContrastApprox( color );
	color = AgXOutsetMatrix * color;
	color = pow( max( vec3( 0.0 ), color ), vec3( 2.2 ) );
	color = LINEAR_REC2020_TO_LINEAR_SRGB * color;
	color = clamp( color, 0.0, 1.0 );
	return color;
}
vec3 NeutralToneMapping( vec3 color ) {
	const float StartCompression = 0.8 - 0.04;
	const float Desaturation = 0.15;
	color *= toneMappingExposure;
	float x = min( color.r, min( color.g, color.b ) );
	float offset = x < 0.08 ? x - 6.25 * x * x : 0.04;
	color -= offset;
	float peak = max( color.r, max( color.g, color.b ) );
	if ( peak < StartCompression ) return color;
	float d = 1. - StartCompression;
	float newPeak = 1. - d * d / ( peak + d - StartCompression );
	color *= newPeak / peak;
	float g = 1. - 1. / ( Desaturation * ( peak - newPeak ) + 1. );
	return mix( color, vec3( newPeak ), g );
}
vec3 CustomToneMapping( vec3 color ) { return color; }`,Zp=`#ifdef USE_TRANSMISSION
	material.transmission = transmission;
	material.transmissionAlpha = 1.0;
	material.thickness = thickness;
	material.attenuationDistance = attenuationDistance;
	material.attenuationColor = attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		material.transmission *= texture2D( transmissionMap, vTransmissionMapUv ).r;
	#endif
	#ifdef USE_THICKNESSMAP
		material.thickness *= texture2D( thicknessMap, vThicknessMapUv ).g;
	#endif
	vec3 pos = vWorldPosition;
	vec3 v = normalize( cameraPosition - pos );
	vec3 n = inverseTransformDirection( normal, viewMatrix );
	vec4 transmitted = getIBLVolumeRefraction(
		n, v, material.roughness, material.diffuseContribution, material.specularColorBlended, material.specularF90,
		pos, modelMatrix, viewMatrix, projectionMatrix, material.dispersion, material.ior, material.thickness,
		material.attenuationColor, material.attenuationDistance );
	material.transmissionAlpha = mix( material.transmissionAlpha, transmitted.a, material.transmission );
	totalDiffuse = mix( totalDiffuse, transmitted.rgb, material.transmission );
#endif`,Jp=`#ifdef USE_TRANSMISSION
	uniform float transmission;
	uniform float thickness;
	uniform float attenuationDistance;
	uniform vec3 attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		uniform sampler2D transmissionMap;
	#endif
	#ifdef USE_THICKNESSMAP
		uniform sampler2D thicknessMap;
	#endif
	uniform vec2 transmissionSamplerSize;
	uniform sampler2D transmissionSamplerMap;
	uniform mat4 modelMatrix;
	uniform mat4 projectionMatrix;
	varying vec3 vWorldPosition;
	float w0( float a ) {
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - a + 3.0 ) - 3.0 ) + 1.0 );
	}
	float w1( float a ) {
		return ( 1.0 / 6.0 ) * ( a *  a * ( 3.0 * a - 6.0 ) + 4.0 );
	}
	float w2( float a ){
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - 3.0 * a + 3.0 ) + 3.0 ) + 1.0 );
	}
	float w3( float a ) {
		return ( 1.0 / 6.0 ) * ( a * a * a );
	}
	float g0( float a ) {
		return w0( a ) + w1( a );
	}
	float g1( float a ) {
		return w2( a ) + w3( a );
	}
	float h0( float a ) {
		return - 1.0 + w1( a ) / ( w0( a ) + w1( a ) );
	}
	float h1( float a ) {
		return 1.0 + w3( a ) / ( w2( a ) + w3( a ) );
	}
	vec4 bicubic( sampler2D tex, vec2 uv, vec4 texelSize, float lod ) {
		uv = uv * texelSize.zw + 0.5;
		vec2 iuv = floor( uv );
		vec2 fuv = fract( uv );
		float g0x = g0( fuv.x );
		float g1x = g1( fuv.x );
		float h0x = h0( fuv.x );
		float h1x = h1( fuv.x );
		float h0y = h0( fuv.y );
		float h1y = h1( fuv.y );
		vec2 p0 = ( vec2( iuv.x + h0x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p1 = ( vec2( iuv.x + h1x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p2 = ( vec2( iuv.x + h0x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		vec2 p3 = ( vec2( iuv.x + h1x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		return g0( fuv.y ) * ( g0x * textureLod( tex, p0, lod ) + g1x * textureLod( tex, p1, lod ) ) +
			g1( fuv.y ) * ( g0x * textureLod( tex, p2, lod ) + g1x * textureLod( tex, p3, lod ) );
	}
	vec4 textureBicubic( sampler2D sampler, vec2 uv, float lod ) {
		vec2 fLodSize = vec2( textureSize( sampler, int( lod ) ) );
		vec2 cLodSize = vec2( textureSize( sampler, int( lod + 1.0 ) ) );
		vec2 fLodSizeInv = 1.0 / fLodSize;
		vec2 cLodSizeInv = 1.0 / cLodSize;
		vec4 fSample = bicubic( sampler, uv, vec4( fLodSizeInv, fLodSize ), floor( lod ) );
		vec4 cSample = bicubic( sampler, uv, vec4( cLodSizeInv, cLodSize ), ceil( lod ) );
		return mix( fSample, cSample, fract( lod ) );
	}
	vec3 getVolumeTransmissionRay( const in vec3 n, const in vec3 v, const in float thickness, const in float ior, const in mat4 modelMatrix ) {
		vec3 refractionVector = refract( - v, normalize( n ), 1.0 / ior );
		vec3 modelScale;
		modelScale.x = length( vec3( modelMatrix[ 0 ].xyz ) );
		modelScale.y = length( vec3( modelMatrix[ 1 ].xyz ) );
		modelScale.z = length( vec3( modelMatrix[ 2 ].xyz ) );
		return normalize( refractionVector ) * thickness * modelScale;
	}
	float applyIorToRoughness( const in float roughness, const in float ior ) {
		return roughness * clamp( ior * 2.0 - 2.0, 0.0, 1.0 );
	}
	vec4 getTransmissionSample( const in vec2 fragCoord, const in float roughness, const in float ior ) {
		float lod = log2( transmissionSamplerSize.x ) * applyIorToRoughness( roughness, ior );
		return textureBicubic( transmissionSamplerMap, fragCoord.xy, lod );
	}
	vec3 volumeAttenuation( const in float transmissionDistance, const in vec3 attenuationColor, const in float attenuationDistance ) {
		if ( isinf( attenuationDistance ) ) {
			return vec3( 1.0 );
		} else {
			vec3 attenuationCoefficient = -log( attenuationColor ) / attenuationDistance;
			vec3 transmittance = exp( - attenuationCoefficient * transmissionDistance );			return transmittance;
		}
	}
	vec4 getIBLVolumeRefraction( const in vec3 n, const in vec3 v, const in float roughness, const in vec3 diffuseColor,
		const in vec3 specularColor, const in float specularF90, const in vec3 position, const in mat4 modelMatrix,
		const in mat4 viewMatrix, const in mat4 projMatrix, const in float dispersion, const in float ior, const in float thickness,
		const in vec3 attenuationColor, const in float attenuationDistance ) {
		vec4 transmittedLight;
		vec3 transmittance;
		#ifdef USE_DISPERSION
			float halfSpread = ( ior - 1.0 ) * 0.025 * dispersion;
			vec3 iors = vec3( ior - halfSpread, ior, ior + halfSpread );
			for ( int i = 0; i < 3; i ++ ) {
				vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, iors[ i ], modelMatrix );
				vec3 refractedRayExit = position + transmissionRay;
				vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
				vec2 refractionCoords = ndcPos.xy / ndcPos.w;
				refractionCoords += 1.0;
				refractionCoords /= 2.0;
				vec4 transmissionSample = getTransmissionSample( refractionCoords, roughness, iors[ i ] );
				transmittedLight[ i ] = transmissionSample[ i ];
				transmittedLight.a += transmissionSample.a;
				transmittance[ i ] = diffuseColor[ i ] * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance )[ i ];
			}
			transmittedLight.a /= 3.0;
		#else
			vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, ior, modelMatrix );
			vec3 refractedRayExit = position + transmissionRay;
			vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
			vec2 refractionCoords = ndcPos.xy / ndcPos.w;
			refractionCoords += 1.0;
			refractionCoords /= 2.0;
			transmittedLight = getTransmissionSample( refractionCoords, roughness, ior );
			transmittance = diffuseColor * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance );
		#endif
		vec3 attenuatedColor = transmittance * transmittedLight.rgb;
		vec3 F = EnvironmentBRDF( n, v, specularColor, specularF90, roughness );
		float transmittanceFactor = ( transmittance.r + transmittance.g + transmittance.b ) / 3.0;
		return vec4( ( 1.0 - F ) * attenuatedColor, 1.0 - ( 1.0 - transmittedLight.a ) * transmittanceFactor );
	}
#endif`,Kp=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_SPECULARMAP
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,jp=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	uniform mat3 mapTransform;
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	uniform mat3 alphaMapTransform;
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	uniform mat3 lightMapTransform;
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	uniform mat3 aoMapTransform;
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	uniform mat3 bumpMapTransform;
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	uniform mat3 normalMapTransform;
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_DISPLACEMENTMAP
	uniform mat3 displacementMapTransform;
	varying vec2 vDisplacementMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	uniform mat3 emissiveMapTransform;
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	uniform mat3 metalnessMapTransform;
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	uniform mat3 roughnessMapTransform;
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	uniform mat3 anisotropyMapTransform;
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	uniform mat3 clearcoatMapTransform;
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform mat3 clearcoatNormalMapTransform;
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform mat3 clearcoatRoughnessMapTransform;
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	uniform mat3 sheenColorMapTransform;
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	uniform mat3 sheenRoughnessMapTransform;
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	uniform mat3 iridescenceMapTransform;
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform mat3 iridescenceThicknessMapTransform;
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SPECULARMAP
	uniform mat3 specularMapTransform;
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	uniform mat3 specularColorMapTransform;
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	uniform mat3 specularIntensityMapTransform;
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Qp=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	vUv = vec3( uv, 1 ).xy;
#endif
#ifdef USE_MAP
	vMapUv = ( mapTransform * vec3( MAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ALPHAMAP
	vAlphaMapUv = ( alphaMapTransform * vec3( ALPHAMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_LIGHTMAP
	vLightMapUv = ( lightMapTransform * vec3( LIGHTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_AOMAP
	vAoMapUv = ( aoMapTransform * vec3( AOMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_BUMPMAP
	vBumpMapUv = ( bumpMapTransform * vec3( BUMPMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_NORMALMAP
	vNormalMapUv = ( normalMapTransform * vec3( NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_DISPLACEMENTMAP
	vDisplacementMapUv = ( displacementMapTransform * vec3( DISPLACEMENTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_EMISSIVEMAP
	vEmissiveMapUv = ( emissiveMapTransform * vec3( EMISSIVEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_METALNESSMAP
	vMetalnessMapUv = ( metalnessMapTransform * vec3( METALNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ROUGHNESSMAP
	vRoughnessMapUv = ( roughnessMapTransform * vec3( ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ANISOTROPYMAP
	vAnisotropyMapUv = ( anisotropyMapTransform * vec3( ANISOTROPYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOATMAP
	vClearcoatMapUv = ( clearcoatMapTransform * vec3( CLEARCOATMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	vClearcoatNormalMapUv = ( clearcoatNormalMapTransform * vec3( CLEARCOAT_NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	vClearcoatRoughnessMapUv = ( clearcoatRoughnessMapTransform * vec3( CLEARCOAT_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCEMAP
	vIridescenceMapUv = ( iridescenceMapTransform * vec3( IRIDESCENCEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	vIridescenceThicknessMapUv = ( iridescenceThicknessMapTransform * vec3( IRIDESCENCE_THICKNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_COLORMAP
	vSheenColorMapUv = ( sheenColorMapTransform * vec3( SHEEN_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	vSheenRoughnessMapUv = ( sheenRoughnessMapTransform * vec3( SHEEN_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULARMAP
	vSpecularMapUv = ( specularMapTransform * vec3( SPECULARMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_COLORMAP
	vSpecularColorMapUv = ( specularColorMapTransform * vec3( SPECULAR_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	vSpecularIntensityMapUv = ( specularIntensityMapTransform * vec3( SPECULAR_INTENSITYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_TRANSMISSIONMAP
	vTransmissionMapUv = ( transmissionMapTransform * vec3( TRANSMISSIONMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_THICKNESSMAP
	vThicknessMapUv = ( thicknessMapTransform * vec3( THICKNESSMAP_UV, 1 ) ).xy;
#endif`,tm=`#if defined( USE_ENVMAP ) || defined( DISTANCE ) || defined ( USE_SHADOWMAP ) || defined ( USE_TRANSMISSION ) || NUM_SPOT_LIGHT_COORDS > 0
	vec4 worldPosition = vec4( transformed, 1.0 );
	#ifdef USE_BATCHING
		worldPosition = batchingMatrix * worldPosition;
	#endif
	#ifdef USE_INSTANCING
		worldPosition = instanceMatrix * worldPosition;
	#endif
	worldPosition = modelMatrix * worldPosition;
#endif`,em=`varying vec2 vUv;
uniform mat3 uvTransform;
void main() {
	vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	gl_Position = vec4( position.xy, 1.0, 1.0 );
}`,nm=`uniform sampler2D t2D;
uniform float backgroundIntensity;
varying vec2 vUv;
void main() {
	vec4 texColor = texture2D( t2D, vUv );
	#ifdef DECODE_VIDEO_TEXTURE
		texColor = vec4( mix( pow( texColor.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), texColor.rgb * 0.0773993808, vec3( lessThanEqual( texColor.rgb, vec3( 0.04045 ) ) ) ), texColor.w );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,im=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,sm=`#ifdef ENVMAP_TYPE_CUBE
	uniform samplerCube envMap;
#elif defined( ENVMAP_TYPE_CUBE_UV )
	uniform sampler2D envMap;
#endif
uniform float backgroundBlurriness;
uniform float backgroundIntensity;
uniform mat3 backgroundRotation;
varying vec3 vWorldDirection;
#include <cube_uv_reflection_fragment>
void main() {
	#ifdef ENVMAP_TYPE_CUBE
		vec4 texColor = textureCube( envMap, backgroundRotation * vWorldDirection );
	#elif defined( ENVMAP_TYPE_CUBE_UV )
		vec4 texColor = textureCubeUV( envMap, backgroundRotation * vWorldDirection, backgroundBlurriness );
	#else
		vec4 texColor = vec4( 0.0, 0.0, 0.0, 1.0 );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,rm=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,am=`uniform samplerCube tCube;
uniform float tFlip;
uniform float opacity;
varying vec3 vWorldDirection;
void main() {
	vec4 texColor = textureCube( tCube, vec3( tFlip * vWorldDirection.x, vWorldDirection.yz ) );
	gl_FragColor = texColor;
	gl_FragColor.a *= opacity;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,om=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
varying vec2 vHighPrecisionZW;
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vHighPrecisionZW = gl_Position.zw;
}`,lm=`#if DEPTH_PACKING == 3200
	uniform float opacity;
#endif
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
varying vec2 vHighPrecisionZW;
void main() {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#if DEPTH_PACKING == 3200
		diffuseColor.a = opacity;
	#endif
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <logdepthbuf_fragment>
	#ifdef USE_REVERSED_DEPTH_BUFFER
		float fragCoordZ = vHighPrecisionZW[ 0 ] / vHighPrecisionZW[ 1 ];
	#else
		float fragCoordZ = 0.5 * vHighPrecisionZW[ 0 ] / vHighPrecisionZW[ 1 ] + 0.5;
	#endif
	#if DEPTH_PACKING == 3200
		gl_FragColor = vec4( vec3( 1.0 - fragCoordZ ), opacity );
	#elif DEPTH_PACKING == 3201
		gl_FragColor = packDepthToRGBA( fragCoordZ );
	#elif DEPTH_PACKING == 3202
		gl_FragColor = vec4( packDepthToRGB( fragCoordZ ), 1.0 );
	#elif DEPTH_PACKING == 3203
		gl_FragColor = vec4( packDepthToRG( fragCoordZ ), 0.0, 1.0 );
	#endif
}`,cm=`#define DISTANCE
varying vec3 vWorldPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <worldpos_vertex>
	#include <clipping_planes_vertex>
	vWorldPosition = worldPosition.xyz;
}`,hm=`#define DISTANCE
uniform vec3 referencePosition;
uniform float nearDistance;
uniform float farDistance;
varying vec3 vWorldPosition;
#include <common>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <clipping_planes_pars_fragment>
void main () {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	float dist = length( vWorldPosition - referencePosition );
	dist = ( dist - nearDistance ) / ( farDistance - nearDistance );
	dist = saturate( dist );
	gl_FragColor = vec4( dist, 0.0, 0.0, 1.0 );
}`,dm=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
}`,um=`uniform sampler2D tEquirect;
varying vec3 vWorldDirection;
#include <common>
void main() {
	vec3 direction = normalize( vWorldDirection );
	vec2 sampleUV = equirectUv( direction );
	gl_FragColor = texture2D( tEquirect, sampleUV );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,fm=`uniform float scale;
attribute float lineDistance;
varying float vLineDistance;
#include <common>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	vLineDistance = scale * lineDistance;
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,pm=`uniform vec3 diffuse;
uniform float opacity;
uniform float dashSize;
uniform float totalSize;
varying float vLineDistance;
#include <common>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	if ( mod( vLineDistance, totalSize ) > dashSize ) {
		discard;
	}
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,mm=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#if defined ( USE_ENVMAP ) || defined ( USE_SKINNING )
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinbase_vertex>
		#include <skinnormal_vertex>
		#include <defaultnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <fog_vertex>
}`,gm=`uniform vec3 diffuse;
uniform float opacity;
#ifndef FLAT_SHADED
	varying vec3 vNormal;
#endif
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		reflectedLight.indirectDiffuse += lightMapTexel.rgb * lightMapIntensity * RECIPROCAL_PI;
	#else
		reflectedLight.indirectDiffuse += vec3( 1.0 );
	#endif
	#include <aomap_fragment>
	reflectedLight.indirectDiffuse *= diffuseColor.rgb;
	vec3 outgoingLight = reflectedLight.indirectDiffuse;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,_m=`#define LAMBERT
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,vm=`#define LAMBERT
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_lambert_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_lambert_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,xm=`#define MATCAP
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <displacementmap_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
	vViewPosition = - mvPosition.xyz;
}`,ym=`#define MATCAP
uniform vec3 diffuse;
uniform float opacity;
uniform sampler2D matcap;
varying vec3 vViewPosition;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	vec3 viewDir = normalize( vViewPosition );
	vec3 x = normalize( vec3( viewDir.z, 0.0, - viewDir.x ) );
	vec3 y = cross( viewDir, x );
	vec2 uv = vec2( dot( x, normal ), dot( y, normal ) ) * 0.495 + 0.5;
	#ifdef USE_MATCAP
		vec4 matcapColor = texture2D( matcap, uv );
	#else
		vec4 matcapColor = vec4( vec3( mix( 0.2, 0.8, uv.y ) ), 1.0 );
	#endif
	vec3 outgoingLight = diffuseColor.rgb * matcapColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,bm=`#define NORMAL
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	vViewPosition = - mvPosition.xyz;
#endif
}`,Sm=`#define NORMAL
uniform float opacity;
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <uv_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( 0.0, 0.0, 0.0, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	gl_FragColor = vec4( normalize( normal ) * 0.5 + 0.5, diffuseColor.a );
	#ifdef OPAQUE
		gl_FragColor.a = 1.0;
	#endif
}`,Mm=`#define PHONG
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,wm=`#define PHONG
uniform vec3 diffuse;
uniform vec3 emissive;
uniform vec3 specular;
uniform float shininess;
uniform float opacity;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_phong_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_phong_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + reflectedLight.directSpecular + reflectedLight.indirectSpecular + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Am=`#define STANDARD
varying vec3 vViewPosition;
#ifdef USE_TRANSMISSION
	varying vec3 vWorldPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
#ifdef USE_TRANSMISSION
	vWorldPosition = worldPosition.xyz;
#endif
}`,Em=`#define STANDARD
#ifdef PHYSICAL
	#define IOR
	#define USE_SPECULAR
#endif
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float roughness;
uniform float metalness;
uniform float opacity;
#ifdef IOR
	uniform float ior;
#endif
#ifdef USE_SPECULAR
	uniform float specularIntensity;
	uniform vec3 specularColor;
	#ifdef USE_SPECULAR_COLORMAP
		uniform sampler2D specularColorMap;
	#endif
	#ifdef USE_SPECULAR_INTENSITYMAP
		uniform sampler2D specularIntensityMap;
	#endif
#endif
#ifdef USE_CLEARCOAT
	uniform float clearcoat;
	uniform float clearcoatRoughness;
#endif
#ifdef USE_DISPERSION
	uniform float dispersion;
#endif
#ifdef USE_IRIDESCENCE
	uniform float iridescence;
	uniform float iridescenceIOR;
	uniform float iridescenceThicknessMinimum;
	uniform float iridescenceThicknessMaximum;
#endif
#ifdef USE_SHEEN
	uniform vec3 sheenColor;
	uniform float sheenRoughness;
	#ifdef USE_SHEEN_COLORMAP
		uniform sampler2D sheenColorMap;
	#endif
	#ifdef USE_SHEEN_ROUGHNESSMAP
		uniform sampler2D sheenRoughnessMap;
	#endif
#endif
#ifdef USE_ANISOTROPY
	uniform vec2 anisotropyVector;
	#ifdef USE_ANISOTROPYMAP
		uniform sampler2D anisotropyMap;
	#endif
#endif
varying vec3 vViewPosition;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <iridescence_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_physical_pars_fragment>
#include <transmission_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <clearcoat_pars_fragment>
#include <iridescence_pars_fragment>
#include <roughnessmap_pars_fragment>
#include <metalnessmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <roughnessmap_fragment>
	#include <metalnessmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <clearcoat_normal_fragment_begin>
	#include <clearcoat_normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_physical_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 totalDiffuse = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse;
	vec3 totalSpecular = reflectedLight.directSpecular + reflectedLight.indirectSpecular;
	#include <transmission_fragment>
	vec3 outgoingLight = totalDiffuse + totalSpecular + totalEmissiveRadiance;
	#ifdef USE_SHEEN
 
		outgoingLight = outgoingLight + sheenSpecularDirect + sheenSpecularIndirect;
 
 	#endif
	#ifdef USE_CLEARCOAT
		float dotNVcc = saturate( dot( geometryClearcoatNormal, geometryViewDir ) );
		vec3 Fcc = F_Schlick( material.clearcoatF0, material.clearcoatF90, dotNVcc );
		outgoingLight = outgoingLight * ( 1.0 - material.clearcoat * Fcc ) + ( clearcoatSpecularDirect + clearcoatSpecularIndirect ) * material.clearcoat;
	#endif
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Tm=`#define TOON
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,Cm=`#define TOON
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <gradientmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_toon_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_toon_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Rm=`uniform float size;
uniform float scale;
#include <common>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
#ifdef USE_POINTS_UV
	varying vec2 vUv;
	uniform mat3 uvTransform;
#endif
void main() {
	#ifdef USE_POINTS_UV
		vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	#endif
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	gl_PointSize = size;
	#ifdef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) gl_PointSize *= ( scale / - mvPosition.z );
	#endif
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <fog_vertex>
}`,Im=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <color_pars_fragment>
#include <map_particle_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_particle_fragment>
	#include <color_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,Pm=`#include <common>
#include <batching_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <shadowmap_pars_vertex>
void main() {
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,Lm=`uniform vec3 color;
uniform float opacity;
#include <common>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <logdepthbuf_pars_fragment>
#include <shadowmap_pars_fragment>
#include <shadowmask_pars_fragment>
void main() {
	#include <logdepthbuf_fragment>
	gl_FragColor = vec4( color, opacity * ( 1.0 - getShadowMask() ) );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,Dm=`uniform float rotation;
uniform vec2 center;
#include <common>
#include <uv_pars_vertex>
#include <fog_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	vec4 mvPosition = modelViewMatrix[ 3 ];
	vec2 scale = vec2( length( modelMatrix[ 0 ].xyz ), length( modelMatrix[ 1 ].xyz ) );
	#ifndef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) scale *= - mvPosition.z;
	#endif
	vec2 alignedPosition = ( position.xy - ( center - vec2( 0.5 ) ) ) * scale;
	vec2 rotatedPosition;
	rotatedPosition.x = cos( rotation ) * alignedPosition.x - sin( rotation ) * alignedPosition.y;
	rotatedPosition.y = sin( rotation ) * alignedPosition.x + cos( rotation ) * alignedPosition.y;
	mvPosition.xy += rotatedPosition;
	gl_Position = projectionMatrix * mvPosition;
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,Nm=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,Ot={alphahash_fragment:tf,alphahash_pars_fragment:ef,alphamap_fragment:nf,alphamap_pars_fragment:sf,alphatest_fragment:rf,alphatest_pars_fragment:af,aomap_fragment:of,aomap_pars_fragment:lf,batching_pars_vertex:cf,batching_vertex:hf,begin_vertex:df,beginnormal_vertex:uf,bsdfs:ff,iridescence_fragment:pf,bumpmap_pars_fragment:mf,clipping_planes_fragment:gf,clipping_planes_pars_fragment:_f,clipping_planes_pars_vertex:vf,clipping_planes_vertex:xf,color_fragment:yf,color_pars_fragment:bf,color_pars_vertex:Sf,color_vertex:Mf,common:wf,cube_uv_reflection_fragment:Af,defaultnormal_vertex:Ef,displacementmap_pars_vertex:Tf,displacementmap_vertex:Cf,emissivemap_fragment:Rf,emissivemap_pars_fragment:If,colorspace_fragment:Pf,colorspace_pars_fragment:Lf,envmap_fragment:Df,envmap_common_pars_fragment:Nf,envmap_pars_fragment:Uf,envmap_pars_vertex:Ff,envmap_physical_pars_fragment:qf,envmap_vertex:Of,fog_vertex:Bf,fog_pars_vertex:kf,fog_fragment:zf,fog_pars_fragment:Vf,gradientmap_pars_fragment:Gf,lightmap_pars_fragment:Hf,lights_lambert_fragment:Wf,lights_lambert_pars_fragment:$f,lights_pars_begin:Xf,lights_toon_fragment:Yf,lights_toon_pars_fragment:Zf,lights_phong_fragment:Jf,lights_phong_pars_fragment:Kf,lights_physical_fragment:jf,lights_physical_pars_fragment:Qf,lights_fragment_begin:tp,lights_fragment_maps:ep,lights_fragment_end:np,lightprobes_pars_fragment:ip,logdepthbuf_fragment:sp,logdepthbuf_pars_fragment:rp,logdepthbuf_pars_vertex:ap,logdepthbuf_vertex:op,map_fragment:lp,map_pars_fragment:cp,map_particle_fragment:hp,map_particle_pars_fragment:dp,metalnessmap_fragment:up,metalnessmap_pars_fragment:fp,morphinstance_vertex:pp,morphcolor_vertex:mp,morphnormal_vertex:gp,morphtarget_pars_vertex:_p,morphtarget_vertex:vp,normal_fragment_begin:xp,normal_fragment_maps:yp,normal_pars_fragment:bp,normal_pars_vertex:Sp,normal_vertex:Mp,normalmap_pars_fragment:wp,clearcoat_normal_fragment_begin:Ap,clearcoat_normal_fragment_maps:Ep,clearcoat_pars_fragment:Tp,iridescence_pars_fragment:Cp,opaque_fragment:Rp,packing:Ip,premultiplied_alpha_fragment:Pp,project_vertex:Lp,dithering_fragment:Dp,dithering_pars_fragment:Np,roughnessmap_fragment:Up,roughnessmap_pars_fragment:Fp,shadowmap_pars_fragment:Op,shadowmap_pars_vertex:Bp,shadowmap_vertex:kp,shadowmask_pars_fragment:zp,skinbase_vertex:Vp,skinning_pars_vertex:Gp,skinning_vertex:Hp,skinnormal_vertex:Wp,specularmap_fragment:$p,specularmap_pars_fragment:Xp,tonemapping_fragment:qp,tonemapping_pars_fragment:Yp,transmission_fragment:Zp,transmission_pars_fragment:Jp,uv_pars_fragment:Kp,uv_pars_vertex:jp,uv_vertex:Qp,worldpos_vertex:tm,background_vert:em,background_frag:nm,backgroundCube_vert:im,backgroundCube_frag:sm,cube_vert:rm,cube_frag:am,depth_vert:om,depth_frag:lm,distance_vert:cm,distance_frag:hm,equirect_vert:dm,equirect_frag:um,linedashed_vert:fm,linedashed_frag:pm,meshbasic_vert:mm,meshbasic_frag:gm,meshlambert_vert:_m,meshlambert_frag:vm,meshmatcap_vert:xm,meshmatcap_frag:ym,meshnormal_vert:bm,meshnormal_frag:Sm,meshphong_vert:Mm,meshphong_frag:wm,meshphysical_vert:Am,meshphysical_frag:Em,meshtoon_vert:Tm,meshtoon_frag:Cm,points_vert:Rm,points_frag:Im,shadow_vert:Pm,shadow_frag:Lm,sprite_vert:Dm,sprite_frag:Nm},dt={common:{diffuse:{value:new zt(16777215)},opacity:{value:1},map:{value:null},mapTransform:{value:new Pt},alphaMap:{value:null},alphaMapTransform:{value:new Pt},alphaTest:{value:0}},specularmap:{specularMap:{value:null},specularMapTransform:{value:new Pt}},envmap:{envMap:{value:null},envMapRotation:{value:new Pt},reflectivity:{value:1},ior:{value:1.5},refractionRatio:{value:.98},dfgLUT:{value:null}},aomap:{aoMap:{value:null},aoMapIntensity:{value:1},aoMapTransform:{value:new Pt}},lightmap:{lightMap:{value:null},lightMapIntensity:{value:1},lightMapTransform:{value:new Pt}},bumpmap:{bumpMap:{value:null},bumpMapTransform:{value:new Pt},bumpScale:{value:1}},normalmap:{normalMap:{value:null},normalMapTransform:{value:new Pt},normalScale:{value:new jt(1,1)}},displacementmap:{displacementMap:{value:null},displacementMapTransform:{value:new Pt},displacementScale:{value:1},displacementBias:{value:0}},emissivemap:{emissiveMap:{value:null},emissiveMapTransform:{value:new Pt}},metalnessmap:{metalnessMap:{value:null},metalnessMapTransform:{value:new Pt}},roughnessmap:{roughnessMap:{value:null},roughnessMapTransform:{value:new Pt}},gradientmap:{gradientMap:{value:null}},fog:{fogDensity:{value:25e-5},fogNear:{value:1},fogFar:{value:2e3},fogColor:{value:new zt(16777215)}},lights:{ambientLightColor:{value:[]},lightProbe:{value:[]},directionalLights:{value:[],properties:{direction:{},color:{}}},directionalLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},directionalShadowMatrix:{value:[]},spotLights:{value:[],properties:{color:{},position:{},direction:{},distance:{},coneCos:{},penumbraCos:{},decay:{}}},spotLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},spotLightMap:{value:[]},spotLightMatrix:{value:[]},pointLights:{value:[],properties:{color:{},position:{},decay:{},distance:{}}},pointLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{},shadowCameraNear:{},shadowCameraFar:{}}},pointShadowMatrix:{value:[]},hemisphereLights:{value:[],properties:{direction:{},skyColor:{},groundColor:{}}},rectAreaLights:{value:[],properties:{color:{},position:{},width:{},height:{}}},ltc_1:{value:null},ltc_2:{value:null},probesSH:{value:null},probesMin:{value:new k},probesMax:{value:new k},probesResolution:{value:new k}},points:{diffuse:{value:new zt(16777215)},opacity:{value:1},size:{value:1},scale:{value:1},map:{value:null},alphaMap:{value:null},alphaMapTransform:{value:new Pt},alphaTest:{value:0},uvTransform:{value:new Pt}},sprite:{diffuse:{value:new zt(16777215)},opacity:{value:1},center:{value:new jt(.5,.5)},rotation:{value:0},map:{value:null},mapTransform:{value:new Pt},alphaMap:{value:null},alphaMapTransform:{value:new Pt},alphaTest:{value:0}}},kn={basic:{uniforms:Be([dt.common,dt.specularmap,dt.envmap,dt.aomap,dt.lightmap,dt.fog]),vertexShader:Ot.meshbasic_vert,fragmentShader:Ot.meshbasic_frag},lambert:{uniforms:Be([dt.common,dt.specularmap,dt.envmap,dt.aomap,dt.lightmap,dt.emissivemap,dt.bumpmap,dt.normalmap,dt.displacementmap,dt.fog,dt.lights,{emissive:{value:new zt(0)},envMapIntensity:{value:1}}]),vertexShader:Ot.meshlambert_vert,fragmentShader:Ot.meshlambert_frag},phong:{uniforms:Be([dt.common,dt.specularmap,dt.envmap,dt.aomap,dt.lightmap,dt.emissivemap,dt.bumpmap,dt.normalmap,dt.displacementmap,dt.fog,dt.lights,{emissive:{value:new zt(0)},specular:{value:new zt(1118481)},shininess:{value:30},envMapIntensity:{value:1}}]),vertexShader:Ot.meshphong_vert,fragmentShader:Ot.meshphong_frag},standard:{uniforms:Be([dt.common,dt.envmap,dt.aomap,dt.lightmap,dt.emissivemap,dt.bumpmap,dt.normalmap,dt.displacementmap,dt.roughnessmap,dt.metalnessmap,dt.fog,dt.lights,{emissive:{value:new zt(0)},roughness:{value:1},metalness:{value:0},envMapIntensity:{value:1}}]),vertexShader:Ot.meshphysical_vert,fragmentShader:Ot.meshphysical_frag},toon:{uniforms:Be([dt.common,dt.aomap,dt.lightmap,dt.emissivemap,dt.bumpmap,dt.normalmap,dt.displacementmap,dt.gradientmap,dt.fog,dt.lights,{emissive:{value:new zt(0)}}]),vertexShader:Ot.meshtoon_vert,fragmentShader:Ot.meshtoon_frag},matcap:{uniforms:Be([dt.common,dt.bumpmap,dt.normalmap,dt.displacementmap,dt.fog,{matcap:{value:null}}]),vertexShader:Ot.meshmatcap_vert,fragmentShader:Ot.meshmatcap_frag},points:{uniforms:Be([dt.points,dt.fog]),vertexShader:Ot.points_vert,fragmentShader:Ot.points_frag},dashed:{uniforms:Be([dt.common,dt.fog,{scale:{value:1},dashSize:{value:1},totalSize:{value:2}}]),vertexShader:Ot.linedashed_vert,fragmentShader:Ot.linedashed_frag},depth:{uniforms:Be([dt.common,dt.displacementmap]),vertexShader:Ot.depth_vert,fragmentShader:Ot.depth_frag},normal:{uniforms:Be([dt.common,dt.bumpmap,dt.normalmap,dt.displacementmap,{opacity:{value:1}}]),vertexShader:Ot.meshnormal_vert,fragmentShader:Ot.meshnormal_frag},sprite:{uniforms:Be([dt.sprite,dt.fog]),vertexShader:Ot.sprite_vert,fragmentShader:Ot.sprite_frag},background:{uniforms:{uvTransform:{value:new Pt},t2D:{value:null},backgroundIntensity:{value:1}},vertexShader:Ot.background_vert,fragmentShader:Ot.background_frag},backgroundCube:{uniforms:{envMap:{value:null},backgroundBlurriness:{value:0},backgroundIntensity:{value:1},backgroundRotation:{value:new Pt}},vertexShader:Ot.backgroundCube_vert,fragmentShader:Ot.backgroundCube_frag},cube:{uniforms:{tCube:{value:null},tFlip:{value:-1},opacity:{value:1}},vertexShader:Ot.cube_vert,fragmentShader:Ot.cube_frag},equirect:{uniforms:{tEquirect:{value:null}},vertexShader:Ot.equirect_vert,fragmentShader:Ot.equirect_frag},distance:{uniforms:Be([dt.common,dt.displacementmap,{referencePosition:{value:new k},nearDistance:{value:1},farDistance:{value:1e3}}]),vertexShader:Ot.distance_vert,fragmentShader:Ot.distance_frag},shadow:{uniforms:Be([dt.lights,dt.fog,{color:{value:new zt(0)},opacity:{value:1}}]),vertexShader:Ot.shadow_vert,fragmentShader:Ot.shadow_frag}};kn.physical={uniforms:Be([kn.standard.uniforms,{clearcoat:{value:0},clearcoatMap:{value:null},clearcoatMapTransform:{value:new Pt},clearcoatNormalMap:{value:null},clearcoatNormalMapTransform:{value:new Pt},clearcoatNormalScale:{value:new jt(1,1)},clearcoatRoughness:{value:0},clearcoatRoughnessMap:{value:null},clearcoatRoughnessMapTransform:{value:new Pt},dispersion:{value:0},iridescence:{value:0},iridescenceMap:{value:null},iridescenceMapTransform:{value:new Pt},iridescenceIOR:{value:1.3},iridescenceThicknessMinimum:{value:100},iridescenceThicknessMaximum:{value:400},iridescenceThicknessMap:{value:null},iridescenceThicknessMapTransform:{value:new Pt},sheen:{value:0},sheenColor:{value:new zt(0)},sheenColorMap:{value:null},sheenColorMapTransform:{value:new Pt},sheenRoughness:{value:1},sheenRoughnessMap:{value:null},sheenRoughnessMapTransform:{value:new Pt},transmission:{value:0},transmissionMap:{value:null},transmissionMapTransform:{value:new Pt},transmissionSamplerSize:{value:new jt},transmissionSamplerMap:{value:null},thickness:{value:0},thicknessMap:{value:null},thicknessMapTransform:{value:new Pt},attenuationDistance:{value:0},attenuationColor:{value:new zt(0)},specularColor:{value:new zt(1,1,1)},specularColorMap:{value:null},specularColorMapTransform:{value:new Pt},specularIntensity:{value:1},specularIntensityMap:{value:null},specularIntensityMapTransform:{value:new Pt},anisotropyVector:{value:new jt},anisotropyMap:{value:null},anisotropyMapTransform:{value:new Pt}}]),vertexShader:Ot.meshphysical_vert,fragmentShader:Ot.meshphysical_frag};var Po={r:0,b:0,g:0},Um=new fe,kd=new Pt;kd.set(-1,0,0,0,1,0,0,0,1);function Fm(i,t,e,n,s,r){let a=new zt(0),o=s===!0?0:1,l,c,d=null,u=0,h=null;function p(S){let M=S.isScene===!0?S.background:null;if(M&&M.isTexture){let E=S.backgroundBlurriness>0;M=t.get(M,E)}return M}function _(S){let M=!1,E=p(S);E===null?m(a,o):E&&E.isColor&&(m(E,1),M=!0);let R=i.xr.getEnvironmentBlendMode();R==="additive"?e.buffers.color.setClear(0,0,0,1,r):R==="alpha-blend"&&e.buffers.color.setClear(0,0,0,0,r),(i.autoClear||M)&&(e.buffers.depth.setTest(!0),e.buffers.depth.setMask(!0),e.buffers.color.setMask(!0),i.clear(i.autoClearColor,i.autoClearDepth,i.autoClearStencil))}function y(S,M){let E=p(M);E&&(E.isCubeTexture||E.mapping===lr)?(c===void 0&&(c=new ke(new pi(1,1,1),new je({name:"BackgroundCubeMaterial",uniforms:Wi(kn.backgroundCube.uniforms),vertexShader:kn.backgroundCube.vertexShader,fragmentShader:kn.backgroundCube.fragmentShader,side:ze,depthTest:!1,depthWrite:!1,fog:!1,allowOverride:!1})),c.geometry.deleteAttribute("normal"),c.geometry.deleteAttribute("uv"),c.onBeforeRender=function(R,w,I){this.matrixWorld.copyPosition(I.matrixWorld)},Object.defineProperty(c.material,"envMap",{get:function(){return this.uniforms.envMap.value}}),n.update(c)),c.material.uniforms.envMap.value=E,c.material.uniforms.backgroundBlurriness.value=M.backgroundBlurriness,c.material.uniforms.backgroundIntensity.value=M.backgroundIntensity,c.material.uniforms.backgroundRotation.value.setFromMatrix4(Um.makeRotationFromEuler(M.backgroundRotation)).transpose(),E.isCubeTexture&&E.isRenderTargetTexture===!1&&c.material.uniforms.backgroundRotation.value.premultiply(kd),c.material.toneMapped=$t.getTransfer(E.colorSpace)!==te,(d!==E||u!==E.version||h!==i.toneMapping)&&(c.material.needsUpdate=!0,d=E,u=E.version,h=i.toneMapping),c.layers.enableAll(),S.unshift(c,c.geometry,c.material,0,0,null)):E&&E.isTexture&&(l===void 0&&(l=new ke(new Gi(2,2),new je({name:"BackgroundMaterial",uniforms:Wi(kn.background.uniforms),vertexShader:kn.background.vertexShader,fragmentShader:kn.background.fragmentShader,side:Zn,depthTest:!1,depthWrite:!1,fog:!1,allowOverride:!1})),l.geometry.deleteAttribute("normal"),Object.defineProperty(l.material,"map",{get:function(){return this.uniforms.t2D.value}}),n.update(l)),l.material.uniforms.t2D.value=E,l.material.uniforms.backgroundIntensity.value=M.backgroundIntensity,l.material.toneMapped=$t.getTransfer(E.colorSpace)!==te,E.matrixAutoUpdate===!0&&E.updateMatrix(),l.material.uniforms.uvTransform.value.copy(E.matrix),(d!==E||u!==E.version||h!==i.toneMapping)&&(l.material.needsUpdate=!0,d=E,u=E.version,h=i.toneMapping),l.layers.enableAll(),S.unshift(l,l.geometry,l.material,0,0,null))}function m(S,M){S.getRGB(Po,dc(i)),e.buffers.color.setClear(Po.r,Po.g,Po.b,M,r)}function f(){c!==void 0&&(c.geometry.dispose(),c.material.dispose(),c=void 0),l!==void 0&&(l.geometry.dispose(),l.material.dispose(),l=void 0)}return{getClearColor:function(){return a},setClearColor:function(S,M=1){a.set(S),o=M,m(a,o)},getClearAlpha:function(){return o},setClearAlpha:function(S){o=S,m(a,o)},render:_,addToRenderList:y,dispose:f}}function Om(i,t){let e=i.getParameter(i.MAX_VERTEX_ATTRIBS),n={},s=h(null),r=s,a=!1;function o(T,O,H,$,U){let L=!1,B=u(T,$,H,O);r!==B&&(r=B,c(r.object)),L=p(T,$,H,U),L&&_(T,$,H,U),U!==null&&t.update(U,i.ELEMENT_ARRAY_BUFFER),(L||a)&&(a=!1,E(T,O,H,$),U!==null&&i.bindBuffer(i.ELEMENT_ARRAY_BUFFER,t.get(U).buffer))}function l(){return i.createVertexArray()}function c(T){return i.bindVertexArray(T)}function d(T){return i.deleteVertexArray(T)}function u(T,O,H,$){let U=$.wireframe===!0,L=n[O.id];L===void 0&&(L={},n[O.id]=L);let B=T.isInstancedMesh===!0?T.id:0,K=L[B];K===void 0&&(K={},L[B]=K);let tt=K[H.id];tt===void 0&&(tt={},K[H.id]=tt);let ct=tt[U];return ct===void 0&&(ct=h(l()),tt[U]=ct),ct}function h(T){let O=[],H=[],$=[];for(let U=0;U<e;U++)O[U]=0,H[U]=0,$[U]=0;return{geometry:null,program:null,wireframe:!1,newAttributes:O,enabledAttributes:H,attributeDivisors:$,object:T,attributes:{},index:null}}function p(T,O,H,$){let U=r.attributes,L=O.attributes,B=0,K=H.getAttributes();for(let tt in K)if(K[tt].location>=0){let bt=U[tt],vt=L[tt];if(vt===void 0&&(tt==="instanceMatrix"&&T.instanceMatrix&&(vt=T.instanceMatrix),tt==="instanceColor"&&T.instanceColor&&(vt=T.instanceColor)),bt===void 0||bt.attribute!==vt||vt&&bt.data!==vt.data)return!0;B++}return r.attributesNum!==B||r.index!==$}function _(T,O,H,$){let U={},L=O.attributes,B=0,K=H.getAttributes();for(let tt in K)if(K[tt].location>=0){let bt=L[tt];bt===void 0&&(tt==="instanceMatrix"&&T.instanceMatrix&&(bt=T.instanceMatrix),tt==="instanceColor"&&T.instanceColor&&(bt=T.instanceColor));let vt={};vt.attribute=bt,bt&&bt.data&&(vt.data=bt.data),U[tt]=vt,B++}r.attributes=U,r.attributesNum=B,r.index=$}function y(){let T=r.newAttributes;for(let O=0,H=T.length;O<H;O++)T[O]=0}function m(T){f(T,0)}function f(T,O){let H=r.newAttributes,$=r.enabledAttributes,U=r.attributeDivisors;H[T]=1,$[T]===0&&(i.enableVertexAttribArray(T),$[T]=1),U[T]!==O&&(i.vertexAttribDivisor(T,O),U[T]=O)}function S(){let T=r.newAttributes,O=r.enabledAttributes;for(let H=0,$=O.length;H<$;H++)O[H]!==T[H]&&(i.disableVertexAttribArray(H),O[H]=0)}function M(T,O,H,$,U,L,B){B===!0?i.vertexAttribIPointer(T,O,H,U,L):i.vertexAttribPointer(T,O,H,$,U,L)}function E(T,O,H,$){y();let U=$.attributes,L=H.getAttributes(),B=O.defaultAttributeValues;for(let K in L){let tt=L[K];if(tt.location>=0){let ct=U[K];if(ct===void 0&&(K==="instanceMatrix"&&T.instanceMatrix&&(ct=T.instanceMatrix),K==="instanceColor"&&T.instanceColor&&(ct=T.instanceColor)),ct!==void 0){let bt=ct.normalized,vt=ct.itemSize,Gt=t.get(ct);if(Gt===void 0)continue;let Qt=Gt.buffer,Ut=Gt.type,J=Gt.bytesPerElement,pt=Ut===i.INT||Ut===i.UNSIGNED_INT||ct.gpuType===$a;if(ct.isInterleavedBufferAttribute){let rt=ct.data,Tt=rt.stride,Lt=ct.offset;if(rt.isInstancedInterleavedBuffer){for(let Rt=0;Rt<tt.locationSize;Rt++)f(tt.location+Rt,rt.meshPerAttribute);T.isInstancedMesh!==!0&&$._maxInstanceCount===void 0&&($._maxInstanceCount=rt.meshPerAttribute*rt.count)}else for(let Rt=0;Rt<tt.locationSize;Rt++)m(tt.location+Rt);i.bindBuffer(i.ARRAY_BUFFER,Qt);for(let Rt=0;Rt<tt.locationSize;Rt++)M(tt.location+Rt,vt/tt.locationSize,Ut,bt,Tt*J,(Lt+vt/tt.locationSize*Rt)*J,pt)}else{if(ct.isInstancedBufferAttribute){for(let rt=0;rt<tt.locationSize;rt++)f(tt.location+rt,ct.meshPerAttribute);T.isInstancedMesh!==!0&&$._maxInstanceCount===void 0&&($._maxInstanceCount=ct.meshPerAttribute*ct.count)}else for(let rt=0;rt<tt.locationSize;rt++)m(tt.location+rt);i.bindBuffer(i.ARRAY_BUFFER,Qt);for(let rt=0;rt<tt.locationSize;rt++)M(tt.location+rt,vt/tt.locationSize,Ut,bt,vt*J,vt/tt.locationSize*rt*J,pt)}}else if(B!==void 0){let bt=B[K];if(bt!==void 0)switch(bt.length){case 2:i.vertexAttrib2fv(tt.location,bt);break;case 3:i.vertexAttrib3fv(tt.location,bt);break;case 4:i.vertexAttrib4fv(tt.location,bt);break;default:i.vertexAttrib1fv(tt.location,bt)}}}}S()}function R(){A();for(let T in n){let O=n[T];for(let H in O){let $=O[H];for(let U in $){let L=$[U];for(let B in L)d(L[B].object),delete L[B];delete $[U]}}delete n[T]}}function w(T){if(n[T.id]===void 0)return;let O=n[T.id];for(let H in O){let $=O[H];for(let U in $){let L=$[U];for(let B in L)d(L[B].object),delete L[B];delete $[U]}}delete n[T.id]}function I(T){for(let O in n){let H=n[O];for(let $ in H){let U=H[$];if(U[T.id]===void 0)continue;let L=U[T.id];for(let B in L)d(L[B].object),delete L[B];delete U[T.id]}}}function v(T){for(let O in n){let H=n[O],$=T.isInstancedMesh===!0?T.id:0,U=H[$];if(U!==void 0){for(let L in U){let B=U[L];for(let K in B)d(B[K].object),delete B[K];delete U[L]}delete H[$],Object.keys(H).length===0&&delete n[O]}}}function A(){D(),a=!0,r!==s&&(r=s,c(r.object))}function D(){s.geometry=null,s.program=null,s.wireframe=!1}return{setup:o,reset:A,resetDefaultState:D,dispose:R,releaseStatesOfGeometry:w,releaseStatesOfObject:v,releaseStatesOfProgram:I,initAttributes:y,enableAttribute:m,disableUnusedAttributes:S}}function Bm(i,t,e){let n;function s(l){n=l}function r(l,c){i.drawArrays(n,l,c),e.update(c,n,1)}function a(l,c,d){d!==0&&(i.drawArraysInstanced(n,l,c,d),e.update(c,n,d))}function o(l,c,d){if(d===0)return;t.get("WEBGL_multi_draw").multiDrawArraysWEBGL(n,l,0,c,0,d);let h=0;for(let p=0;p<d;p++)h+=c[p];e.update(h,n,1)}this.setMode=s,this.render=r,this.renderInstances=a,this.renderMultiDraw=o}function km(i,t,e,n){let s;function r(){if(s!==void 0)return s;if(t.has("EXT_texture_filter_anisotropic")===!0){let I=t.get("EXT_texture_filter_anisotropic");s=i.getParameter(I.MAX_TEXTURE_MAX_ANISOTROPY_EXT)}else s=0;return s}function a(I){return!(I!==hn&&n.convert(I)!==i.getParameter(i.IMPLEMENTATION_COLOR_READ_FORMAT))}function o(I){let v=I===On&&(t.has("EXT_color_buffer_half_float")||t.has("EXT_color_buffer_float"));return!(I!==He&&n.convert(I)!==i.getParameter(i.IMPLEMENTATION_COLOR_READ_TYPE)&&I!==bn&&!v)}function l(I){if(I==="highp"){if(i.getShaderPrecisionFormat(i.VERTEX_SHADER,i.HIGH_FLOAT).precision>0&&i.getShaderPrecisionFormat(i.FRAGMENT_SHADER,i.HIGH_FLOAT).precision>0)return"highp";I="mediump"}return I==="mediump"&&i.getShaderPrecisionFormat(i.VERTEX_SHADER,i.MEDIUM_FLOAT).precision>0&&i.getShaderPrecisionFormat(i.FRAGMENT_SHADER,i.MEDIUM_FLOAT).precision>0?"mediump":"lowp"}let c=e.precision!==void 0?e.precision:"highp",d=l(c);d!==c&&(Et("WebGLRenderer:",c,"not supported, using",d,"instead."),c=d);let u=e.logarithmicDepthBuffer===!0,h=e.reversedDepthBuffer===!0&&t.has("EXT_clip_control");e.reversedDepthBuffer===!0&&h===!1&&Et("WebGLRenderer: Unable to use reversed depth buffer due to missing EXT_clip_control extension. Fallback to default depth buffer.");let p=i.getParameter(i.MAX_TEXTURE_IMAGE_UNITS),_=i.getParameter(i.MAX_VERTEX_TEXTURE_IMAGE_UNITS),y=i.getParameter(i.MAX_TEXTURE_SIZE),m=i.getParameter(i.MAX_CUBE_MAP_TEXTURE_SIZE),f=i.getParameter(i.MAX_VERTEX_ATTRIBS),S=i.getParameter(i.MAX_VERTEX_UNIFORM_VECTORS),M=i.getParameter(i.MAX_VARYING_VECTORS),E=i.getParameter(i.MAX_FRAGMENT_UNIFORM_VECTORS),R=i.getParameter(i.MAX_SAMPLES),w=i.getParameter(i.SAMPLES);return{isWebGL2:!0,getMaxAnisotropy:r,getMaxPrecision:l,textureFormatReadable:a,textureTypeReadable:o,precision:c,logarithmicDepthBuffer:u,reversedDepthBuffer:h,maxTextures:p,maxVertexTextures:_,maxTextureSize:y,maxCubemapSize:m,maxAttributes:f,maxVertexUniforms:S,maxVaryings:M,maxFragmentUniforms:E,maxSamples:R,samples:w}}function zm(i){let t=this,e=null,n=0,s=!1,r=!1,a=new In,o=new Pt,l={value:null,needsUpdate:!1};this.uniform=l,this.numPlanes=0,this.numIntersection=0,this.init=function(u,h){let p=u.length!==0||h||n!==0||s;return s=h,n=u.length,p},this.beginShadows=function(){r=!0,d(null)},this.endShadows=function(){r=!1},this.setGlobalState=function(u,h){e=d(u,h,0)},this.setState=function(u,h,p){let _=u.clippingPlanes,y=u.clipIntersection,m=u.clipShadows,f=i.get(u);if(!s||_===null||_.length===0||r&&!m)r?d(null):c();else{let S=r?0:n,M=S*4,E=f.clippingState||null;l.value=E,E=d(_,h,M,p);for(let R=0;R!==M;++R)E[R]=e[R];f.clippingState=E,this.numIntersection=y?this.numPlanes:0,this.numPlanes+=S}};function c(){l.value!==e&&(l.value=e,l.needsUpdate=n>0),t.numPlanes=n,t.numIntersection=0}function d(u,h,p,_){let y=u!==null?u.length:0,m=null;if(y!==0){if(m=l.value,_!==!0||m===null){let f=p+y*4,S=h.matrixWorldInverse;o.getNormalMatrix(S),(m===null||m.length<f)&&(m=new Float32Array(f));for(let M=0,E=p;M!==y;++M,E+=4)a.copy(u[M]).applyMatrix4(S,o),a.normal.toArray(m,E),m[E+3]=a.constant}l.value=m,l.needsUpdate=!0}return t.numPlanes=y,t.numIntersection=0,m}}var Si=4,gd=[.125,.215,.35,.446,.526,.582],$i=20,Vm=256,_r=new _s,_d=new zt,mc=null,gc=0,_c=0,vc=!1,Gm=new k,Do=class{constructor(t){this._renderer=t,this._pingPongRenderTarget=null,this._lodMax=0,this._cubeSize=0,this._sizeLods=[],this._sigmas=[],this._lodMeshes=[],this._backgroundBox=null,this._cubemapMaterial=null,this._equirectMaterial=null,this._blurMaterial=null,this._ggxMaterial=null}fromScene(t,e=0,n=.1,s=100,r={}){let{size:a=256,position:o=Gm}=r;mc=this._renderer.getRenderTarget(),gc=this._renderer.getActiveCubeFace(),_c=this._renderer.getActiveMipmapLevel(),vc=this._renderer.xr.enabled,this._renderer.xr.enabled=!1,this._setSize(a);let l=this._allocateTargets();return l.depthBuffer=!0,this._sceneToCubeUV(t,n,s,l,o),e>0&&this._blur(l,0,0,e),this._applyPMREM(l),this._cleanup(l),l}fromEquirectangular(t,e=null){return this._fromTexture(t,e)}fromCubemap(t,e=null){return this._fromTexture(t,e)}compileCubemapShader(){this._cubemapMaterial===null&&(this._cubemapMaterial=yd(),this._compileMaterial(this._cubemapMaterial))}compileEquirectangularShader(){this._equirectMaterial===null&&(this._equirectMaterial=xd(),this._compileMaterial(this._equirectMaterial))}dispose(){this._dispose(),this._cubemapMaterial!==null&&this._cubemapMaterial.dispose(),this._equirectMaterial!==null&&this._equirectMaterial.dispose(),this._backgroundBox!==null&&(this._backgroundBox.geometry.dispose(),this._backgroundBox.material.dispose())}_setSize(t){this._lodMax=Math.floor(Math.log2(t)),this._cubeSize=Math.pow(2,this._lodMax)}_dispose(){this._blurMaterial!==null&&this._blurMaterial.dispose(),this._ggxMaterial!==null&&this._ggxMaterial.dispose(),this._pingPongRenderTarget!==null&&this._pingPongRenderTarget.dispose();for(let t=0;t<this._lodMeshes.length;t++)this._lodMeshes[t].geometry.dispose()}_cleanup(t){this._renderer.setRenderTarget(mc,gc,_c),this._renderer.xr.enabled=vc,t.scissorTest=!1,bs(t,0,0,t.width,t.height)}_fromTexture(t,e){t.mapping===vi||t.mapping===Hi?this._setSize(t.image.length===0?16:t.image[0].width||t.image[0].image.width):this._setSize(t.image.width/4),mc=this._renderer.getRenderTarget(),gc=this._renderer.getActiveCubeFace(),_c=this._renderer.getActiveMipmapLevel(),vc=this._renderer.xr.enabled,this._renderer.xr.enabled=!1;let n=e||this._allocateTargets();return this._textureToCubeUV(t,n),this._applyPMREM(n),this._cleanup(n),n}_allocateTargets(){let t=3*Math.max(this._cubeSize,112),e=4*this._cubeSize,n={magFilter:Pe,minFilter:Pe,generateMipmaps:!1,type:On,format:hn,colorSpace:Gs,depthBuffer:!1},s=vd(t,e,n);if(this._pingPongRenderTarget===null||this._pingPongRenderTarget.width!==t||this._pingPongRenderTarget.height!==e){this._pingPongRenderTarget!==null&&this._dispose(),this._pingPongRenderTarget=vd(t,e,n);let{_lodMax:r}=this;({lodMeshes:this._lodMeshes,sizeLods:this._sizeLods,sigmas:this._sigmas}=Hm(r)),this._blurMaterial=$m(r,t,e),this._ggxMaterial=Wm(r,t,e)}return s}_compileMaterial(t){let e=new ke(new cn,t);this._renderer.compile(e,_r)}_sceneToCubeUV(t,e,n,s,r){let l=new Ue(90,1,e,n),c=[1,-1,1,1,1,1],d=[1,1,1,-1,-1,-1],u=this._renderer,h=u.autoClear,p=u.toneMapping;u.getClearColor(_d),u.toneMapping=xn,u.autoClear=!1,u.state.buffers.depth.getReversed()&&(u.setRenderTarget(s),u.clearDepth(),u.setRenderTarget(null)),this._backgroundBox===null&&(this._backgroundBox=new ke(new pi,new Ks({name:"PMREM.Background",side:ze,depthWrite:!1,depthTest:!1})));let y=this._backgroundBox,m=y.material,f=!1,S=t.background;S?S.isColor&&(m.color.copy(S),t.background=null,f=!0):(m.color.copy(_d),f=!0);for(let M=0;M<6;M++){let E=M%3;E===0?(l.up.set(0,c[M],0),l.position.set(r.x,r.y,r.z),l.lookAt(r.x+d[M],r.y,r.z)):E===1?(l.up.set(0,0,c[M]),l.position.set(r.x,r.y,r.z),l.lookAt(r.x,r.y+d[M],r.z)):(l.up.set(0,c[M],0),l.position.set(r.x,r.y,r.z),l.lookAt(r.x,r.y,r.z+d[M]));let R=this._cubeSize;bs(s,E*R,M>2?R:0,R,R),u.setRenderTarget(s),f&&u.render(y,l),u.render(t,l)}u.toneMapping=p,u.autoClear=h,t.background=S}_textureToCubeUV(t,e){let n=this._renderer,s=t.mapping===vi||t.mapping===Hi;s?(this._cubemapMaterial===null&&(this._cubemapMaterial=yd()),this._cubemapMaterial.uniforms.flipEnvMap.value=t.isRenderTargetTexture===!1?-1:1):this._equirectMaterial===null&&(this._equirectMaterial=xd());let r=s?this._cubemapMaterial:this._equirectMaterial,a=this._lodMeshes[0];a.material=r;let o=r.uniforms;o.envMap.value=t;let l=this._cubeSize;bs(e,0,0,3*l,2*l),n.setRenderTarget(e),n.render(a,_r)}_applyPMREM(t){let e=this._renderer,n=e.autoClear;e.autoClear=!1;let s=this._lodMeshes.length;for(let r=1;r<s;r++)this._applyGGXFilter(t,r-1,r);e.autoClear=n}_applyGGXFilter(t,e,n){let s=this._renderer,r=this._pingPongRenderTarget,a=this._ggxMaterial,o=this._lodMeshes[n];o.material=a;let l=a.uniforms,c=n/(this._lodMeshes.length-1),d=e/(this._lodMeshes.length-1),u=Math.sqrt(c*c-d*d),h=0+c*1.25,p=u*h,{_lodMax:_}=this,y=this._sizeLods[n],m=3*y*(n>_-Si?n-_+Si:0),f=4*(this._cubeSize-y);l.envMap.value=t.texture,l.roughness.value=p,l.mipInt.value=_-e,bs(r,m,f,3*y,2*y),s.setRenderTarget(r),s.render(o,_r),l.envMap.value=r.texture,l.roughness.value=0,l.mipInt.value=_-n,bs(t,m,f,3*y,2*y),s.setRenderTarget(t),s.render(o,_r)}_blur(t,e,n,s,r){let a=this._pingPongRenderTarget;this._halfBlur(t,a,e,n,s,"latitudinal",r),this._halfBlur(a,t,n,n,s,"longitudinal",r)}_halfBlur(t,e,n,s,r,a,o){let l=this._renderer,c=this._blurMaterial;a!=="latitudinal"&&a!=="longitudinal"&&Ct("blur direction must be either latitudinal or longitudinal!");let d=3,u=this._lodMeshes[s];u.material=c;let h=c.uniforms,p=this._sizeLods[n]-1,_=isFinite(r)?Math.PI/(2*p):2*Math.PI/(2*$i-1),y=r/_,m=isFinite(r)?1+Math.floor(d*y):$i;m>$i&&Et(`sigmaRadians, ${r}, is too large and will clip, as it requested ${m} samples when the maximum is set to ${$i}`);let f=[],S=0;for(let I=0;I<$i;++I){let v=I/y,A=Math.exp(-v*v/2);f.push(A),I===0?S+=A:I<m&&(S+=2*A)}for(let I=0;I<f.length;I++)f[I]=f[I]/S;h.envMap.value=t.texture,h.samples.value=m,h.weights.value=f,h.latitudinal.value=a==="latitudinal",o&&(h.poleAxis.value=o);let{_lodMax:M}=this;h.dTheta.value=_,h.mipInt.value=M-n;let E=this._sizeLods[s],R=3*E*(s>M-Si?s-M+Si:0),w=4*(this._cubeSize-E);bs(e,R,w,3*E,2*E),l.setRenderTarget(e),l.render(u,_r)}};function Hm(i){let t=[],e=[],n=[],s=i,r=i-Si+1+gd.length;for(let a=0;a<r;a++){let o=Math.pow(2,s);t.push(o);let l=1/o;a>i-Si?l=gd[a-i+Si-1]:a===0&&(l=0),e.push(l);let c=1/(o-2),d=-c,u=1+c,h=[d,d,u,d,u,u,d,d,u,u,d,u],p=6,_=6,y=3,m=2,f=1,S=new Float32Array(y*_*p),M=new Float32Array(m*_*p),E=new Float32Array(f*_*p);for(let w=0;w<p;w++){let I=w%3*2/3-1,v=w>2?0:-1,A=[I,v,0,I+2/3,v,0,I+2/3,v+1,0,I,v,0,I+2/3,v+1,0,I,v+1,0];S.set(A,y*_*w),M.set(h,m*_*w);let D=[w,w,w,w,w,w];E.set(D,f*_*w)}let R=new cn;R.setAttribute("position",new Je(S,y)),R.setAttribute("uv",new Je(M,m)),R.setAttribute("faceIndex",new Je(E,f)),n.push(new ke(R,null)),s>Si&&s--}return{lodMeshes:n,sizeLods:t,sigmas:e}}function vd(i,t,e){let n=new Ke(i,t,e);return n.texture.mapping=lr,n.texture.name="PMREM.cubeUv",n.scissorTest=!0,n}function bs(i,t,e,n,s){i.viewport.set(t,e,n,s),i.scissor.set(t,e,n,s)}function Wm(i,t,e){return new je({name:"PMREMGGXConvolution",defines:{GGX_SAMPLES:Vm,CUBEUV_TEXEL_WIDTH:1/t,CUBEUV_TEXEL_HEIGHT:1/e,CUBEUV_MAX_MIP:`${i}.0`},uniforms:{envMap:{value:null},roughness:{value:0},mipInt:{value:0}},vertexShader:Fo(),fragmentShader:`

			precision highp float;
			precision highp int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;
			uniform float roughness;
			uniform float mipInt;

			#define ENVMAP_TYPE_CUBE_UV
			#include <cube_uv_reflection_fragment>

			#define PI 3.14159265359

			// Van der Corput radical inverse
			float radicalInverse_VdC(uint bits) {
				bits = (bits << 16u) | (bits >> 16u);
				bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
				bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
				bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
				bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
				return float(bits) * 2.3283064365386963e-10; // / 0x100000000
			}

			// Hammersley sequence
			vec2 hammersley(uint i, uint N) {
				return vec2(float(i) / float(N), radicalInverse_VdC(i));
			}

			// GGX VNDF importance sampling (Eric Heitz 2018)
			// "Sampling the GGX Distribution of Visible Normals"
			// https://jcgt.org/published/0007/04/01/
			vec3 importanceSampleGGX_VNDF(vec2 Xi, vec3 V, float roughness) {
				float alpha = roughness * roughness;

				// Section 4.1: Orthonormal basis
				vec3 T1 = vec3(1.0, 0.0, 0.0);
				vec3 T2 = cross(V, T1);

				// Section 4.2: Parameterization of projected area
				float r = sqrt(Xi.x);
				float phi = 2.0 * PI * Xi.y;
				float t1 = r * cos(phi);
				float t2 = r * sin(phi);
				float s = 0.5 * (1.0 + V.z);
				t2 = (1.0 - s) * sqrt(1.0 - t1 * t1) + s * t2;

				// Section 4.3: Reprojection onto hemisphere
				vec3 Nh = t1 * T1 + t2 * T2 + sqrt(max(0.0, 1.0 - t1 * t1 - t2 * t2)) * V;

				// Section 3.4: Transform back to ellipsoid configuration
				return normalize(vec3(alpha * Nh.x, alpha * Nh.y, max(0.0, Nh.z)));
			}

			void main() {
				vec3 N = normalize(vOutputDirection);
				vec3 V = N; // Assume view direction equals normal for pre-filtering

				vec3 prefilteredColor = vec3(0.0);
				float totalWeight = 0.0;

				// For very low roughness, just sample the environment directly
				if (roughness < 0.001) {
					gl_FragColor = vec4(bilinearCubeUV(envMap, N, mipInt), 1.0);
					return;
				}

				// Tangent space basis for VNDF sampling
				vec3 up = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
				vec3 tangent = normalize(cross(up, N));
				vec3 bitangent = cross(N, tangent);

				for(uint i = 0u; i < uint(GGX_SAMPLES); i++) {
					vec2 Xi = hammersley(i, uint(GGX_SAMPLES));

					// For PMREM, V = N, so in tangent space V is always (0, 0, 1)
					vec3 H_tangent = importanceSampleGGX_VNDF(Xi, vec3(0.0, 0.0, 1.0), roughness);

					// Transform H back to world space
					vec3 H = normalize(tangent * H_tangent.x + bitangent * H_tangent.y + N * H_tangent.z);
					vec3 L = normalize(2.0 * dot(V, H) * H - V);

					float NdotL = max(dot(N, L), 0.0);

					if(NdotL > 0.0) {
						// Sample environment at fixed mip level
						// VNDF importance sampling handles the distribution filtering
						vec3 sampleColor = bilinearCubeUV(envMap, L, mipInt);

						// Weight by NdotL for the split-sum approximation
						// VNDF PDF naturally accounts for the visible microfacet distribution
						prefilteredColor += sampleColor * NdotL;
						totalWeight += NdotL;
					}
				}

				if (totalWeight > 0.0) {
					prefilteredColor = prefilteredColor / totalWeight;
				}

				gl_FragColor = vec4(prefilteredColor, 1.0);
			}
		`,blending:Fn,depthTest:!1,depthWrite:!1})}function $m(i,t,e){let n=new Float32Array($i),s=new k(0,1,0);return new je({name:"SphericalGaussianBlur",defines:{n:$i,CUBEUV_TEXEL_WIDTH:1/t,CUBEUV_TEXEL_HEIGHT:1/e,CUBEUV_MAX_MIP:`${i}.0`},uniforms:{envMap:{value:null},samples:{value:1},weights:{value:n},latitudinal:{value:!1},dTheta:{value:0},mipInt:{value:0},poleAxis:{value:s}},vertexShader:Fo(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;
			uniform int samples;
			uniform float weights[ n ];
			uniform bool latitudinal;
			uniform float dTheta;
			uniform float mipInt;
			uniform vec3 poleAxis;

			#define ENVMAP_TYPE_CUBE_UV
			#include <cube_uv_reflection_fragment>

			vec3 getSample( float theta, vec3 axis ) {

				float cosTheta = cos( theta );
				// Rodrigues' axis-angle rotation
				vec3 sampleDirection = vOutputDirection * cosTheta
					+ cross( axis, vOutputDirection ) * sin( theta )
					+ axis * dot( axis, vOutputDirection ) * ( 1.0 - cosTheta );

				return bilinearCubeUV( envMap, sampleDirection, mipInt );

			}

			void main() {

				vec3 axis = latitudinal ? poleAxis : cross( poleAxis, vOutputDirection );

				if ( all( equal( axis, vec3( 0.0 ) ) ) ) {

					axis = vec3( vOutputDirection.z, 0.0, - vOutputDirection.x );

				}

				axis = normalize( axis );

				gl_FragColor = vec4( 0.0, 0.0, 0.0, 1.0 );
				gl_FragColor.rgb += weights[ 0 ] * getSample( 0.0, axis );

				for ( int i = 1; i < n; i++ ) {

					if ( i >= samples ) {

						break;

					}

					float theta = dTheta * float( i );
					gl_FragColor.rgb += weights[ i ] * getSample( -1.0 * theta, axis );
					gl_FragColor.rgb += weights[ i ] * getSample( theta, axis );

				}

			}
		`,blending:Fn,depthTest:!1,depthWrite:!1})}function xd(){return new je({name:"EquirectangularToCubeUV",uniforms:{envMap:{value:null}},vertexShader:Fo(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;

			#include <common>

			void main() {

				vec3 outputDirection = normalize( vOutputDirection );
				vec2 uv = equirectUv( outputDirection );

				gl_FragColor = vec4( texture2D ( envMap, uv ).rgb, 1.0 );

			}
		`,blending:Fn,depthTest:!1,depthWrite:!1})}function yd(){return new je({name:"CubemapToCubeUV",uniforms:{envMap:{value:null},flipEnvMap:{value:-1}},vertexShader:Fo(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			uniform float flipEnvMap;

			varying vec3 vOutputDirection;

			uniform samplerCube envMap;

			void main() {

				gl_FragColor = textureCube( envMap, vec3( flipEnvMap * vOutputDirection.x, vOutputDirection.yz ) );

			}
		`,blending:Fn,depthTest:!1,depthWrite:!1})}function Fo(){return`

		precision mediump float;
		precision mediump int;

		attribute float faceIndex;

		varying vec3 vOutputDirection;

		// RH coordinate system; PMREM face-indexing convention
		vec3 getDirection( vec2 uv, float face ) {

			uv = 2.0 * uv - 1.0;

			vec3 direction = vec3( uv, 1.0 );

			if ( face == 0.0 ) {

				direction = direction.zyx; // ( 1, v, u ) pos x

			} else if ( face == 1.0 ) {

				direction = direction.xzy;
				direction.xz *= -1.0; // ( -u, 1, -v ) pos y

			} else if ( face == 2.0 ) {

				direction.x *= -1.0; // ( -u, v, 1 ) pos z

			} else if ( face == 3.0 ) {

				direction = direction.zyx;
				direction.xz *= -1.0; // ( -1, v, -u ) neg x

			} else if ( face == 4.0 ) {

				direction = direction.xzy;
				direction.xy *= -1.0; // ( -u, -1, v ) neg y

			} else if ( face == 5.0 ) {

				direction.z *= -1.0; // ( u, v, -1 ) neg z

			}

			return direction;

		}

		void main() {

			vOutputDirection = getDirection( uv, faceIndex );
			gl_Position = vec4( position, 1.0 );

		}
	`}var No=class extends Ke{constructor(t=1,e={}){super(t,t,e),this.isWebGLCubeRenderTarget=!0;let n={width:t,height:t,depth:1},s=[n,n,n,n,n,n];this.texture=new Qs(s),this._setTextureOptions(e),this.texture.isRenderTargetTexture=!0}fromEquirectangularTexture(t,e){this.texture.type=e.type,this.texture.colorSpace=e.colorSpace,this.texture.generateMipmaps=e.generateMipmaps,this.texture.minFilter=e.minFilter,this.texture.magFilter=e.magFilter;let n={uniforms:{tEquirect:{value:null}},vertexShader:`

				varying vec3 vWorldDirection;

				vec3 transformDirection( in vec3 dir, in mat4 matrix ) {

					return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );

				}

				void main() {

					vWorldDirection = transformDirection( position, modelMatrix );

					#include <begin_vertex>
					#include <project_vertex>

				}
			`,fragmentShader:`

				uniform sampler2D tEquirect;

				varying vec3 vWorldDirection;

				#include <common>

				void main() {

					vec3 direction = normalize( vWorldDirection );

					vec2 sampleUV = equirectUv( direction );

					gl_FragColor = texture2D( tEquirect, sampleUV );

				}
			`},s=new pi(5,5,5),r=new je({name:"CubemapFromEquirect",uniforms:Wi(n.uniforms),vertexShader:n.vertexShader,fragmentShader:n.fragmentShader,side:ze,blending:Fn});r.uniforms.tEquirect.value=e;let a=new ke(s,r),o=e.minFilter;return e.minFilter===xi&&(e.minFilter=Pe),new za(1,10,this).update(t,a),e.minFilter=o,a.geometry.dispose(),a.material.dispose(),this}clear(t,e=!0,n=!0,s=!0){let r=t.getRenderTarget();for(let a=0;a<6;a++)t.setRenderTarget(this,a),t.clear(e,n,s);t.setRenderTarget(r)}};function Xm(i){let t=new WeakMap,e=new WeakMap,n=null;function s(h,p=!1){return h==null?null:p?a(h):r(h)}function r(h){if(h&&h.isTexture){let p=h.mapping;if(p===Ga||p===Ha)if(t.has(h)){let _=t.get(h).texture;return o(_,h.mapping)}else{let _=h.image;if(_&&_.height>0){let y=new No(_.height);return y.fromEquirectangularTexture(i,h),t.set(h,y),h.addEventListener("dispose",c),o(y.texture,h.mapping)}else return null}}return h}function a(h){if(h&&h.isTexture){let p=h.mapping,_=p===Ga||p===Ha,y=p===vi||p===Hi;if(_||y){let m=e.get(h),f=m!==void 0?m.texture.pmremVersion:0;if(h.isRenderTargetTexture&&h.pmremVersion!==f)return n===null&&(n=new Do(i)),m=_?n.fromEquirectangular(h,m):n.fromCubemap(h,m),m.texture.pmremVersion=h.pmremVersion,e.set(h,m),m.texture;if(m!==void 0)return m.texture;{let S=h.image;return _&&S&&S.height>0||y&&S&&l(S)?(n===null&&(n=new Do(i)),m=_?n.fromEquirectangular(h):n.fromCubemap(h),m.texture.pmremVersion=h.pmremVersion,e.set(h,m),h.addEventListener("dispose",d),m.texture):null}}}return h}function o(h,p){return p===Ga?h.mapping=vi:p===Ha&&(h.mapping=Hi),h}function l(h){let p=0,_=6;for(let y=0;y<_;y++)h[y]!==void 0&&p++;return p===_}function c(h){let p=h.target;p.removeEventListener("dispose",c);let _=t.get(p);_!==void 0&&(t.delete(p),_.dispose())}function d(h){let p=h.target;p.removeEventListener("dispose",d);let _=e.get(p);_!==void 0&&(e.delete(p),_.dispose())}function u(){t=new WeakMap,e=new WeakMap,n!==null&&(n.dispose(),n=null)}return{get:s,dispose:u}}function qm(i){let t={};function e(n){if(t[n]!==void 0)return t[n];let s=i.getExtension(n);return t[n]=s,s}return{has:function(n){return e(n)!==null},init:function(){e("EXT_color_buffer_float"),e("WEBGL_clip_cull_distance"),e("OES_texture_float_linear"),e("EXT_color_buffer_half_float"),e("WEBGL_multisampled_render_to_texture"),e("WEBGL_render_shared_exponent")},get:function(n){let s=e(n);return s===null&&ga("WebGLRenderer: "+n+" extension not supported."),s}}}function Ym(i,t,e,n){let s={},r=new WeakMap;function a(u){let h=u.target;h.index!==null&&t.remove(h.index);for(let _ in h.attributes)t.remove(h.attributes[_]);h.removeEventListener("dispose",a),delete s[h.id];let p=r.get(h);p&&(t.remove(p),r.delete(h)),n.releaseStatesOfGeometry(h),h.isInstancedBufferGeometry===!0&&delete h._maxInstanceCount,e.memory.geometries--}function o(u,h){return s[h.id]===!0||(h.addEventListener("dispose",a),s[h.id]=!0,e.memory.geometries++),h}function l(u){let h=u.attributes;for(let p in h)t.update(h[p],i.ARRAY_BUFFER)}function c(u){let h=[],p=u.index,_=u.attributes.position,y=0;if(_===void 0)return;if(p!==null){let S=p.array;y=p.version;for(let M=0,E=S.length;M<E;M+=3){let R=S[M+0],w=S[M+1],I=S[M+2];h.push(R,w,w,I,I,R)}}else{let S=_.array;y=_.version;for(let M=0,E=S.length/3-1;M<E;M+=3){let R=M+0,w=M+1,I=M+2;h.push(R,w,w,I,I,R)}}let m=new(_.count>=65535?Zs:Ys)(h,1);m.version=y;let f=r.get(u);f&&t.remove(f),r.set(u,m)}function d(u){let h=r.get(u);if(h){let p=u.index;p!==null&&h.version<p.version&&c(u)}else c(u);return r.get(u)}return{get:o,update:l,getWireframeAttribute:d}}function Zm(i,t,e){let n;function s(u){n=u}let r,a;function o(u){r=u.type,a=u.bytesPerElement}function l(u,h){i.drawElements(n,h,r,u*a),e.update(h,n,1)}function c(u,h,p){p!==0&&(i.drawElementsInstanced(n,h,r,u*a,p),e.update(h,n,p))}function d(u,h,p){if(p===0)return;t.get("WEBGL_multi_draw").multiDrawElementsWEBGL(n,h,0,r,u,0,p);let y=0;for(let m=0;m<p;m++)y+=h[m];e.update(y,n,1)}this.setMode=s,this.setIndex=o,this.render=l,this.renderInstances=c,this.renderMultiDraw=d}function Jm(i){let t={geometries:0,textures:0},e={frame:0,calls:0,triangles:0,points:0,lines:0};function n(r,a,o){switch(e.calls++,a){case i.TRIANGLES:e.triangles+=o*(r/3);break;case i.LINES:e.lines+=o*(r/2);break;case i.LINE_STRIP:e.lines+=o*(r-1);break;case i.LINE_LOOP:e.lines+=o*r;break;case i.POINTS:e.points+=o*r;break;default:Ct("WebGLInfo: Unknown draw mode:",a);break}}function s(){e.calls=0,e.triangles=0,e.points=0,e.lines=0}return{memory:t,render:e,programs:null,autoReset:!0,reset:s,update:n}}function Km(i,t,e){let n=new WeakMap,s=new pe;function r(a,o,l){let c=a.morphTargetInfluences,d=o.morphAttributes.position||o.morphAttributes.normal||o.morphAttributes.color,u=d!==void 0?d.length:0,h=n.get(o);if(h===void 0||h.count!==u){let A=function(){I.dispose(),n.delete(o),o.removeEventListener("dispose",A)};h!==void 0&&h.texture.dispose();let p=o.morphAttributes.position!==void 0,_=o.morphAttributes.normal!==void 0,y=o.morphAttributes.color!==void 0,m=o.morphAttributes.position||[],f=o.morphAttributes.normal||[],S=o.morphAttributes.color||[],M=0;p===!0&&(M=1),_===!0&&(M=2),y===!0&&(M=3);let E=o.attributes.position.count*M,R=1;E>t.maxTextureSize&&(R=Math.ceil(E/t.maxTextureSize),E=t.maxTextureSize);let w=new Float32Array(E*R*4*u),I=new $s(w,E,R,u);I.type=bn,I.needsUpdate=!0;let v=M*4;for(let D=0;D<u;D++){let T=m[D],O=f[D],H=S[D],$=E*R*4*D;for(let U=0;U<T.count;U++){let L=U*v;p===!0&&(s.fromBufferAttribute(T,U),w[$+L+0]=s.x,w[$+L+1]=s.y,w[$+L+2]=s.z,w[$+L+3]=0),_===!0&&(s.fromBufferAttribute(O,U),w[$+L+4]=s.x,w[$+L+5]=s.y,w[$+L+6]=s.z,w[$+L+7]=0),y===!0&&(s.fromBufferAttribute(H,U),w[$+L+8]=s.x,w[$+L+9]=s.y,w[$+L+10]=s.z,w[$+L+11]=H.itemSize===4?s.w:1)}}h={count:u,texture:I,size:new jt(E,R)},n.set(o,h),o.addEventListener("dispose",A)}if(a.isInstancedMesh===!0&&a.morphTexture!==null)l.getUniforms().setValue(i,"morphTexture",a.morphTexture,e);else{let p=0;for(let y=0;y<c.length;y++)p+=c[y];let _=o.morphTargetsRelative?1:1-p;l.getUniforms().setValue(i,"morphTargetBaseInfluence",_),l.getUniforms().setValue(i,"morphTargetInfluences",c)}l.getUniforms().setValue(i,"morphTargetsTexture",h.texture,e),l.getUniforms().setValue(i,"morphTargetsTextureSize",h.size)}return{update:r}}function jm(i,t,e,n,s){let r=new WeakMap;function a(c){let d=s.render.frame,u=c.geometry,h=t.get(c,u);if(r.get(h)!==d&&(t.update(h),r.set(h,d)),c.isInstancedMesh&&(c.hasEventListener("dispose",l)===!1&&c.addEventListener("dispose",l),r.get(c)!==d&&(e.update(c.instanceMatrix,i.ARRAY_BUFFER),c.instanceColor!==null&&e.update(c.instanceColor,i.ARRAY_BUFFER),r.set(c,d))),c.isSkinnedMesh){let p=c.skeleton;r.get(p)!==d&&(p.update(),r.set(p,d))}return h}function o(){r=new WeakMap}function l(c){let d=c.target;d.removeEventListener("dispose",l),n.releaseStatesOfObject(d),e.remove(d.instanceMatrix),d.instanceColor!==null&&e.remove(d.instanceColor)}return{update:a,dispose:o}}var Qm={[Yl]:"LINEAR_TONE_MAPPING",[Zl]:"REINHARD_TONE_MAPPING",[Jl]:"CINEON_TONE_MAPPING",[Kl]:"ACES_FILMIC_TONE_MAPPING",[Ql]:"AGX_TONE_MAPPING",[tc]:"NEUTRAL_TONE_MAPPING",[jl]:"CUSTOM_TONE_MAPPING"};function tg(i,t,e,n,s){let r=new Ke(t,e,{type:i,depthBuffer:n,stencilBuffer:s,depthTexture:n?new jn(t,e):void 0}),a=new Ke(t,e,{type:On,depthBuffer:!1,stencilBuffer:!1}),o=new cn;o.setAttribute("position",new Fe([-1,3,0,-1,-1,0,3,-1,0],3)),o.setAttribute("uv",new Fe([0,2,0,0,2,0],2));let l=new Ta({uniforms:{tDiffuse:{value:null}},vertexShader:`
			precision highp float;

			uniform mat4 modelViewMatrix;
			uniform mat4 projectionMatrix;

			attribute vec3 position;
			attribute vec2 uv;

			varying vec2 vUv;

			void main() {
				vUv = uv;
				gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
			}`,fragmentShader:`
			precision highp float;

			uniform sampler2D tDiffuse;

			varying vec2 vUv;

			#include <tonemapping_pars_fragment>
			#include <colorspace_pars_fragment>

			void main() {
				gl_FragColor = texture2D( tDiffuse, vUv );

				#ifdef LINEAR_TONE_MAPPING
					gl_FragColor.rgb = LinearToneMapping( gl_FragColor.rgb );
				#elif defined( REINHARD_TONE_MAPPING )
					gl_FragColor.rgb = ReinhardToneMapping( gl_FragColor.rgb );
				#elif defined( CINEON_TONE_MAPPING )
					gl_FragColor.rgb = CineonToneMapping( gl_FragColor.rgb );
				#elif defined( ACES_FILMIC_TONE_MAPPING )
					gl_FragColor.rgb = ACESFilmicToneMapping( gl_FragColor.rgb );
				#elif defined( AGX_TONE_MAPPING )
					gl_FragColor.rgb = AgXToneMapping( gl_FragColor.rgb );
				#elif defined( NEUTRAL_TONE_MAPPING )
					gl_FragColor.rgb = NeutralToneMapping( gl_FragColor.rgb );
				#elif defined( CUSTOM_TONE_MAPPING )
					gl_FragColor.rgb = CustomToneMapping( gl_FragColor.rgb );
				#endif

				#ifdef SRGB_TRANSFER
					gl_FragColor = sRGBTransferOETF( gl_FragColor );
				#endif
			}`,depthTest:!1,depthWrite:!1}),c=new ke(o,l),d=new _s(-1,1,1,-1,0,1),u=null,h=null,p=!1,_,y=null,m=[],f=!1;this.setSize=function(S,M){r.setSize(S,M),a.setSize(S,M);for(let E=0;E<m.length;E++){let R=m[E];R.setSize&&R.setSize(S,M)}},this.setEffects=function(S){m=S,f=m.length>0&&m[0].isRenderPass===!0;let M=r.width,E=r.height;for(let R=0;R<m.length;R++){let w=m[R];w.setSize&&w.setSize(M,E)}},this.begin=function(S,M){if(p||S.toneMapping===xn&&m.length===0)return!1;if(y=M,M!==null){let E=M.width,R=M.height;(r.width!==E||r.height!==R)&&this.setSize(E,R)}return f===!1&&S.setRenderTarget(r),_=S.toneMapping,S.toneMapping=xn,!0},this.hasRenderPass=function(){return f},this.end=function(S,M){S.toneMapping=_,p=!0;let E=r,R=a;for(let w=0;w<m.length;w++){let I=m[w];if(I.enabled!==!1&&(I.render(S,R,E,M),I.needsSwap!==!1)){let v=E;E=R,R=v}}if(u!==S.outputColorSpace||h!==S.toneMapping){u=S.outputColorSpace,h=S.toneMapping,l.defines={},$t.getTransfer(u)===te&&(l.defines.SRGB_TRANSFER="");let w=Qm[h];w&&(l.defines[w]=""),l.needsUpdate=!0}l.uniforms.tDiffuse.value=E.texture,S.setRenderTarget(y),S.render(c,d),y=null,p=!1},this.isCompositing=function(){return p},this.dispose=function(){r.depthTexture&&r.depthTexture.dispose(),r.dispose(),a.dispose(),o.dispose(),l.dispose()}}var zd=new Ge,bc=new jn(1,1),Vd=new $s,Gd=new ya,Hd=new Qs,bd=[],Sd=[],Md=new Float32Array(16),wd=new Float32Array(9),Ad=new Float32Array(4);function Ms(i,t,e){let n=i[0];if(n<=0||n>0)return i;let s=t*e,r=bd[s];if(r===void 0&&(r=new Float32Array(s),bd[s]=r),t!==0){n.toArray(r,0);for(let a=1,o=0;a!==t;++a)o+=e,i[a].toArray(r,o)}return r}function Ae(i,t){if(i.length!==t.length)return!1;for(let e=0,n=i.length;e<n;e++)if(i[e]!==t[e])return!1;return!0}function Ee(i,t){for(let e=0,n=t.length;e<n;e++)i[e]=t[e]}function Oo(i,t){let e=Sd[t];e===void 0&&(e=new Int32Array(t),Sd[t]=e);for(let n=0;n!==t;++n)e[n]=i.allocateTextureUnit();return e}function eg(i,t){let e=this.cache;e[0]!==t&&(i.uniform1f(this.addr,t),e[0]=t)}function ng(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2f(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ae(e,t))return;i.uniform2fv(this.addr,t),Ee(e,t)}}function ig(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3f(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else if(t.r!==void 0)(e[0]!==t.r||e[1]!==t.g||e[2]!==t.b)&&(i.uniform3f(this.addr,t.r,t.g,t.b),e[0]=t.r,e[1]=t.g,e[2]=t.b);else{if(Ae(e,t))return;i.uniform3fv(this.addr,t),Ee(e,t)}}function sg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4f(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ae(e,t))return;i.uniform4fv(this.addr,t),Ee(e,t)}}function rg(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ae(e,t))return;i.uniformMatrix2fv(this.addr,!1,t),Ee(e,t)}else{if(Ae(e,n))return;Ad.set(n),i.uniformMatrix2fv(this.addr,!1,Ad),Ee(e,n)}}function ag(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ae(e,t))return;i.uniformMatrix3fv(this.addr,!1,t),Ee(e,t)}else{if(Ae(e,n))return;wd.set(n),i.uniformMatrix3fv(this.addr,!1,wd),Ee(e,n)}}function og(i,t){let e=this.cache,n=t.elements;if(n===void 0){if(Ae(e,t))return;i.uniformMatrix4fv(this.addr,!1,t),Ee(e,t)}else{if(Ae(e,n))return;Md.set(n),i.uniformMatrix4fv(this.addr,!1,Md),Ee(e,n)}}function lg(i,t){let e=this.cache;e[0]!==t&&(i.uniform1i(this.addr,t),e[0]=t)}function cg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2i(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ae(e,t))return;i.uniform2iv(this.addr,t),Ee(e,t)}}function hg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3i(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Ae(e,t))return;i.uniform3iv(this.addr,t),Ee(e,t)}}function dg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4i(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ae(e,t))return;i.uniform4iv(this.addr,t),Ee(e,t)}}function ug(i,t){let e=this.cache;e[0]!==t&&(i.uniform1ui(this.addr,t),e[0]=t)}function fg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y)&&(i.uniform2ui(this.addr,t.x,t.y),e[0]=t.x,e[1]=t.y);else{if(Ae(e,t))return;i.uniform2uiv(this.addr,t),Ee(e,t)}}function pg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z)&&(i.uniform3ui(this.addr,t.x,t.y,t.z),e[0]=t.x,e[1]=t.y,e[2]=t.z);else{if(Ae(e,t))return;i.uniform3uiv(this.addr,t),Ee(e,t)}}function mg(i,t){let e=this.cache;if(t.x!==void 0)(e[0]!==t.x||e[1]!==t.y||e[2]!==t.z||e[3]!==t.w)&&(i.uniform4ui(this.addr,t.x,t.y,t.z,t.w),e[0]=t.x,e[1]=t.y,e[2]=t.z,e[3]=t.w);else{if(Ae(e,t))return;i.uniform4uiv(this.addr,t),Ee(e,t)}}function gg(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s);let r;this.type===i.SAMPLER_2D_SHADOW?(bc.compareFunction=e.isReversedDepthBuffer()?Io:Ro,r=bc):r=zd,e.setTexture2D(t||r,s)}function _g(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTexture3D(t||Gd,s)}function vg(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTextureCube(t||Hd,s)}function xg(i,t,e){let n=this.cache,s=e.allocateTextureUnit();n[0]!==s&&(i.uniform1i(this.addr,s),n[0]=s),e.setTexture2DArray(t||Vd,s)}function yg(i){switch(i){case 5126:return eg;case 35664:return ng;case 35665:return ig;case 35666:return sg;case 35674:return rg;case 35675:return ag;case 35676:return og;case 5124:case 35670:return lg;case 35667:case 35671:return cg;case 35668:case 35672:return hg;case 35669:case 35673:return dg;case 5125:return ug;case 36294:return fg;case 36295:return pg;case 36296:return mg;case 35678:case 36198:case 36298:case 36306:case 35682:return gg;case 35679:case 36299:case 36307:return _g;case 35680:case 36300:case 36308:case 36293:return vg;case 36289:case 36303:case 36311:case 36292:return xg}}function bg(i,t){i.uniform1fv(this.addr,t)}function Sg(i,t){let e=Ms(t,this.size,2);i.uniform2fv(this.addr,e)}function Mg(i,t){let e=Ms(t,this.size,3);i.uniform3fv(this.addr,e)}function wg(i,t){let e=Ms(t,this.size,4);i.uniform4fv(this.addr,e)}function Ag(i,t){let e=Ms(t,this.size,4);i.uniformMatrix2fv(this.addr,!1,e)}function Eg(i,t){let e=Ms(t,this.size,9);i.uniformMatrix3fv(this.addr,!1,e)}function Tg(i,t){let e=Ms(t,this.size,16);i.uniformMatrix4fv(this.addr,!1,e)}function Cg(i,t){i.uniform1iv(this.addr,t)}function Rg(i,t){i.uniform2iv(this.addr,t)}function Ig(i,t){i.uniform3iv(this.addr,t)}function Pg(i,t){i.uniform4iv(this.addr,t)}function Lg(i,t){i.uniform1uiv(this.addr,t)}function Dg(i,t){i.uniform2uiv(this.addr,t)}function Ng(i,t){i.uniform3uiv(this.addr,t)}function Ug(i,t){i.uniform4uiv(this.addr,t)}function Fg(i,t,e){let n=this.cache,s=t.length,r=Oo(e,s);Ae(n,r)||(i.uniform1iv(this.addr,r),Ee(n,r));let a;this.type===i.SAMPLER_2D_SHADOW?a=bc:a=zd;for(let o=0;o!==s;++o)e.setTexture2D(t[o]||a,r[o])}function Og(i,t,e){let n=this.cache,s=t.length,r=Oo(e,s);Ae(n,r)||(i.uniform1iv(this.addr,r),Ee(n,r));for(let a=0;a!==s;++a)e.setTexture3D(t[a]||Gd,r[a])}function Bg(i,t,e){let n=this.cache,s=t.length,r=Oo(e,s);Ae(n,r)||(i.uniform1iv(this.addr,r),Ee(n,r));for(let a=0;a!==s;++a)e.setTextureCube(t[a]||Hd,r[a])}function kg(i,t,e){let n=this.cache,s=t.length,r=Oo(e,s);Ae(n,r)||(i.uniform1iv(this.addr,r),Ee(n,r));for(let a=0;a!==s;++a)e.setTexture2DArray(t[a]||Vd,r[a])}function zg(i){switch(i){case 5126:return bg;case 35664:return Sg;case 35665:return Mg;case 35666:return wg;case 35674:return Ag;case 35675:return Eg;case 35676:return Tg;case 5124:case 35670:return Cg;case 35667:case 35671:return Rg;case 35668:case 35672:return Ig;case 35669:case 35673:return Pg;case 5125:return Lg;case 36294:return Dg;case 36295:return Ng;case 36296:return Ug;case 35678:case 36198:case 36298:case 36306:case 35682:return Fg;case 35679:case 36299:case 36307:return Og;case 35680:case 36300:case 36308:case 36293:return Bg;case 36289:case 36303:case 36311:case 36292:return kg}}var Sc=class{constructor(t,e,n){this.id=t,this.addr=n,this.cache=[],this.type=e.type,this.setValue=yg(e.type)}},Mc=class{constructor(t,e,n){this.id=t,this.addr=n,this.cache=[],this.type=e.type,this.size=e.size,this.setValue=zg(e.type)}},wc=class{constructor(t){this.id=t,this.seq=[],this.map={}}setValue(t,e,n){let s=this.seq;for(let r=0,a=s.length;r!==a;++r){let o=s[r];o.setValue(t,e[o.id],n)}}},xc=/(\w+)(\])?(\[|\.)?/g;function Ed(i,t){i.seq.push(t),i.map[t.id]=t}function Vg(i,t,e){let n=i.name,s=n.length;for(xc.lastIndex=0;;){let r=xc.exec(n),a=xc.lastIndex,o=r[1],l=r[2]==="]",c=r[3];if(l&&(o=o|0),c===void 0||c==="["&&a+2===s){Ed(e,c===void 0?new Sc(o,i,t):new Mc(o,i,t));break}else{let u=e.map[o];u===void 0&&(u=new wc(o),Ed(e,u)),e=u}}}var Ss=class{constructor(t,e){this.seq=[],this.map={};let n=t.getProgramParameter(e,t.ACTIVE_UNIFORMS);for(let a=0;a<n;++a){let o=t.getActiveUniform(e,a),l=t.getUniformLocation(e,o.name);Vg(o,l,this)}let s=[],r=[];for(let a of this.seq)a.type===t.SAMPLER_2D_SHADOW||a.type===t.SAMPLER_CUBE_SHADOW||a.type===t.SAMPLER_2D_ARRAY_SHADOW?s.push(a):r.push(a);s.length>0&&(this.seq=s.concat(r))}setValue(t,e,n,s){let r=this.map[e];r!==void 0&&r.setValue(t,n,s)}setOptional(t,e,n){let s=e[n];s!==void 0&&this.setValue(t,n,s)}static upload(t,e,n,s){for(let r=0,a=e.length;r!==a;++r){let o=e[r],l=n[o.id];l.needsUpdate!==!1&&o.setValue(t,l.value,s)}}static seqWithValue(t,e){let n=[];for(let s=0,r=t.length;s!==r;++s){let a=t[s];a.id in e&&n.push(a)}return n}};function Td(i,t,e){let n=i.createShader(t);return i.shaderSource(n,e),i.compileShader(n),n}var Gg=37297,Hg=0;function Wg(i,t){let e=i.split(`
`),n=[],s=Math.max(t-6,0),r=Math.min(t+6,e.length);for(let a=s;a<r;a++){let o=a+1;n.push(`${o===t?">":" "} ${o}: ${e[a]}`)}return n.join(`
`)}var Cd=new Pt;function $g(i){$t._getMatrix(Cd,$t.workingColorSpace,i);let t=`mat3( ${Cd.elements.map(e=>e.toFixed(4))} )`;switch($t.getTransfer(i)){case Hs:return[t,"LinearTransferOETF"];case te:return[t,"sRGBTransferOETF"];default:return Et("WebGLProgram: Unsupported color space: ",i),[t,"LinearTransferOETF"]}}function Rd(i,t,e){let n=i.getShaderParameter(t,i.COMPILE_STATUS),r=(i.getShaderInfoLog(t)||"").trim();if(n&&r==="")return"";let a=/ERROR: 0:(\d+)/.exec(r);if(a){let o=parseInt(a[1]);return e.toUpperCase()+`

`+r+`

`+Wg(i.getShaderSource(t),o)}else return r}function Xg(i,t){let e=$g(t);return[`vec4 ${i}( vec4 value ) {`,`	return ${e[1]}( vec4( value.rgb * ${e[0]}, value.a ) );`,"}"].join(`
`)}var qg={[Yl]:"Linear",[Zl]:"Reinhard",[Jl]:"Cineon",[Kl]:"ACESFilmic",[Ql]:"AgX",[tc]:"Neutral",[jl]:"Custom"};function Yg(i,t){let e=qg[t];return e===void 0?(Et("WebGLProgram: Unsupported toneMapping:",t),"vec3 "+i+"( vec3 color ) { return LinearToneMapping( color ); }"):"vec3 "+i+"( vec3 color ) { return "+e+"ToneMapping( color ); }"}var Lo=new k;function Zg(){$t.getLuminanceCoefficients(Lo);let i=Lo.x.toFixed(4),t=Lo.y.toFixed(4),e=Lo.z.toFixed(4);return["float luminance( const in vec3 rgb ) {",`	const vec3 weights = vec3( ${i}, ${t}, ${e} );`,"	return dot( weights, rgb );","}"].join(`
`)}function Jg(i){return[i.extensionClipCullDistance?"#extension GL_ANGLE_clip_cull_distance : require":"",i.extensionMultiDraw?"#extension GL_ANGLE_multi_draw : require":""].filter(xr).join(`
`)}function Kg(i){let t=[];for(let e in i){let n=i[e];n!==!1&&t.push("#define "+e+" "+n)}return t.join(`
`)}function jg(i,t){let e={},n=i.getProgramParameter(t,i.ACTIVE_ATTRIBUTES);for(let s=0;s<n;s++){let r=i.getActiveAttrib(t,s),a=r.name,o=1;r.type===i.FLOAT_MAT2&&(o=2),r.type===i.FLOAT_MAT3&&(o=3),r.type===i.FLOAT_MAT4&&(o=4),e[a]={type:r.type,location:i.getAttribLocation(t,a),locationSize:o}}return e}function xr(i){return i!==""}function Id(i,t){let e=t.numSpotLightShadows+t.numSpotLightMaps-t.numSpotLightShadowsWithMaps;return i.replace(/NUM_DIR_LIGHTS/g,t.numDirLights).replace(/NUM_SPOT_LIGHTS/g,t.numSpotLights).replace(/NUM_SPOT_LIGHT_MAPS/g,t.numSpotLightMaps).replace(/NUM_SPOT_LIGHT_COORDS/g,e).replace(/NUM_RECT_AREA_LIGHTS/g,t.numRectAreaLights).replace(/NUM_POINT_LIGHTS/g,t.numPointLights).replace(/NUM_HEMI_LIGHTS/g,t.numHemiLights).replace(/NUM_DIR_LIGHT_SHADOWS/g,t.numDirLightShadows).replace(/NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS/g,t.numSpotLightShadowsWithMaps).replace(/NUM_SPOT_LIGHT_SHADOWS/g,t.numSpotLightShadows).replace(/NUM_POINT_LIGHT_SHADOWS/g,t.numPointLightShadows)}function Pd(i,t){return i.replace(/NUM_CLIPPING_PLANES/g,t.numClippingPlanes).replace(/UNION_CLIPPING_PLANES/g,t.numClippingPlanes-t.numClipIntersection)}var Qg=/^[ \t]*#include +<([\w\d./]+)>/gm;function Ac(i){return i.replace(Qg,e_)}var t_=new Map;function e_(i,t){let e=Ot[t];if(e===void 0){let n=t_.get(t);if(n!==void 0)e=Ot[n],Et('WebGLRenderer: Shader chunk "%s" has been deprecated. Use "%s" instead.',t,n);else throw new Error("Can not resolve #include <"+t+">")}return Ac(e)}var n_=/#pragma unroll_loop_start\s+for\s*\(\s*int\s+i\s*=\s*(\d+)\s*;\s*i\s*<\s*(\d+)\s*;\s*i\s*\+\+\s*\)\s*{([\s\S]+?)}\s+#pragma unroll_loop_end/g;function Ld(i){return i.replace(n_,i_)}function i_(i,t,e,n){let s="";for(let r=parseInt(t);r<parseInt(e);r++)s+=n.replace(/\[\s*i\s*\]/g,"[ "+r+" ]").replace(/UNROLLED_LOOP_INDEX/g,r);return s}function Dd(i){let t=`precision ${i.precision} float;
	precision ${i.precision} int;
	precision ${i.precision} sampler2D;
	precision ${i.precision} samplerCube;
	precision ${i.precision} sampler3D;
	precision ${i.precision} sampler2DArray;
	precision ${i.precision} sampler2DShadow;
	precision ${i.precision} samplerCubeShadow;
	precision ${i.precision} sampler2DArrayShadow;
	precision ${i.precision} isampler2D;
	precision ${i.precision} isampler3D;
	precision ${i.precision} isamplerCube;
	precision ${i.precision} isampler2DArray;
	precision ${i.precision} usampler2D;
	precision ${i.precision} usampler3D;
	precision ${i.precision} usamplerCube;
	precision ${i.precision} usampler2DArray;
	`;return i.precision==="highp"?t+=`
#define HIGH_PRECISION`:i.precision==="mediump"?t+=`
#define MEDIUM_PRECISION`:i.precision==="lowp"&&(t+=`
#define LOW_PRECISION`),t}var s_={[or]:"SHADOWMAP_TYPE_PCF",[vs]:"SHADOWMAP_TYPE_VSM"};function r_(i){return s_[i.shadowMapType]||"SHADOWMAP_TYPE_BASIC"}var a_={[vi]:"ENVMAP_TYPE_CUBE",[Hi]:"ENVMAP_TYPE_CUBE",[lr]:"ENVMAP_TYPE_CUBE_UV"};function o_(i){return i.envMap===!1?"ENVMAP_TYPE_CUBE":a_[i.envMapMode]||"ENVMAP_TYPE_CUBE"}var l_={[Hi]:"ENVMAP_MODE_REFRACTION"};function c_(i){return i.envMap===!1?"ENVMAP_MODE_REFLECTION":l_[i.envMapMode]||"ENVMAP_MODE_REFLECTION"}var h_={[ql]:"ENVMAP_BLENDING_MULTIPLY",[jh]:"ENVMAP_BLENDING_MIX",[Qh]:"ENVMAP_BLENDING_ADD"};function d_(i){return i.envMap===!1?"ENVMAP_BLENDING_NONE":h_[i.combine]||"ENVMAP_BLENDING_NONE"}function u_(i){let t=i.envMapCubeUVHeight;if(t===null)return null;let e=Math.log2(t)-2,n=1/t;return{texelWidth:1/(3*Math.max(Math.pow(2,e),112)),texelHeight:n,maxMip:e}}function f_(i,t,e,n){let s=i.getContext(),r=e.defines,a=e.vertexShader,o=e.fragmentShader,l=r_(e),c=o_(e),d=c_(e),u=d_(e),h=u_(e),p=Jg(e),_=Kg(r),y=s.createProgram(),m,f,S=e.glslVersion?"#version "+e.glslVersion+`
`:"";e.isRawShaderMaterial?(m=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,_].filter(xr).join(`
`),m.length>0&&(m+=`
`),f=["#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,_].filter(xr).join(`
`),f.length>0&&(f+=`
`)):(m=[Dd(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,_,e.extensionClipCullDistance?"#define USE_CLIP_DISTANCE":"",e.batching?"#define USE_BATCHING":"",e.batchingColor?"#define USE_BATCHING_COLOR":"",e.instancing?"#define USE_INSTANCING":"",e.instancingColor?"#define USE_INSTANCING_COLOR":"",e.instancingMorph?"#define USE_INSTANCING_MORPH":"",e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.map?"#define USE_MAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+d:"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.displacementMap?"#define USE_DISPLACEMENTMAP":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.mapUv?"#define MAP_UV "+e.mapUv:"",e.alphaMapUv?"#define ALPHAMAP_UV "+e.alphaMapUv:"",e.lightMapUv?"#define LIGHTMAP_UV "+e.lightMapUv:"",e.aoMapUv?"#define AOMAP_UV "+e.aoMapUv:"",e.emissiveMapUv?"#define EMISSIVEMAP_UV "+e.emissiveMapUv:"",e.bumpMapUv?"#define BUMPMAP_UV "+e.bumpMapUv:"",e.normalMapUv?"#define NORMALMAP_UV "+e.normalMapUv:"",e.displacementMapUv?"#define DISPLACEMENTMAP_UV "+e.displacementMapUv:"",e.metalnessMapUv?"#define METALNESSMAP_UV "+e.metalnessMapUv:"",e.roughnessMapUv?"#define ROUGHNESSMAP_UV "+e.roughnessMapUv:"",e.anisotropyMapUv?"#define ANISOTROPYMAP_UV "+e.anisotropyMapUv:"",e.clearcoatMapUv?"#define CLEARCOATMAP_UV "+e.clearcoatMapUv:"",e.clearcoatNormalMapUv?"#define CLEARCOAT_NORMALMAP_UV "+e.clearcoatNormalMapUv:"",e.clearcoatRoughnessMapUv?"#define CLEARCOAT_ROUGHNESSMAP_UV "+e.clearcoatRoughnessMapUv:"",e.iridescenceMapUv?"#define IRIDESCENCEMAP_UV "+e.iridescenceMapUv:"",e.iridescenceThicknessMapUv?"#define IRIDESCENCE_THICKNESSMAP_UV "+e.iridescenceThicknessMapUv:"",e.sheenColorMapUv?"#define SHEEN_COLORMAP_UV "+e.sheenColorMapUv:"",e.sheenRoughnessMapUv?"#define SHEEN_ROUGHNESSMAP_UV "+e.sheenRoughnessMapUv:"",e.specularMapUv?"#define SPECULARMAP_UV "+e.specularMapUv:"",e.specularColorMapUv?"#define SPECULAR_COLORMAP_UV "+e.specularColorMapUv:"",e.specularIntensityMapUv?"#define SPECULAR_INTENSITYMAP_UV "+e.specularIntensityMapUv:"",e.transmissionMapUv?"#define TRANSMISSIONMAP_UV "+e.transmissionMapUv:"",e.thicknessMapUv?"#define THICKNESSMAP_UV "+e.thicknessMapUv:"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexNormals?"#define HAS_NORMAL":"",e.vertexColors?"#define USE_COLOR":"",e.vertexAlphas?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.flatShading?"#define FLAT_SHADED":"",e.skinning?"#define USE_SKINNING":"",e.morphTargets?"#define USE_MORPHTARGETS":"",e.morphNormals&&e.flatShading===!1?"#define USE_MORPHNORMALS":"",e.morphColors?"#define USE_MORPHCOLORS":"",e.morphTargetsCount>0?"#define MORPHTARGETS_TEXTURE_STRIDE "+e.morphTextureStride:"",e.morphTargetsCount>0?"#define MORPHTARGETS_COUNT "+e.morphTargetsCount:"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+l:"",e.sizeAttenuation?"#define USE_SIZEATTENUATION":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.logarithmicDepthBuffer?"#define USE_LOGARITHMIC_DEPTH_BUFFER":"",e.reversedDepthBuffer?"#define USE_REVERSED_DEPTH_BUFFER":"","uniform mat4 modelMatrix;","uniform mat4 modelViewMatrix;","uniform mat4 projectionMatrix;","uniform mat4 viewMatrix;","uniform mat3 normalMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;","#ifdef USE_INSTANCING","	attribute mat4 instanceMatrix;","#endif","#ifdef USE_INSTANCING_COLOR","	attribute vec3 instanceColor;","#endif","#ifdef USE_INSTANCING_MORPH","	uniform sampler2D morphTexture;","#endif","attribute vec3 position;","attribute vec3 normal;","attribute vec2 uv;","#ifdef USE_UV1","	attribute vec2 uv1;","#endif","#ifdef USE_UV2","	attribute vec2 uv2;","#endif","#ifdef USE_UV3","	attribute vec2 uv3;","#endif","#ifdef USE_TANGENT","	attribute vec4 tangent;","#endif","#if defined( USE_COLOR_ALPHA )","	attribute vec4 color;","#elif defined( USE_COLOR )","	attribute vec3 color;","#endif","#ifdef USE_SKINNING","	attribute vec4 skinIndex;","	attribute vec4 skinWeight;","#endif",`
`].filter(xr).join(`
`),f=[Dd(e),"#define SHADER_TYPE "+e.shaderType,"#define SHADER_NAME "+e.shaderName,_,e.useFog&&e.fog?"#define USE_FOG":"",e.useFog&&e.fogExp2?"#define FOG_EXP2":"",e.alphaToCoverage?"#define ALPHA_TO_COVERAGE":"",e.map?"#define USE_MAP":"",e.matcap?"#define USE_MATCAP":"",e.envMap?"#define USE_ENVMAP":"",e.envMap?"#define "+c:"",e.envMap?"#define "+d:"",e.envMap?"#define "+u:"",h?"#define CUBEUV_TEXEL_WIDTH "+h.texelWidth:"",h?"#define CUBEUV_TEXEL_HEIGHT "+h.texelHeight:"",h?"#define CUBEUV_MAX_MIP "+h.maxMip+".0":"",e.lightMap?"#define USE_LIGHTMAP":"",e.aoMap?"#define USE_AOMAP":"",e.bumpMap?"#define USE_BUMPMAP":"",e.normalMap?"#define USE_NORMALMAP":"",e.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",e.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",e.packedNormalMap?"#define USE_PACKED_NORMALMAP":"",e.emissiveMap?"#define USE_EMISSIVEMAP":"",e.anisotropy?"#define USE_ANISOTROPY":"",e.anisotropyMap?"#define USE_ANISOTROPYMAP":"",e.clearcoat?"#define USE_CLEARCOAT":"",e.clearcoatMap?"#define USE_CLEARCOATMAP":"",e.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",e.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",e.dispersion?"#define USE_DISPERSION":"",e.iridescence?"#define USE_IRIDESCENCE":"",e.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",e.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",e.specularMap?"#define USE_SPECULARMAP":"",e.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",e.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",e.roughnessMap?"#define USE_ROUGHNESSMAP":"",e.metalnessMap?"#define USE_METALNESSMAP":"",e.alphaMap?"#define USE_ALPHAMAP":"",e.alphaTest?"#define USE_ALPHATEST":"",e.alphaHash?"#define USE_ALPHAHASH":"",e.sheen?"#define USE_SHEEN":"",e.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",e.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",e.transmission?"#define USE_TRANSMISSION":"",e.transmissionMap?"#define USE_TRANSMISSIONMAP":"",e.thicknessMap?"#define USE_THICKNESSMAP":"",e.vertexTangents&&e.flatShading===!1?"#define USE_TANGENT":"",e.vertexColors||e.instancingColor?"#define USE_COLOR":"",e.vertexAlphas||e.batchingColor?"#define USE_COLOR_ALPHA":"",e.vertexUv1s?"#define USE_UV1":"",e.vertexUv2s?"#define USE_UV2":"",e.vertexUv3s?"#define USE_UV3":"",e.pointsUvs?"#define USE_POINTS_UV":"",e.gradientMap?"#define USE_GRADIENTMAP":"",e.flatShading?"#define FLAT_SHADED":"",e.doubleSided?"#define DOUBLE_SIDED":"",e.flipSided?"#define FLIP_SIDED":"",e.shadowMapEnabled?"#define USE_SHADOWMAP":"",e.shadowMapEnabled?"#define "+l:"",e.premultipliedAlpha?"#define PREMULTIPLIED_ALPHA":"",e.numLightProbes>0?"#define USE_LIGHT_PROBES":"",e.numLightProbeGrids>0?"#define USE_LIGHT_PROBES_GRID":"",e.decodeVideoTexture?"#define DECODE_VIDEO_TEXTURE":"",e.decodeVideoTextureEmissive?"#define DECODE_VIDEO_TEXTURE_EMISSIVE":"",e.logarithmicDepthBuffer?"#define USE_LOGARITHMIC_DEPTH_BUFFER":"",e.reversedDepthBuffer?"#define USE_REVERSED_DEPTH_BUFFER":"","uniform mat4 viewMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;",e.toneMapping!==xn?"#define TONE_MAPPING":"",e.toneMapping!==xn?Ot.tonemapping_pars_fragment:"",e.toneMapping!==xn?Yg("toneMapping",e.toneMapping):"",e.dithering?"#define DITHERING":"",e.opaque?"#define OPAQUE":"",Ot.colorspace_pars_fragment,Xg("linearToOutputTexel",e.outputColorSpace),Zg(),e.useDepthPacking?"#define DEPTH_PACKING "+e.depthPacking:"",`
`].filter(xr).join(`
`)),a=Ac(a),a=Id(a,e),a=Pd(a,e),o=Ac(o),o=Id(o,e),o=Pd(o,e),a=Ld(a),o=Ld(o),e.isRawShaderMaterial!==!0&&(S=`#version 300 es
`,m=[p,"#define attribute in","#define varying out","#define texture2D texture"].join(`
`)+`
`+m,f=["#define varying in",e.glslVersion===cc?"":"layout(location = 0) out highp vec4 pc_fragColor;",e.glslVersion===cc?"":"#define gl_FragColor pc_fragColor","#define gl_FragDepthEXT gl_FragDepth","#define texture2D texture","#define textureCube texture","#define texture2DProj textureProj","#define texture2DLodEXT textureLod","#define texture2DProjLodEXT textureProjLod","#define textureCubeLodEXT textureLod","#define texture2DGradEXT textureGrad","#define texture2DProjGradEXT textureProjGrad","#define textureCubeGradEXT textureGrad"].join(`
`)+`
`+f);let M=S+m+a,E=S+f+o,R=Td(s,s.VERTEX_SHADER,M),w=Td(s,s.FRAGMENT_SHADER,E);s.attachShader(y,R),s.attachShader(y,w),e.index0AttributeName!==void 0?s.bindAttribLocation(y,0,e.index0AttributeName):e.morphTargets===!0&&s.bindAttribLocation(y,0,"position"),s.linkProgram(y);function I(T){if(i.debug.checkShaderErrors){let O=s.getProgramInfoLog(y)||"",H=s.getShaderInfoLog(R)||"",$=s.getShaderInfoLog(w)||"",U=O.trim(),L=H.trim(),B=$.trim(),K=!0,tt=!0;if(s.getProgramParameter(y,s.LINK_STATUS)===!1)if(K=!1,typeof i.debug.onShaderError=="function")i.debug.onShaderError(s,y,R,w);else{let ct=Rd(s,R,"vertex"),bt=Rd(s,w,"fragment");Ct("THREE.WebGLProgram: Shader Error "+s.getError()+" - VALIDATE_STATUS "+s.getProgramParameter(y,s.VALIDATE_STATUS)+`

Material Name: `+T.name+`
Material Type: `+T.type+`

Program Info Log: `+U+`
`+ct+`
`+bt)}else U!==""?Et("WebGLProgram: Program Info Log:",U):(L===""||B==="")&&(tt=!1);tt&&(T.diagnostics={runnable:K,programLog:U,vertexShader:{log:L,prefix:m},fragmentShader:{log:B,prefix:f}})}s.deleteShader(R),s.deleteShader(w),v=new Ss(s,y),A=jg(s,y)}let v;this.getUniforms=function(){return v===void 0&&I(this),v};let A;this.getAttributes=function(){return A===void 0&&I(this),A};let D=e.rendererExtensionParallelShaderCompile===!1;return this.isReady=function(){return D===!1&&(D=s.getProgramParameter(y,Gg)),D},this.destroy=function(){n.releaseStatesOfProgram(this),s.deleteProgram(y),this.program=void 0},this.type=e.shaderType,this.name=e.shaderName,this.id=Hg++,this.cacheKey=t,this.usedTimes=1,this.program=y,this.vertexShader=R,this.fragmentShader=w,this}var p_=0,Ec=class{constructor(){this.shaderCache=new Map,this.materialCache=new Map}update(t){let e=t.vertexShader,n=t.fragmentShader,s=this._getShaderStage(e),r=this._getShaderStage(n),a=this._getShaderCacheForMaterial(t);return a.has(s)===!1&&(a.add(s),s.usedTimes++),a.has(r)===!1&&(a.add(r),r.usedTimes++),this}remove(t){let e=this.materialCache.get(t);for(let n of e)n.usedTimes--,n.usedTimes===0&&this.shaderCache.delete(n.code);return this.materialCache.delete(t),this}getVertexShaderID(t){return this._getShaderStage(t.vertexShader).id}getFragmentShaderID(t){return this._getShaderStage(t.fragmentShader).id}dispose(){this.shaderCache.clear(),this.materialCache.clear()}_getShaderCacheForMaterial(t){let e=this.materialCache,n=e.get(t);return n===void 0&&(n=new Set,e.set(t,n)),n}_getShaderStage(t){let e=this.shaderCache,n=e.get(t);return n===void 0&&(n=new Tc(t),e.set(t,n)),n}},Tc=class{constructor(t){this.id=p_++,this.code=t,this.usedTimes=0}};function m_(i){return i===bi||i===pr||i===mr}function g_(i,t,e,n,s,r){let a=new Xs,o=new Ec,l=new Set,c=[],d=new Map,u=n.logarithmicDepthBuffer,h=n.precision,p={MeshDepthMaterial:"depth",MeshDistanceMaterial:"distance",MeshNormalMaterial:"normal",MeshBasicMaterial:"basic",MeshLambertMaterial:"lambert",MeshPhongMaterial:"phong",MeshToonMaterial:"toon",MeshStandardMaterial:"physical",MeshPhysicalMaterial:"physical",MeshMatcapMaterial:"matcap",LineBasicMaterial:"basic",LineDashedMaterial:"dashed",PointsMaterial:"points",ShadowMaterial:"shadow",SpriteMaterial:"sprite"};function _(v){return l.add(v),v===0?"uv":`uv${v}`}function y(v,A,D,T,O,H){let $=T.fog,U=O.geometry,L=v.isMeshStandardMaterial||v.isMeshLambertMaterial||v.isMeshPhongMaterial?T.environment:null,B=v.isMeshStandardMaterial||v.isMeshLambertMaterial&&!v.envMap||v.isMeshPhongMaterial&&!v.envMap,K=t.get(v.envMap||L,B),tt=K&&K.mapping===lr?K.image.height:null,ct=p[v.type];v.precision!==null&&(h=n.getMaxPrecision(v.precision),h!==v.precision&&Et("WebGLProgram.getParameters:",v.precision,"not supported, using",h,"instead."));let bt=U.morphAttributes.position||U.morphAttributes.normal||U.morphAttributes.color,vt=bt!==void 0?bt.length:0,Gt=0;U.morphAttributes.position!==void 0&&(Gt=1),U.morphAttributes.normal!==void 0&&(Gt=2),U.morphAttributes.color!==void 0&&(Gt=3);let Qt,Ut,J,pt;if(ct){let Dt=kn[ct];Qt=Dt.vertexShader,Ut=Dt.fragmentShader}else Qt=v.vertexShader,Ut=v.fragmentShader,o.update(v),J=o.getVertexShaderID(v),pt=o.getFragmentShaderID(v);let rt=i.getRenderTarget(),Tt=i.state.buffers.depth.getReversed(),Lt=O.isInstancedMesh===!0,Rt=O.isBatchedMesh===!0,de=!!v.map,Ht=!!v.matcap,ee=!!K,ce=!!v.aoMap,Vt=!!v.lightMap,Me=!!v.bumpMap,ue=!!v.normalMap,We=!!v.displacementMap,P=!!v.emissiveMap,we=!!v.metalnessMap,Wt=!!v.roughnessMap,ae=v.anisotropy>0,ht=v.clearcoat>0,me=v.dispersion>0,b=v.iridescence>0,g=v.sheen>0,F=v.transmission>0,q=ae&&!!v.anisotropyMap,Q=ht&&!!v.clearcoatMap,nt=ht&&!!v.clearcoatNormalMap,lt=ht&&!!v.clearcoatRoughnessMap,W=b&&!!v.iridescenceMap,Y=b&&!!v.iridescenceThicknessMap,mt=g&&!!v.sheenColorMap,xt=g&&!!v.sheenRoughnessMap,at=!!v.specularMap,it=!!v.specularColorMap,It=!!v.specularIntensityMap,Ft=F&&!!v.transmissionMap,Jt=F&&!!v.thicknessMap,C=!!v.gradientMap,st=!!v.alphaMap,X=v.alphaTest>0,gt=!!v.alphaHash,ot=!!v.extensions,j=xn;v.toneMapped&&(rt===null||rt.isXRRenderTarget===!0)&&(j=i.toneMapping);let Mt={shaderID:ct,shaderType:v.type,shaderName:v.name,vertexShader:Qt,fragmentShader:Ut,defines:v.defines,customVertexShaderID:J,customFragmentShaderID:pt,isRawShaderMaterial:v.isRawShaderMaterial===!0,glslVersion:v.glslVersion,precision:h,batching:Rt,batchingColor:Rt&&O._colorsTexture!==null,instancing:Lt,instancingColor:Lt&&O.instanceColor!==null,instancingMorph:Lt&&O.morphTexture!==null,outputColorSpace:rt===null?i.outputColorSpace:rt.isXRRenderTarget===!0?rt.texture.colorSpace:$t.workingColorSpace,alphaToCoverage:!!v.alphaToCoverage,map:de,matcap:Ht,envMap:ee,envMapMode:ee&&K.mapping,envMapCubeUVHeight:tt,aoMap:ce,lightMap:Vt,bumpMap:Me,normalMap:ue,displacementMap:We,emissiveMap:P,normalMapObjectSpace:ue&&v.normalMapType===nd,normalMapTangentSpace:ue&&v.normalMapType===Co,packedNormalMap:ue&&v.normalMapType===Co&&m_(v.normalMap.format),metalnessMap:we,roughnessMap:Wt,anisotropy:ae,anisotropyMap:q,clearcoat:ht,clearcoatMap:Q,clearcoatNormalMap:nt,clearcoatRoughnessMap:lt,dispersion:me,iridescence:b,iridescenceMap:W,iridescenceThicknessMap:Y,sheen:g,sheenColorMap:mt,sheenRoughnessMap:xt,specularMap:at,specularColorMap:it,specularIntensityMap:It,transmission:F,transmissionMap:Ft,thicknessMap:Jt,gradientMap:C,opaque:v.transparent===!1&&v.blending===ki&&v.alphaToCoverage===!1,alphaMap:st,alphaTest:X,alphaHash:gt,combine:v.combine,mapUv:de&&_(v.map.channel),aoMapUv:ce&&_(v.aoMap.channel),lightMapUv:Vt&&_(v.lightMap.channel),bumpMapUv:Me&&_(v.bumpMap.channel),normalMapUv:ue&&_(v.normalMap.channel),displacementMapUv:We&&_(v.displacementMap.channel),emissiveMapUv:P&&_(v.emissiveMap.channel),metalnessMapUv:we&&_(v.metalnessMap.channel),roughnessMapUv:Wt&&_(v.roughnessMap.channel),anisotropyMapUv:q&&_(v.anisotropyMap.channel),clearcoatMapUv:Q&&_(v.clearcoatMap.channel),clearcoatNormalMapUv:nt&&_(v.clearcoatNormalMap.channel),clearcoatRoughnessMapUv:lt&&_(v.clearcoatRoughnessMap.channel),iridescenceMapUv:W&&_(v.iridescenceMap.channel),iridescenceThicknessMapUv:Y&&_(v.iridescenceThicknessMap.channel),sheenColorMapUv:mt&&_(v.sheenColorMap.channel),sheenRoughnessMapUv:xt&&_(v.sheenRoughnessMap.channel),specularMapUv:at&&_(v.specularMap.channel),specularColorMapUv:it&&_(v.specularColorMap.channel),specularIntensityMapUv:It&&_(v.specularIntensityMap.channel),transmissionMapUv:Ft&&_(v.transmissionMap.channel),thicknessMapUv:Jt&&_(v.thicknessMap.channel),alphaMapUv:st&&_(v.alphaMap.channel),vertexTangents:!!U.attributes.tangent&&(ue||ae),vertexNormals:!!U.attributes.normal,vertexColors:v.vertexColors,vertexAlphas:v.vertexColors===!0&&!!U.attributes.color&&U.attributes.color.itemSize===4,pointsUvs:O.isPoints===!0&&!!U.attributes.uv&&(de||st),fog:!!$,useFog:v.fog===!0,fogExp2:!!$&&$.isFogExp2,flatShading:v.wireframe===!1&&(v.flatShading===!0||U.attributes.normal===void 0&&ue===!1&&(v.isMeshLambertMaterial||v.isMeshPhongMaterial||v.isMeshStandardMaterial||v.isMeshPhysicalMaterial)),sizeAttenuation:v.sizeAttenuation===!0,logarithmicDepthBuffer:u,reversedDepthBuffer:Tt,skinning:O.isSkinnedMesh===!0,morphTargets:U.morphAttributes.position!==void 0,morphNormals:U.morphAttributes.normal!==void 0,morphColors:U.morphAttributes.color!==void 0,morphTargetsCount:vt,morphTextureStride:Gt,numDirLights:A.directional.length,numPointLights:A.point.length,numSpotLights:A.spot.length,numSpotLightMaps:A.spotLightMap.length,numRectAreaLights:A.rectArea.length,numHemiLights:A.hemi.length,numDirLightShadows:A.directionalShadowMap.length,numPointLightShadows:A.pointShadowMap.length,numSpotLightShadows:A.spotShadowMap.length,numSpotLightShadowsWithMaps:A.numSpotLightShadowsWithMaps,numLightProbes:A.numLightProbes,numLightProbeGrids:H.length,numClippingPlanes:r.numPlanes,numClipIntersection:r.numIntersection,dithering:v.dithering,shadowMapEnabled:i.shadowMap.enabled&&D.length>0,shadowMapType:i.shadowMap.type,toneMapping:j,decodeVideoTexture:de&&v.map.isVideoTexture===!0&&$t.getTransfer(v.map.colorSpace)===te,decodeVideoTextureEmissive:P&&v.emissiveMap.isVideoTexture===!0&&$t.getTransfer(v.emissiveMap.colorSpace)===te,premultipliedAlpha:v.premultipliedAlpha,doubleSided:v.side===Un,flipSided:v.side===ze,useDepthPacking:v.depthPacking>=0,depthPacking:v.depthPacking||0,index0AttributeName:v.index0AttributeName,extensionClipCullDistance:ot&&v.extensions.clipCullDistance===!0&&e.has("WEBGL_clip_cull_distance"),extensionMultiDraw:(ot&&v.extensions.multiDraw===!0||Rt)&&e.has("WEBGL_multi_draw"),rendererExtensionParallelShaderCompile:e.has("KHR_parallel_shader_compile"),customProgramCacheKey:v.customProgramCacheKey()};return Mt.vertexUv1s=l.has(1),Mt.vertexUv2s=l.has(2),Mt.vertexUv3s=l.has(3),l.clear(),Mt}function m(v){let A=[];if(v.shaderID?A.push(v.shaderID):(A.push(v.customVertexShaderID),A.push(v.customFragmentShaderID)),v.defines!==void 0)for(let D in v.defines)A.push(D),A.push(v.defines[D]);return v.isRawShaderMaterial===!1&&(f(A,v),S(A,v),A.push(i.outputColorSpace)),A.push(v.customProgramCacheKey),A.join()}function f(v,A){v.push(A.precision),v.push(A.outputColorSpace),v.push(A.envMapMode),v.push(A.envMapCubeUVHeight),v.push(A.mapUv),v.push(A.alphaMapUv),v.push(A.lightMapUv),v.push(A.aoMapUv),v.push(A.bumpMapUv),v.push(A.normalMapUv),v.push(A.displacementMapUv),v.push(A.emissiveMapUv),v.push(A.metalnessMapUv),v.push(A.roughnessMapUv),v.push(A.anisotropyMapUv),v.push(A.clearcoatMapUv),v.push(A.clearcoatNormalMapUv),v.push(A.clearcoatRoughnessMapUv),v.push(A.iridescenceMapUv),v.push(A.iridescenceThicknessMapUv),v.push(A.sheenColorMapUv),v.push(A.sheenRoughnessMapUv),v.push(A.specularMapUv),v.push(A.specularColorMapUv),v.push(A.specularIntensityMapUv),v.push(A.transmissionMapUv),v.push(A.thicknessMapUv),v.push(A.combine),v.push(A.fogExp2),v.push(A.sizeAttenuation),v.push(A.morphTargetsCount),v.push(A.morphAttributeCount),v.push(A.numDirLights),v.push(A.numPointLights),v.push(A.numSpotLights),v.push(A.numSpotLightMaps),v.push(A.numHemiLights),v.push(A.numRectAreaLights),v.push(A.numDirLightShadows),v.push(A.numPointLightShadows),v.push(A.numSpotLightShadows),v.push(A.numSpotLightShadowsWithMaps),v.push(A.numLightProbes),v.push(A.shadowMapType),v.push(A.toneMapping),v.push(A.numClippingPlanes),v.push(A.numClipIntersection),v.push(A.depthPacking)}function S(v,A){a.disableAll(),A.instancing&&a.enable(0),A.instancingColor&&a.enable(1),A.instancingMorph&&a.enable(2),A.matcap&&a.enable(3),A.envMap&&a.enable(4),A.normalMapObjectSpace&&a.enable(5),A.normalMapTangentSpace&&a.enable(6),A.clearcoat&&a.enable(7),A.iridescence&&a.enable(8),A.alphaTest&&a.enable(9),A.vertexColors&&a.enable(10),A.vertexAlphas&&a.enable(11),A.vertexUv1s&&a.enable(12),A.vertexUv2s&&a.enable(13),A.vertexUv3s&&a.enable(14),A.vertexTangents&&a.enable(15),A.anisotropy&&a.enable(16),A.alphaHash&&a.enable(17),A.batching&&a.enable(18),A.dispersion&&a.enable(19),A.batchingColor&&a.enable(20),A.gradientMap&&a.enable(21),A.packedNormalMap&&a.enable(22),A.vertexNormals&&a.enable(23),v.push(a.mask),a.disableAll(),A.fog&&a.enable(0),A.useFog&&a.enable(1),A.flatShading&&a.enable(2),A.logarithmicDepthBuffer&&a.enable(3),A.reversedDepthBuffer&&a.enable(4),A.skinning&&a.enable(5),A.morphTargets&&a.enable(6),A.morphNormals&&a.enable(7),A.morphColors&&a.enable(8),A.premultipliedAlpha&&a.enable(9),A.shadowMapEnabled&&a.enable(10),A.doubleSided&&a.enable(11),A.flipSided&&a.enable(12),A.useDepthPacking&&a.enable(13),A.dithering&&a.enable(14),A.transmission&&a.enable(15),A.sheen&&a.enable(16),A.opaque&&a.enable(17),A.pointsUvs&&a.enable(18),A.decodeVideoTexture&&a.enable(19),A.decodeVideoTextureEmissive&&a.enable(20),A.alphaToCoverage&&a.enable(21),A.numLightProbeGrids>0&&a.enable(22),v.push(a.mask)}function M(v){let A=p[v.type],D;if(A){let T=kn[A];D=pd.clone(T.uniforms)}else D=v.uniforms;return D}function E(v,A){let D=d.get(A);return D!==void 0?++D.usedTimes:(D=new f_(i,A,v,s),c.push(D),d.set(A,D)),D}function R(v){if(--v.usedTimes===0){let A=c.indexOf(v);c[A]=c[c.length-1],c.pop(),d.delete(v.cacheKey),v.destroy()}}function w(v){o.remove(v)}function I(){o.dispose()}return{getParameters:y,getProgramCacheKey:m,getUniforms:M,acquireProgram:E,releaseProgram:R,releaseShaderCache:w,programs:c,dispose:I}}function __(){let i=new WeakMap;function t(a){return i.has(a)}function e(a){let o=i.get(a);return o===void 0&&(o={},i.set(a,o)),o}function n(a){i.delete(a)}function s(a,o,l){i.get(a)[o]=l}function r(){i=new WeakMap}return{has:t,get:e,remove:n,update:s,dispose:r}}function v_(i,t){return i.groupOrder!==t.groupOrder?i.groupOrder-t.groupOrder:i.renderOrder!==t.renderOrder?i.renderOrder-t.renderOrder:i.material.id!==t.material.id?i.material.id-t.material.id:i.materialVariant!==t.materialVariant?i.materialVariant-t.materialVariant:i.z!==t.z?i.z-t.z:i.id-t.id}function Nd(i,t){return i.groupOrder!==t.groupOrder?i.groupOrder-t.groupOrder:i.renderOrder!==t.renderOrder?i.renderOrder-t.renderOrder:i.z!==t.z?t.z-i.z:i.id-t.id}function Ud(){let i=[],t=0,e=[],n=[],s=[];function r(){t=0,e.length=0,n.length=0,s.length=0}function a(h){let p=0;return h.isInstancedMesh&&(p+=2),h.isSkinnedMesh&&(p+=1),p}function o(h,p,_,y,m,f){let S=i[t];return S===void 0?(S={id:h.id,object:h,geometry:p,material:_,materialVariant:a(h),groupOrder:y,renderOrder:h.renderOrder,z:m,group:f},i[t]=S):(S.id=h.id,S.object=h,S.geometry=p,S.material=_,S.materialVariant=a(h),S.groupOrder=y,S.renderOrder=h.renderOrder,S.z=m,S.group=f),t++,S}function l(h,p,_,y,m,f){let S=o(h,p,_,y,m,f);_.transmission>0?n.push(S):_.transparent===!0?s.push(S):e.push(S)}function c(h,p,_,y,m,f){let S=o(h,p,_,y,m,f);_.transmission>0?n.unshift(S):_.transparent===!0?s.unshift(S):e.unshift(S)}function d(h,p){e.length>1&&e.sort(h||v_),n.length>1&&n.sort(p||Nd),s.length>1&&s.sort(p||Nd)}function u(){for(let h=t,p=i.length;h<p;h++){let _=i[h];if(_.id===null)break;_.id=null,_.object=null,_.geometry=null,_.material=null,_.group=null}}return{opaque:e,transmissive:n,transparent:s,init:r,push:l,unshift:c,finish:u,sort:d}}function x_(){let i=new WeakMap;function t(n,s){let r=i.get(n),a;return r===void 0?(a=new Ud,i.set(n,[a])):s>=r.length?(a=new Ud,r.push(a)):a=r[s],a}function e(){i=new WeakMap}return{get:t,dispose:e}}function y_(){let i={};return{get:function(t){if(i[t.id]!==void 0)return i[t.id];let e;switch(t.type){case"DirectionalLight":e={direction:new k,color:new zt};break;case"SpotLight":e={position:new k,direction:new k,color:new zt,distance:0,coneCos:0,penumbraCos:0,decay:0};break;case"PointLight":e={position:new k,color:new zt,distance:0,decay:0};break;case"HemisphereLight":e={direction:new k,skyColor:new zt,groundColor:new zt};break;case"RectAreaLight":e={color:new zt,position:new k,halfWidth:new k,halfHeight:new k};break}return i[t.id]=e,e}}}function b_(){let i={};return{get:function(t){if(i[t.id]!==void 0)return i[t.id];let e;switch(t.type){case"DirectionalLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new jt};break;case"SpotLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new jt};break;case"PointLight":e={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new jt,shadowCameraNear:1,shadowCameraFar:1e3};break}return i[t.id]=e,e}}}var S_=0;function M_(i,t){return(t.castShadow?2:0)-(i.castShadow?2:0)+(t.map?1:0)-(i.map?1:0)}function w_(i){let t=new y_,e=b_(),n={version:0,hash:{directionalLength:-1,pointLength:-1,spotLength:-1,rectAreaLength:-1,hemiLength:-1,numDirectionalShadows:-1,numPointShadows:-1,numSpotShadows:-1,numSpotMaps:-1,numLightProbes:-1},ambient:[0,0,0],probe:[],directional:[],directionalShadow:[],directionalShadowMap:[],directionalShadowMatrix:[],spot:[],spotLightMap:[],spotShadow:[],spotShadowMap:[],spotLightMatrix:[],rectArea:[],rectAreaLTC1:null,rectAreaLTC2:null,point:[],pointShadow:[],pointShadowMap:[],pointShadowMatrix:[],hemi:[],numSpotLightShadowsWithMaps:0,numLightProbes:0};for(let c=0;c<9;c++)n.probe.push(new k);let s=new k,r=new fe,a=new fe;function o(c){let d=0,u=0,h=0;for(let A=0;A<9;A++)n.probe[A].set(0,0,0);let p=0,_=0,y=0,m=0,f=0,S=0,M=0,E=0,R=0,w=0,I=0;c.sort(M_);for(let A=0,D=c.length;A<D;A++){let T=c[A],O=T.color,H=T.intensity,$=T.distance,U=null;if(T.shadow&&T.shadow.map&&(T.shadow.map.texture.format===bi?U=T.shadow.map.texture:U=T.shadow.map.depthTexture||T.shadow.map.texture),T.isAmbientLight)d+=O.r*H,u+=O.g*H,h+=O.b*H;else if(T.isLightProbe){for(let L=0;L<9;L++)n.probe[L].addScaledVector(T.sh.coefficients[L],H);I++}else if(T.isDirectionalLight){let L=t.get(T);if(L.color.copy(T.color).multiplyScalar(T.intensity),T.castShadow){let B=T.shadow,K=e.get(T);K.shadowIntensity=B.intensity,K.shadowBias=B.bias,K.shadowNormalBias=B.normalBias,K.shadowRadius=B.radius,K.shadowMapSize=B.mapSize,n.directionalShadow[p]=K,n.directionalShadowMap[p]=U,n.directionalShadowMatrix[p]=T.shadow.matrix,S++}n.directional[p]=L,p++}else if(T.isSpotLight){let L=t.get(T);L.position.setFromMatrixPosition(T.matrixWorld),L.color.copy(O).multiplyScalar(H),L.distance=$,L.coneCos=Math.cos(T.angle),L.penumbraCos=Math.cos(T.angle*(1-T.penumbra)),L.decay=T.decay,n.spot[y]=L;let B=T.shadow;if(T.map&&(n.spotLightMap[R]=T.map,R++,B.updateMatrices(T),T.castShadow&&w++),n.spotLightMatrix[y]=B.matrix,T.castShadow){let K=e.get(T);K.shadowIntensity=B.intensity,K.shadowBias=B.bias,K.shadowNormalBias=B.normalBias,K.shadowRadius=B.radius,K.shadowMapSize=B.mapSize,n.spotShadow[y]=K,n.spotShadowMap[y]=U,E++}y++}else if(T.isRectAreaLight){let L=t.get(T);L.color.copy(O).multiplyScalar(H),L.halfWidth.set(T.width*.5,0,0),L.halfHeight.set(0,T.height*.5,0),n.rectArea[m]=L,m++}else if(T.isPointLight){let L=t.get(T);if(L.color.copy(T.color).multiplyScalar(T.intensity),L.distance=T.distance,L.decay=T.decay,T.castShadow){let B=T.shadow,K=e.get(T);K.shadowIntensity=B.intensity,K.shadowBias=B.bias,K.shadowNormalBias=B.normalBias,K.shadowRadius=B.radius,K.shadowMapSize=B.mapSize,K.shadowCameraNear=B.camera.near,K.shadowCameraFar=B.camera.far,n.pointShadow[_]=K,n.pointShadowMap[_]=U,n.pointShadowMatrix[_]=T.shadow.matrix,M++}n.point[_]=L,_++}else if(T.isHemisphereLight){let L=t.get(T);L.skyColor.copy(T.color).multiplyScalar(H),L.groundColor.copy(T.groundColor).multiplyScalar(H),n.hemi[f]=L,f++}}m>0&&(i.has("OES_texture_float_linear")===!0?(n.rectAreaLTC1=dt.LTC_FLOAT_1,n.rectAreaLTC2=dt.LTC_FLOAT_2):(n.rectAreaLTC1=dt.LTC_HALF_1,n.rectAreaLTC2=dt.LTC_HALF_2)),n.ambient[0]=d,n.ambient[1]=u,n.ambient[2]=h;let v=n.hash;(v.directionalLength!==p||v.pointLength!==_||v.spotLength!==y||v.rectAreaLength!==m||v.hemiLength!==f||v.numDirectionalShadows!==S||v.numPointShadows!==M||v.numSpotShadows!==E||v.numSpotMaps!==R||v.numLightProbes!==I)&&(n.directional.length=p,n.spot.length=y,n.rectArea.length=m,n.point.length=_,n.hemi.length=f,n.directionalShadow.length=S,n.directionalShadowMap.length=S,n.pointShadow.length=M,n.pointShadowMap.length=M,n.spotShadow.length=E,n.spotShadowMap.length=E,n.directionalShadowMatrix.length=S,n.pointShadowMatrix.length=M,n.spotLightMatrix.length=E+R-w,n.spotLightMap.length=R,n.numSpotLightShadowsWithMaps=w,n.numLightProbes=I,v.directionalLength=p,v.pointLength=_,v.spotLength=y,v.rectAreaLength=m,v.hemiLength=f,v.numDirectionalShadows=S,v.numPointShadows=M,v.numSpotShadows=E,v.numSpotMaps=R,v.numLightProbes=I,n.version=S_++)}function l(c,d){let u=0,h=0,p=0,_=0,y=0,m=d.matrixWorldInverse;for(let f=0,S=c.length;f<S;f++){let M=c[f];if(M.isDirectionalLight){let E=n.directional[u];E.direction.setFromMatrixPosition(M.matrixWorld),s.setFromMatrixPosition(M.target.matrixWorld),E.direction.sub(s),E.direction.transformDirection(m),u++}else if(M.isSpotLight){let E=n.spot[p];E.position.setFromMatrixPosition(M.matrixWorld),E.position.applyMatrix4(m),E.direction.setFromMatrixPosition(M.matrixWorld),s.setFromMatrixPosition(M.target.matrixWorld),E.direction.sub(s),E.direction.transformDirection(m),p++}else if(M.isRectAreaLight){let E=n.rectArea[_];E.position.setFromMatrixPosition(M.matrixWorld),E.position.applyMatrix4(m),a.identity(),r.copy(M.matrixWorld),r.premultiply(m),a.extractRotation(r),E.halfWidth.set(M.width*.5,0,0),E.halfHeight.set(0,M.height*.5,0),E.halfWidth.applyMatrix4(a),E.halfHeight.applyMatrix4(a),_++}else if(M.isPointLight){let E=n.point[h];E.position.setFromMatrixPosition(M.matrixWorld),E.position.applyMatrix4(m),h++}else if(M.isHemisphereLight){let E=n.hemi[y];E.direction.setFromMatrixPosition(M.matrixWorld),E.direction.transformDirection(m),y++}}}return{setup:o,setupView:l,state:n}}function Fd(i){let t=new w_(i),e=[],n=[],s=[];function r(h){u.camera=h,e.length=0,n.length=0,s.length=0}function a(h){e.push(h)}function o(h){n.push(h)}function l(h){s.push(h)}function c(){t.setup(e)}function d(h){t.setupView(e,h)}let u={lightsArray:e,shadowsArray:n,lightProbeGridArray:s,camera:null,lights:t,transmissionRenderTarget:{},textureUnits:0};return{init:r,state:u,setupLights:c,setupLightsView:d,pushLight:a,pushShadow:o,pushLightProbeGrid:l}}function A_(i){let t=new WeakMap;function e(s,r=0){let a=t.get(s),o;return a===void 0?(o=new Fd(i),t.set(s,[o])):r>=a.length?(o=new Fd(i),a.push(o)):o=a[r],o}function n(){t=new WeakMap}return{get:e,dispose:n}}var E_=`void main() {
	gl_Position = vec4( position, 1.0 );
}`,T_=`uniform sampler2D shadow_pass;
uniform vec2 resolution;
uniform float radius;
void main() {
	const float samples = float( VSM_SAMPLES );
	float mean = 0.0;
	float squared_mean = 0.0;
	float uvStride = samples <= 1.0 ? 0.0 : 2.0 / ( samples - 1.0 );
	float uvStart = samples <= 1.0 ? 0.0 : - 1.0;
	for ( float i = 0.0; i < samples; i ++ ) {
		float uvOffset = uvStart + i * uvStride;
		#ifdef HORIZONTAL_PASS
			vec2 distribution = texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( uvOffset, 0.0 ) * radius ) / resolution ).rg;
			mean += distribution.x;
			squared_mean += distribution.y * distribution.y + distribution.x * distribution.x;
		#else
			float depth = texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( 0.0, uvOffset ) * radius ) / resolution ).r;
			mean += depth;
			squared_mean += depth * depth;
		#endif
	}
	mean = mean / samples;
	squared_mean = squared_mean / samples;
	float std_dev = sqrt( max( 0.0, squared_mean - mean * mean ) );
	gl_FragColor = vec4( mean, std_dev, 0.0, 1.0 );
}`,C_=[new k(1,0,0),new k(-1,0,0),new k(0,1,0),new k(0,-1,0),new k(0,0,1),new k(0,0,-1)],R_=[new k(0,-1,0),new k(0,-1,0),new k(0,0,1),new k(0,0,-1),new k(0,-1,0),new k(0,-1,0)],Od=new fe,vr=new k,yc=new k;function I_(i,t,e){let n=new ms,s=new jt,r=new jt,a=new pe,o=new Ca,l=new Ra,c={},d=e.maxTextureSize,u={[Zn]:ze,[ze]:Zn,[Un]:Un},h=new je({defines:{VSM_SAMPLES:8},uniforms:{shadow_pass:{value:null},resolution:{value:new jt},radius:{value:4}},vertexShader:E_,fragmentShader:T_}),p=h.clone();p.defines.HORIZONTAL_PASS=1;let _=new cn;_.setAttribute("position",new Je(new Float32Array([-1,-1,.5,3,-1,.5,-1,3,.5]),3));let y=new ke(_,h),m=this;this.enabled=!1,this.autoUpdate=!0,this.needsUpdate=!1,this.type=or;let f=this.type;this.render=function(w,I,v){if(m.enabled===!1||m.autoUpdate===!1&&m.needsUpdate===!1||w.length===0)return;this.type===Dh&&(Et("WebGLShadowMap: PCFSoftShadowMap has been deprecated. Using PCFShadowMap instead."),this.type=or);let A=i.getRenderTarget(),D=i.getActiveCubeFace(),T=i.getActiveMipmapLevel(),O=i.state;O.setBlending(Fn),O.buffers.depth.getReversed()===!0?O.buffers.color.setClear(0,0,0,0):O.buffers.color.setClear(1,1,1,1),O.buffers.depth.setTest(!0),O.setScissorTest(!1);let H=f!==this.type;H&&I.traverse(function($){$.material&&(Array.isArray($.material)?$.material.forEach(U=>U.needsUpdate=!0):$.material.needsUpdate=!0)});for(let $=0,U=w.length;$<U;$++){let L=w[$],B=L.shadow;if(B===void 0){Et("WebGLShadowMap:",L,"has no shadow.");continue}if(B.autoUpdate===!1&&B.needsUpdate===!1)continue;s.copy(B.mapSize);let K=B.getFrameExtents();s.multiply(K),r.copy(B.mapSize),(s.x>d||s.y>d)&&(s.x>d&&(r.x=Math.floor(d/K.x),s.x=r.x*K.x,B.mapSize.x=r.x),s.y>d&&(r.y=Math.floor(d/K.y),s.y=r.y*K.y,B.mapSize.y=r.y));let tt=i.state.buffers.depth.getReversed();if(B.camera._reversedDepth=tt,B.map===null||H===!0){if(B.map!==null&&(B.map.depthTexture!==null&&(B.map.depthTexture.dispose(),B.map.depthTexture=null),B.map.dispose()),this.type===vs){if(L.isPointLight){Et("WebGLShadowMap: VSM shadow maps are not supported for PointLights. Use PCF or BasicShadowMap instead.");continue}B.map=new Ke(s.x,s.y,{format:bi,type:On,minFilter:Pe,magFilter:Pe,generateMipmaps:!1}),B.map.texture.name=L.name+".shadowMap",B.map.depthTexture=new jn(s.x,s.y,bn),B.map.depthTexture.name=L.name+".shadowMapDepth",B.map.depthTexture.format=Ln,B.map.depthTexture.compareFunction=null,B.map.depthTexture.minFilter=Re,B.map.depthTexture.magFilter=Re}else L.isPointLight?(B.map=new No(s.x),B.map.depthTexture=new Ea(s.x,yn)):(B.map=new Ke(s.x,s.y),B.map.depthTexture=new jn(s.x,s.y,yn)),B.map.depthTexture.name=L.name+".shadowMap",B.map.depthTexture.format=Ln,this.type===or?(B.map.depthTexture.compareFunction=tt?Io:Ro,B.map.depthTexture.minFilter=Pe,B.map.depthTexture.magFilter=Pe):(B.map.depthTexture.compareFunction=null,B.map.depthTexture.minFilter=Re,B.map.depthTexture.magFilter=Re);B.camera.updateProjectionMatrix()}let ct=B.map.isWebGLCubeRenderTarget?6:1;for(let bt=0;bt<ct;bt++){if(B.map.isWebGLCubeRenderTarget)i.setRenderTarget(B.map,bt),i.clear();else{bt===0&&(i.setRenderTarget(B.map),i.clear());let vt=B.getViewport(bt);a.set(r.x*vt.x,r.y*vt.y,r.x*vt.z,r.y*vt.w),O.viewport(a)}if(L.isPointLight){let vt=B.camera,Gt=B.matrix,Qt=L.distance||vt.far;Qt!==vt.far&&(vt.far=Qt,vt.updateProjectionMatrix()),vr.setFromMatrixPosition(L.matrixWorld),vt.position.copy(vr),yc.copy(vt.position),yc.add(C_[bt]),vt.up.copy(R_[bt]),vt.lookAt(yc),vt.updateMatrixWorld(),Gt.makeTranslation(-vr.x,-vr.y,-vr.z),Od.multiplyMatrices(vt.projectionMatrix,vt.matrixWorldInverse),B._frustum.setFromProjectionMatrix(Od,vt.coordinateSystem,vt.reversedDepth)}else B.updateMatrices(L);n=B.getFrustum(),E(I,v,B.camera,L,this.type)}B.isPointLightShadow!==!0&&this.type===vs&&S(B,v),B.needsUpdate=!1}f=this.type,m.needsUpdate=!1,i.setRenderTarget(A,D,T)};function S(w,I){let v=t.update(y);h.defines.VSM_SAMPLES!==w.blurSamples&&(h.defines.VSM_SAMPLES=w.blurSamples,p.defines.VSM_SAMPLES=w.blurSamples,h.needsUpdate=!0,p.needsUpdate=!0),w.mapPass===null&&(w.mapPass=new Ke(s.x,s.y,{format:bi,type:On})),h.uniforms.shadow_pass.value=w.map.depthTexture,h.uniforms.resolution.value=w.mapSize,h.uniforms.radius.value=w.radius,i.setRenderTarget(w.mapPass),i.clear(),i.renderBufferDirect(I,null,v,h,y,null),p.uniforms.shadow_pass.value=w.mapPass.texture,p.uniforms.resolution.value=w.mapSize,p.uniforms.radius.value=w.radius,i.setRenderTarget(w.map),i.clear(),i.renderBufferDirect(I,null,v,p,y,null)}function M(w,I,v,A){let D=null,T=v.isPointLight===!0?w.customDistanceMaterial:w.customDepthMaterial;if(T!==void 0)D=T;else if(D=v.isPointLight===!0?l:o,i.localClippingEnabled&&I.clipShadows===!0&&Array.isArray(I.clippingPlanes)&&I.clippingPlanes.length!==0||I.displacementMap&&I.displacementScale!==0||I.alphaMap&&I.alphaTest>0||I.map&&I.alphaTest>0||I.alphaToCoverage===!0){let O=D.uuid,H=I.uuid,$=c[O];$===void 0&&($={},c[O]=$);let U=$[H];U===void 0&&(U=D.clone(),$[H]=U,I.addEventListener("dispose",R)),D=U}if(D.visible=I.visible,D.wireframe=I.wireframe,A===vs?D.side=I.shadowSide!==null?I.shadowSide:I.side:D.side=I.shadowSide!==null?I.shadowSide:u[I.side],D.alphaMap=I.alphaMap,D.alphaTest=I.alphaToCoverage===!0?.5:I.alphaTest,D.map=I.map,D.clipShadows=I.clipShadows,D.clippingPlanes=I.clippingPlanes,D.clipIntersection=I.clipIntersection,D.displacementMap=I.displacementMap,D.displacementScale=I.displacementScale,D.displacementBias=I.displacementBias,D.wireframeLinewidth=I.wireframeLinewidth,D.linewidth=I.linewidth,v.isPointLight===!0&&D.isMeshDistanceMaterial===!0){let O=i.properties.get(D);O.light=v}return D}function E(w,I,v,A,D){if(w.visible===!1)return;if(w.layers.test(I.layers)&&(w.isMesh||w.isLine||w.isPoints)&&(w.castShadow||w.receiveShadow&&D===vs)&&(!w.frustumCulled||n.intersectsObject(w))){w.modelViewMatrix.multiplyMatrices(v.matrixWorldInverse,w.matrixWorld);let H=t.update(w),$=w.material;if(Array.isArray($)){let U=H.groups;for(let L=0,B=U.length;L<B;L++){let K=U[L],tt=$[K.materialIndex];if(tt&&tt.visible){let ct=M(w,tt,A,D);w.onBeforeShadow(i,w,I,v,H,ct,K),i.renderBufferDirect(v,null,H,ct,w,K),w.onAfterShadow(i,w,I,v,H,ct,K)}}}else if($.visible){let U=M(w,$,A,D);w.onBeforeShadow(i,w,I,v,H,U,null),i.renderBufferDirect(v,null,H,U,w,null),w.onAfterShadow(i,w,I,v,H,U,null)}}let O=w.children;for(let H=0,$=O.length;H<$;H++)E(O[H],I,v,A,D)}function R(w){w.target.removeEventListener("dispose",R);for(let v in c){let A=c[v],D=w.target.uuid;D in A&&(A[D].dispose(),delete A[D])}}}function P_(i,t){function e(){let C=!1,st=new pe,X=null,gt=new pe(0,0,0,0);return{setMask:function(ot){X!==ot&&!C&&(i.colorMask(ot,ot,ot,ot),X=ot)},setLocked:function(ot){C=ot},setClear:function(ot,j,Mt,Dt,_e){_e===!0&&(ot*=Dt,j*=Dt,Mt*=Dt),st.set(ot,j,Mt,Dt),gt.equals(st)===!1&&(i.clearColor(ot,j,Mt,Dt),gt.copy(st))},reset:function(){C=!1,X=null,gt.set(-1,0,0,0)}}}function n(){let C=!1,st=!1,X=null,gt=null,ot=null;return{setReversed:function(j){if(st!==j){let Mt=t.get("EXT_clip_control");j?Mt.clipControlEXT(Mt.LOWER_LEFT_EXT,Mt.ZERO_TO_ONE_EXT):Mt.clipControlEXT(Mt.LOWER_LEFT_EXT,Mt.NEGATIVE_ONE_TO_ONE_EXT),st=j;let Dt=ot;ot=null,this.setClear(Dt)}},getReversed:function(){return st},setTest:function(j){j?rt(i.DEPTH_TEST):Tt(i.DEPTH_TEST)},setMask:function(j){X!==j&&!C&&(i.depthMask(j),X=j)},setFunc:function(j){if(st&&(j=ud[j]),gt!==j){switch(j){case aa:i.depthFunc(i.NEVER);break;case oa:i.depthFunc(i.ALWAYS);break;case la:i.depthFunc(i.LESS);break;case zi:i.depthFunc(i.LEQUAL);break;case ca:i.depthFunc(i.EQUAL);break;case ha:i.depthFunc(i.GEQUAL);break;case da:i.depthFunc(i.GREATER);break;case ua:i.depthFunc(i.NOTEQUAL);break;default:i.depthFunc(i.LEQUAL)}gt=j}},setLocked:function(j){C=j},setClear:function(j){ot!==j&&(ot=j,st&&(j=1-j),i.clearDepth(j))},reset:function(){C=!1,X=null,gt=null,ot=null,st=!1}}}function s(){let C=!1,st=null,X=null,gt=null,ot=null,j=null,Mt=null,Dt=null,_e=null;return{setTest:function(ne){C||(ne?rt(i.STENCIL_TEST):Tt(i.STENCIL_TEST))},setMask:function(ne){st!==ne&&!C&&(i.stencilMask(ne),st=ne)},setFunc:function(ne,Vn,Mn){(X!==ne||gt!==Vn||ot!==Mn)&&(i.stencilFunc(ne,Vn,Mn),X=ne,gt=Vn,ot=Mn)},setOp:function(ne,Vn,Mn){(j!==ne||Mt!==Vn||Dt!==Mn)&&(i.stencilOp(ne,Vn,Mn),j=ne,Mt=Vn,Dt=Mn)},setLocked:function(ne){C=ne},setClear:function(ne){_e!==ne&&(i.clearStencil(ne),_e=ne)},reset:function(){C=!1,st=null,X=null,gt=null,ot=null,j=null,Mt=null,Dt=null,_e=null}}}let r=new e,a=new n,o=new s,l=new WeakMap,c=new WeakMap,d={},u={},h={},p=new WeakMap,_=[],y=null,m=!1,f=null,S=null,M=null,E=null,R=null,w=null,I=null,v=new zt(0,0,0),A=0,D=!1,T=null,O=null,H=null,$=null,U=null,L=i.getParameter(i.MAX_COMBINED_TEXTURE_IMAGE_UNITS),B=!1,K=0,tt=i.getParameter(i.VERSION);tt.indexOf("WebGL")!==-1?(K=parseFloat(/^WebGL (\d)/.exec(tt)[1]),B=K>=1):tt.indexOf("OpenGL ES")!==-1&&(K=parseFloat(/^OpenGL ES (\d)/.exec(tt)[1]),B=K>=2);let ct=null,bt={},vt=i.getParameter(i.SCISSOR_BOX),Gt=i.getParameter(i.VIEWPORT),Qt=new pe().fromArray(vt),Ut=new pe().fromArray(Gt);function J(C,st,X,gt){let ot=new Uint8Array(4),j=i.createTexture();i.bindTexture(C,j),i.texParameteri(C,i.TEXTURE_MIN_FILTER,i.NEAREST),i.texParameteri(C,i.TEXTURE_MAG_FILTER,i.NEAREST);for(let Mt=0;Mt<X;Mt++)C===i.TEXTURE_3D||C===i.TEXTURE_2D_ARRAY?i.texImage3D(st,0,i.RGBA,1,1,gt,0,i.RGBA,i.UNSIGNED_BYTE,ot):i.texImage2D(st+Mt,0,i.RGBA,1,1,0,i.RGBA,i.UNSIGNED_BYTE,ot);return j}let pt={};pt[i.TEXTURE_2D]=J(i.TEXTURE_2D,i.TEXTURE_2D,1),pt[i.TEXTURE_CUBE_MAP]=J(i.TEXTURE_CUBE_MAP,i.TEXTURE_CUBE_MAP_POSITIVE_X,6),pt[i.TEXTURE_2D_ARRAY]=J(i.TEXTURE_2D_ARRAY,i.TEXTURE_2D_ARRAY,1,1),pt[i.TEXTURE_3D]=J(i.TEXTURE_3D,i.TEXTURE_3D,1,1),r.setClear(0,0,0,1),a.setClear(1),o.setClear(0),rt(i.DEPTH_TEST),a.setFunc(zi),Me(!1),ue(Hl),rt(i.CULL_FACE),ce(Fn);function rt(C){d[C]!==!0&&(i.enable(C),d[C]=!0)}function Tt(C){d[C]!==!1&&(i.disable(C),d[C]=!1)}function Lt(C,st){return h[C]!==st?(i.bindFramebuffer(C,st),h[C]=st,C===i.DRAW_FRAMEBUFFER&&(h[i.FRAMEBUFFER]=st),C===i.FRAMEBUFFER&&(h[i.DRAW_FRAMEBUFFER]=st),!0):!1}function Rt(C,st){let X=_,gt=!1;if(C){X=p.get(st),X===void 0&&(X=[],p.set(st,X));let ot=C.textures;if(X.length!==ot.length||X[0]!==i.COLOR_ATTACHMENT0){for(let j=0,Mt=ot.length;j<Mt;j++)X[j]=i.COLOR_ATTACHMENT0+j;X.length=ot.length,gt=!0}}else X[0]!==i.BACK&&(X[0]=i.BACK,gt=!0);gt&&i.drawBuffers(X)}function de(C){return y!==C?(i.useProgram(C),y=C,!0):!1}let Ht={[ui]:i.FUNC_ADD,[Uh]:i.FUNC_SUBTRACT,[Fh]:i.FUNC_REVERSE_SUBTRACT};Ht[Oh]=i.MIN,Ht[Bh]=i.MAX;let ee={[kh]:i.ZERO,[zh]:i.ONE,[Vh]:i.SRC_COLOR,[sa]:i.SRC_ALPHA,[qh]:i.SRC_ALPHA_SATURATE,[$h]:i.DST_COLOR,[Hh]:i.DST_ALPHA,[Gh]:i.ONE_MINUS_SRC_COLOR,[ra]:i.ONE_MINUS_SRC_ALPHA,[Xh]:i.ONE_MINUS_DST_COLOR,[Wh]:i.ONE_MINUS_DST_ALPHA,[Yh]:i.CONSTANT_COLOR,[Zh]:i.ONE_MINUS_CONSTANT_COLOR,[Jh]:i.CONSTANT_ALPHA,[Kh]:i.ONE_MINUS_CONSTANT_ALPHA};function ce(C,st,X,gt,ot,j,Mt,Dt,_e,ne){if(C===Fn){m===!0&&(Tt(i.BLEND),m=!1);return}if(m===!1&&(rt(i.BLEND),m=!0),C!==Nh){if(C!==f||ne!==D){if((S!==ui||R!==ui)&&(i.blendEquation(i.FUNC_ADD),S=ui,R=ui),ne)switch(C){case ki:i.blendFuncSeparate(i.ONE,i.ONE_MINUS_SRC_ALPHA,i.ONE,i.ONE_MINUS_SRC_ALPHA);break;case Wl:i.blendFunc(i.ONE,i.ONE);break;case $l:i.blendFuncSeparate(i.ZERO,i.ONE_MINUS_SRC_COLOR,i.ZERO,i.ONE);break;case Xl:i.blendFuncSeparate(i.DST_COLOR,i.ONE_MINUS_SRC_ALPHA,i.ZERO,i.ONE);break;default:Ct("WebGLState: Invalid blending: ",C);break}else switch(C){case ki:i.blendFuncSeparate(i.SRC_ALPHA,i.ONE_MINUS_SRC_ALPHA,i.ONE,i.ONE_MINUS_SRC_ALPHA);break;case Wl:i.blendFuncSeparate(i.SRC_ALPHA,i.ONE,i.ONE,i.ONE);break;case $l:Ct("WebGLState: SubtractiveBlending requires material.premultipliedAlpha = true");break;case Xl:Ct("WebGLState: MultiplyBlending requires material.premultipliedAlpha = true");break;default:Ct("WebGLState: Invalid blending: ",C);break}M=null,E=null,w=null,I=null,v.set(0,0,0),A=0,f=C,D=ne}return}ot=ot||st,j=j||X,Mt=Mt||gt,(st!==S||ot!==R)&&(i.blendEquationSeparate(Ht[st],Ht[ot]),S=st,R=ot),(X!==M||gt!==E||j!==w||Mt!==I)&&(i.blendFuncSeparate(ee[X],ee[gt],ee[j],ee[Mt]),M=X,E=gt,w=j,I=Mt),(Dt.equals(v)===!1||_e!==A)&&(i.blendColor(Dt.r,Dt.g,Dt.b,_e),v.copy(Dt),A=_e),f=C,D=!1}function Vt(C,st){C.side===Un?Tt(i.CULL_FACE):rt(i.CULL_FACE);let X=C.side===ze;st&&(X=!X),Me(X),C.blending===ki&&C.transparent===!1?ce(Fn):ce(C.blending,C.blendEquation,C.blendSrc,C.blendDst,C.blendEquationAlpha,C.blendSrcAlpha,C.blendDstAlpha,C.blendColor,C.blendAlpha,C.premultipliedAlpha),a.setFunc(C.depthFunc),a.setTest(C.depthTest),a.setMask(C.depthWrite),r.setMask(C.colorWrite);let gt=C.stencilWrite;o.setTest(gt),gt&&(o.setMask(C.stencilWriteMask),o.setFunc(C.stencilFunc,C.stencilRef,C.stencilFuncMask),o.setOp(C.stencilFail,C.stencilZFail,C.stencilZPass)),P(C.polygonOffset,C.polygonOffsetFactor,C.polygonOffsetUnits),C.alphaToCoverage===!0?rt(i.SAMPLE_ALPHA_TO_COVERAGE):Tt(i.SAMPLE_ALPHA_TO_COVERAGE)}function Me(C){T!==C&&(C?i.frontFace(i.CW):i.frontFace(i.CCW),T=C)}function ue(C){C!==Ph?(rt(i.CULL_FACE),C!==O&&(C===Hl?i.cullFace(i.BACK):C===Lh?i.cullFace(i.FRONT):i.cullFace(i.FRONT_AND_BACK))):Tt(i.CULL_FACE),O=C}function We(C){C!==H&&(B&&i.lineWidth(C),H=C)}function P(C,st,X){C?(rt(i.POLYGON_OFFSET_FILL),($!==st||U!==X)&&($=st,U=X,a.getReversed()&&(st=-st),i.polygonOffset(st,X))):Tt(i.POLYGON_OFFSET_FILL)}function we(C){C?rt(i.SCISSOR_TEST):Tt(i.SCISSOR_TEST)}function Wt(C){C===void 0&&(C=i.TEXTURE0+L-1),ct!==C&&(i.activeTexture(C),ct=C)}function ae(C,st,X){X===void 0&&(ct===null?X=i.TEXTURE0+L-1:X=ct);let gt=bt[X];gt===void 0&&(gt={type:void 0,texture:void 0},bt[X]=gt),(gt.type!==C||gt.texture!==st)&&(ct!==X&&(i.activeTexture(X),ct=X),i.bindTexture(C,st||pt[C]),gt.type=C,gt.texture=st)}function ht(){let C=bt[ct];C!==void 0&&C.type!==void 0&&(i.bindTexture(C.type,null),C.type=void 0,C.texture=void 0)}function me(){try{i.compressedTexImage2D(...arguments)}catch(C){Ct("WebGLState:",C)}}function b(){try{i.compressedTexImage3D(...arguments)}catch(C){Ct("WebGLState:",C)}}function g(){try{i.texSubImage2D(...arguments)}catch(C){Ct("WebGLState:",C)}}function F(){try{i.texSubImage3D(...arguments)}catch(C){Ct("WebGLState:",C)}}function q(){try{i.compressedTexSubImage2D(...arguments)}catch(C){Ct("WebGLState:",C)}}function Q(){try{i.compressedTexSubImage3D(...arguments)}catch(C){Ct("WebGLState:",C)}}function nt(){try{i.texStorage2D(...arguments)}catch(C){Ct("WebGLState:",C)}}function lt(){try{i.texStorage3D(...arguments)}catch(C){Ct("WebGLState:",C)}}function W(){try{i.texImage2D(...arguments)}catch(C){Ct("WebGLState:",C)}}function Y(){try{i.texImage3D(...arguments)}catch(C){Ct("WebGLState:",C)}}function mt(C){return u[C]!==void 0?u[C]:i.getParameter(C)}function xt(C,st){u[C]!==st&&(i.pixelStorei(C,st),u[C]=st)}function at(C){Qt.equals(C)===!1&&(i.scissor(C.x,C.y,C.z,C.w),Qt.copy(C))}function it(C){Ut.equals(C)===!1&&(i.viewport(C.x,C.y,C.z,C.w),Ut.copy(C))}function It(C,st){let X=c.get(st);X===void 0&&(X=new WeakMap,c.set(st,X));let gt=X.get(C);gt===void 0&&(gt=i.getUniformBlockIndex(st,C.name),X.set(C,gt))}function Ft(C,st){let gt=c.get(st).get(C);l.get(st)!==gt&&(i.uniformBlockBinding(st,gt,C.__bindingPointIndex),l.set(st,gt))}function Jt(){i.disable(i.BLEND),i.disable(i.CULL_FACE),i.disable(i.DEPTH_TEST),i.disable(i.POLYGON_OFFSET_FILL),i.disable(i.SCISSOR_TEST),i.disable(i.STENCIL_TEST),i.disable(i.SAMPLE_ALPHA_TO_COVERAGE),i.blendEquation(i.FUNC_ADD),i.blendFunc(i.ONE,i.ZERO),i.blendFuncSeparate(i.ONE,i.ZERO,i.ONE,i.ZERO),i.blendColor(0,0,0,0),i.colorMask(!0,!0,!0,!0),i.clearColor(0,0,0,0),i.depthMask(!0),i.depthFunc(i.LESS),a.setReversed(!1),i.clearDepth(1),i.stencilMask(4294967295),i.stencilFunc(i.ALWAYS,0,4294967295),i.stencilOp(i.KEEP,i.KEEP,i.KEEP),i.clearStencil(0),i.cullFace(i.BACK),i.frontFace(i.CCW),i.polygonOffset(0,0),i.activeTexture(i.TEXTURE0),i.bindFramebuffer(i.FRAMEBUFFER,null),i.bindFramebuffer(i.DRAW_FRAMEBUFFER,null),i.bindFramebuffer(i.READ_FRAMEBUFFER,null),i.useProgram(null),i.lineWidth(1),i.scissor(0,0,i.canvas.width,i.canvas.height),i.viewport(0,0,i.canvas.width,i.canvas.height),i.pixelStorei(i.PACK_ALIGNMENT,4),i.pixelStorei(i.UNPACK_ALIGNMENT,4),i.pixelStorei(i.UNPACK_FLIP_Y_WEBGL,!1),i.pixelStorei(i.UNPACK_PREMULTIPLY_ALPHA_WEBGL,!1),i.pixelStorei(i.UNPACK_COLORSPACE_CONVERSION_WEBGL,i.BROWSER_DEFAULT_WEBGL),i.pixelStorei(i.PACK_ROW_LENGTH,0),i.pixelStorei(i.PACK_SKIP_PIXELS,0),i.pixelStorei(i.PACK_SKIP_ROWS,0),i.pixelStorei(i.UNPACK_ROW_LENGTH,0),i.pixelStorei(i.UNPACK_IMAGE_HEIGHT,0),i.pixelStorei(i.UNPACK_SKIP_PIXELS,0),i.pixelStorei(i.UNPACK_SKIP_ROWS,0),i.pixelStorei(i.UNPACK_SKIP_IMAGES,0),d={},u={},ct=null,bt={},h={},p=new WeakMap,_=[],y=null,m=!1,f=null,S=null,M=null,E=null,R=null,w=null,I=null,v=new zt(0,0,0),A=0,D=!1,T=null,O=null,H=null,$=null,U=null,Qt.set(0,0,i.canvas.width,i.canvas.height),Ut.set(0,0,i.canvas.width,i.canvas.height),r.reset(),a.reset(),o.reset()}return{buffers:{color:r,depth:a,stencil:o},enable:rt,disable:Tt,bindFramebuffer:Lt,drawBuffers:Rt,useProgram:de,setBlending:ce,setMaterial:Vt,setFlipSided:Me,setCullFace:ue,setLineWidth:We,setPolygonOffset:P,setScissorTest:we,activeTexture:Wt,bindTexture:ae,unbindTexture:ht,compressedTexImage2D:me,compressedTexImage3D:b,texImage2D:W,texImage3D:Y,pixelStorei:xt,getParameter:mt,updateUBOMapping:It,uniformBlockBinding:Ft,texStorage2D:nt,texStorage3D:lt,texSubImage2D:g,texSubImage3D:F,compressedTexSubImage2D:q,compressedTexSubImage3D:Q,scissor:at,viewport:it,reset:Jt}}function L_(i,t,e,n,s,r,a){let o=t.has("WEBGL_multisampled_render_to_texture")?t.get("WEBGL_multisampled_render_to_texture"):null,l=typeof navigator>"u"?!1:/OculusBrowser/g.test(navigator.userAgent),c=new jt,d=new WeakMap,u=new Set,h,p=new WeakMap,_=!1;try{_=typeof OffscreenCanvas<"u"&&new OffscreenCanvas(1,1).getContext("2d")!==null}catch{}function y(b,g){return _?new OffscreenCanvas(b,g):Ws("canvas")}function m(b,g,F){let q=1,Q=me(b);if((Q.width>F||Q.height>F)&&(q=F/Math.max(Q.width,Q.height)),q<1)if(typeof HTMLImageElement<"u"&&b instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&b instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&b instanceof ImageBitmap||typeof VideoFrame<"u"&&b instanceof VideoFrame){let nt=Math.floor(q*Q.width),lt=Math.floor(q*Q.height);h===void 0&&(h=y(nt,lt));let W=g?y(nt,lt):h;return W.width=nt,W.height=lt,W.getContext("2d").drawImage(b,0,0,nt,lt),Et("WebGLRenderer: Texture has been resized from ("+Q.width+"x"+Q.height+") to ("+nt+"x"+lt+")."),W}else return"data"in b&&Et("WebGLRenderer: Image in DataTexture is too big ("+Q.width+"x"+Q.height+")."),b;return b}function f(b){return b.generateMipmaps}function S(b){i.generateMipmap(b)}function M(b){return b.isWebGLCubeRenderTarget?i.TEXTURE_CUBE_MAP:b.isWebGL3DRenderTarget?i.TEXTURE_3D:b.isWebGLArrayRenderTarget||b.isCompressedArrayTexture?i.TEXTURE_2D_ARRAY:i.TEXTURE_2D}function E(b,g,F,q,Q,nt=!1){if(b!==null){if(i[b]!==void 0)return i[b];Et("WebGLRenderer: Attempt to use non-existing WebGL internal format '"+b+"'")}let lt;q&&(lt=t.get("EXT_texture_norm16"),lt||Et("WebGLRenderer: Unable to use normalized textures without EXT_texture_norm16 extension"));let W=g;if(g===i.RED&&(F===i.FLOAT&&(W=i.R32F),F===i.HALF_FLOAT&&(W=i.R16F),F===i.UNSIGNED_BYTE&&(W=i.R8),F===i.UNSIGNED_SHORT&&lt&&(W=lt.R16_EXT),F===i.SHORT&&lt&&(W=lt.R16_SNORM_EXT)),g===i.RED_INTEGER&&(F===i.UNSIGNED_BYTE&&(W=i.R8UI),F===i.UNSIGNED_SHORT&&(W=i.R16UI),F===i.UNSIGNED_INT&&(W=i.R32UI),F===i.BYTE&&(W=i.R8I),F===i.SHORT&&(W=i.R16I),F===i.INT&&(W=i.R32I)),g===i.RG&&(F===i.FLOAT&&(W=i.RG32F),F===i.HALF_FLOAT&&(W=i.RG16F),F===i.UNSIGNED_BYTE&&(W=i.RG8),F===i.UNSIGNED_SHORT&&lt&&(W=lt.RG16_EXT),F===i.SHORT&&lt&&(W=lt.RG16_SNORM_EXT)),g===i.RG_INTEGER&&(F===i.UNSIGNED_BYTE&&(W=i.RG8UI),F===i.UNSIGNED_SHORT&&(W=i.RG16UI),F===i.UNSIGNED_INT&&(W=i.RG32UI),F===i.BYTE&&(W=i.RG8I),F===i.SHORT&&(W=i.RG16I),F===i.INT&&(W=i.RG32I)),g===i.RGB_INTEGER&&(F===i.UNSIGNED_BYTE&&(W=i.RGB8UI),F===i.UNSIGNED_SHORT&&(W=i.RGB16UI),F===i.UNSIGNED_INT&&(W=i.RGB32UI),F===i.BYTE&&(W=i.RGB8I),F===i.SHORT&&(W=i.RGB16I),F===i.INT&&(W=i.RGB32I)),g===i.RGBA_INTEGER&&(F===i.UNSIGNED_BYTE&&(W=i.RGBA8UI),F===i.UNSIGNED_SHORT&&(W=i.RGBA16UI),F===i.UNSIGNED_INT&&(W=i.RGBA32UI),F===i.BYTE&&(W=i.RGBA8I),F===i.SHORT&&(W=i.RGBA16I),F===i.INT&&(W=i.RGBA32I)),g===i.RGB&&(F===i.UNSIGNED_SHORT&&lt&&(W=lt.RGB16_EXT),F===i.SHORT&&lt&&(W=lt.RGB16_SNORM_EXT),F===i.UNSIGNED_INT_5_9_9_9_REV&&(W=i.RGB9_E5),F===i.UNSIGNED_INT_10F_11F_11F_REV&&(W=i.R11F_G11F_B10F)),g===i.RGBA){let Y=nt?Hs:$t.getTransfer(Q);F===i.FLOAT&&(W=i.RGBA32F),F===i.HALF_FLOAT&&(W=i.RGBA16F),F===i.UNSIGNED_BYTE&&(W=Y===te?i.SRGB8_ALPHA8:i.RGBA8),F===i.UNSIGNED_SHORT&&lt&&(W=lt.RGBA16_EXT),F===i.SHORT&&lt&&(W=lt.RGBA16_SNORM_EXT),F===i.UNSIGNED_SHORT_4_4_4_4&&(W=i.RGBA4),F===i.UNSIGNED_SHORT_5_5_5_1&&(W=i.RGB5_A1)}return(W===i.R16F||W===i.R32F||W===i.RG16F||W===i.RG32F||W===i.RGBA16F||W===i.RGBA32F)&&t.get("EXT_color_buffer_float"),W}function R(b,g){let F;return b?g===null||g===yn||g===ys?F=i.DEPTH24_STENCIL8:g===bn?F=i.DEPTH32F_STENCIL8:g===xs&&(F=i.DEPTH24_STENCIL8,Et("DepthTexture: 16 bit depth attachment is not supported with stencil. Using 24-bit attachment.")):g===null||g===yn||g===ys?F=i.DEPTH_COMPONENT24:g===bn?F=i.DEPTH_COMPONENT32F:g===xs&&(F=i.DEPTH_COMPONENT16),F}function w(b,g){return f(b)===!0||b.isFramebufferTexture&&b.minFilter!==Re&&b.minFilter!==Pe?Math.log2(Math.max(g.width,g.height))+1:b.mipmaps!==void 0&&b.mipmaps.length>0?b.mipmaps.length:b.isCompressedTexture&&Array.isArray(b.image)?g.mipmaps.length:1}function I(b){let g=b.target;g.removeEventListener("dispose",I),A(g),g.isVideoTexture&&d.delete(g),g.isHTMLTexture&&u.delete(g)}function v(b){let g=b.target;g.removeEventListener("dispose",v),T(g)}function A(b){let g=n.get(b);if(g.__webglInit===void 0)return;let F=b.source,q=p.get(F);if(q){let Q=q[g.__cacheKey];Q.usedTimes--,Q.usedTimes===0&&D(b),Object.keys(q).length===0&&p.delete(F)}n.remove(b)}function D(b){let g=n.get(b);i.deleteTexture(g.__webglTexture);let F=b.source,q=p.get(F);delete q[g.__cacheKey],a.memory.textures--}function T(b){let g=n.get(b);if(b.depthTexture&&(b.depthTexture.dispose(),n.remove(b.depthTexture)),b.isWebGLCubeRenderTarget)for(let q=0;q<6;q++){if(Array.isArray(g.__webglFramebuffer[q]))for(let Q=0;Q<g.__webglFramebuffer[q].length;Q++)i.deleteFramebuffer(g.__webglFramebuffer[q][Q]);else i.deleteFramebuffer(g.__webglFramebuffer[q]);g.__webglDepthbuffer&&i.deleteRenderbuffer(g.__webglDepthbuffer[q])}else{if(Array.isArray(g.__webglFramebuffer))for(let q=0;q<g.__webglFramebuffer.length;q++)i.deleteFramebuffer(g.__webglFramebuffer[q]);else i.deleteFramebuffer(g.__webglFramebuffer);if(g.__webglDepthbuffer&&i.deleteRenderbuffer(g.__webglDepthbuffer),g.__webglMultisampledFramebuffer&&i.deleteFramebuffer(g.__webglMultisampledFramebuffer),g.__webglColorRenderbuffer)for(let q=0;q<g.__webglColorRenderbuffer.length;q++)g.__webglColorRenderbuffer[q]&&i.deleteRenderbuffer(g.__webglColorRenderbuffer[q]);g.__webglDepthRenderbuffer&&i.deleteRenderbuffer(g.__webglDepthRenderbuffer)}let F=b.textures;for(let q=0,Q=F.length;q<Q;q++){let nt=n.get(F[q]);nt.__webglTexture&&(i.deleteTexture(nt.__webglTexture),a.memory.textures--),n.remove(F[q])}n.remove(b)}let O=0;function H(){O=0}function $(){return O}function U(b){O=b}function L(){let b=O;return b>=s.maxTextures&&Et("WebGLTextures: Trying to use "+b+" texture units while this GPU supports only "+s.maxTextures),O+=1,b}function B(b){let g=[];return g.push(b.wrapS),g.push(b.wrapT),g.push(b.wrapR||0),g.push(b.magFilter),g.push(b.minFilter),g.push(b.anisotropy),g.push(b.internalFormat),g.push(b.format),g.push(b.type),g.push(b.generateMipmaps),g.push(b.premultiplyAlpha),g.push(b.flipY),g.push(b.unpackAlignment),g.push(b.colorSpace),g.join()}function K(b,g){let F=n.get(b);if(b.isVideoTexture&&ae(b),b.isRenderTargetTexture===!1&&b.isExternalTexture!==!0&&b.version>0&&F.__version!==b.version){let q=b.image;if(q===null)Et("WebGLRenderer: Texture marked for update but no image data found.");else if(q.complete===!1)Et("WebGLRenderer: Texture marked for update but image is incomplete");else{Tt(F,b,g);return}}else b.isExternalTexture&&(F.__webglTexture=b.sourceTexture?b.sourceTexture:null);e.bindTexture(i.TEXTURE_2D,F.__webglTexture,i.TEXTURE0+g)}function tt(b,g){let F=n.get(b);if(b.isRenderTargetTexture===!1&&b.version>0&&F.__version!==b.version){Tt(F,b,g);return}else b.isExternalTexture&&(F.__webglTexture=b.sourceTexture?b.sourceTexture:null);e.bindTexture(i.TEXTURE_2D_ARRAY,F.__webglTexture,i.TEXTURE0+g)}function ct(b,g){let F=n.get(b);if(b.isRenderTargetTexture===!1&&b.version>0&&F.__version!==b.version){Tt(F,b,g);return}e.bindTexture(i.TEXTURE_3D,F.__webglTexture,i.TEXTURE0+g)}function bt(b,g){let F=n.get(b);if(b.isCubeDepthTexture!==!0&&b.version>0&&F.__version!==b.version){Lt(F,b,g);return}e.bindTexture(i.TEXTURE_CUBE_MAP,F.__webglTexture,i.TEXTURE0+g)}let vt={[fa]:i.REPEAT,[Pn]:i.CLAMP_TO_EDGE,[pa]:i.MIRRORED_REPEAT},Gt={[Re]:i.NEAREST,[td]:i.NEAREST_MIPMAP_NEAREST,[cr]:i.NEAREST_MIPMAP_LINEAR,[Pe]:i.LINEAR,[Wa]:i.LINEAR_MIPMAP_NEAREST,[xi]:i.LINEAR_MIPMAP_LINEAR},Qt={[id]:i.NEVER,[ld]:i.ALWAYS,[sd]:i.LESS,[Ro]:i.LEQUAL,[rd]:i.EQUAL,[Io]:i.GEQUAL,[ad]:i.GREATER,[od]:i.NOTEQUAL};function Ut(b,g){if(g.type===bn&&t.has("OES_texture_float_linear")===!1&&(g.magFilter===Pe||g.magFilter===Wa||g.magFilter===cr||g.magFilter===xi||g.minFilter===Pe||g.minFilter===Wa||g.minFilter===cr||g.minFilter===xi)&&Et("WebGLRenderer: Unable to use linear filtering with floating point textures. OES_texture_float_linear not supported on this device."),i.texParameteri(b,i.TEXTURE_WRAP_S,vt[g.wrapS]),i.texParameteri(b,i.TEXTURE_WRAP_T,vt[g.wrapT]),(b===i.TEXTURE_3D||b===i.TEXTURE_2D_ARRAY)&&i.texParameteri(b,i.TEXTURE_WRAP_R,vt[g.wrapR]),i.texParameteri(b,i.TEXTURE_MAG_FILTER,Gt[g.magFilter]),i.texParameteri(b,i.TEXTURE_MIN_FILTER,Gt[g.minFilter]),g.compareFunction&&(i.texParameteri(b,i.TEXTURE_COMPARE_MODE,i.COMPARE_REF_TO_TEXTURE),i.texParameteri(b,i.TEXTURE_COMPARE_FUNC,Qt[g.compareFunction])),t.has("EXT_texture_filter_anisotropic")===!0){if(g.magFilter===Re||g.minFilter!==cr&&g.minFilter!==xi||g.type===bn&&t.has("OES_texture_float_linear")===!1)return;if(g.anisotropy>1||n.get(g).__currentAnisotropy){let F=t.get("EXT_texture_filter_anisotropic");i.texParameterf(b,F.TEXTURE_MAX_ANISOTROPY_EXT,Math.min(g.anisotropy,s.getMaxAnisotropy())),n.get(g).__currentAnisotropy=g.anisotropy}}}function J(b,g){let F=!1;b.__webglInit===void 0&&(b.__webglInit=!0,g.addEventListener("dispose",I));let q=g.source,Q=p.get(q);Q===void 0&&(Q={},p.set(q,Q));let nt=B(g);if(nt!==b.__cacheKey){Q[nt]===void 0&&(Q[nt]={texture:i.createTexture(),usedTimes:0},a.memory.textures++,F=!0),Q[nt].usedTimes++;let lt=Q[b.__cacheKey];lt!==void 0&&(Q[b.__cacheKey].usedTimes--,lt.usedTimes===0&&D(g)),b.__cacheKey=nt,b.__webglTexture=Q[nt].texture}return F}function pt(b,g,F){return Math.floor(Math.floor(b/F)/g)}function rt(b,g,F,q){let nt=b.updateRanges;if(nt.length===0)e.texSubImage2D(i.TEXTURE_2D,0,0,0,g.width,g.height,F,q,g.data);else{nt.sort((xt,at)=>xt.start-at.start);let lt=0;for(let xt=1;xt<nt.length;xt++){let at=nt[lt],it=nt[xt],It=at.start+at.count,Ft=pt(it.start,g.width,4),Jt=pt(at.start,g.width,4);it.start<=It+1&&Ft===Jt&&pt(it.start+it.count-1,g.width,4)===Ft?at.count=Math.max(at.count,it.start+it.count-at.start):(++lt,nt[lt]=it)}nt.length=lt+1;let W=e.getParameter(i.UNPACK_ROW_LENGTH),Y=e.getParameter(i.UNPACK_SKIP_PIXELS),mt=e.getParameter(i.UNPACK_SKIP_ROWS);e.pixelStorei(i.UNPACK_ROW_LENGTH,g.width);for(let xt=0,at=nt.length;xt<at;xt++){let it=nt[xt],It=Math.floor(it.start/4),Ft=Math.ceil(it.count/4),Jt=It%g.width,C=Math.floor(It/g.width),st=Ft,X=1;e.pixelStorei(i.UNPACK_SKIP_PIXELS,Jt),e.pixelStorei(i.UNPACK_SKIP_ROWS,C),e.texSubImage2D(i.TEXTURE_2D,0,Jt,C,st,X,F,q,g.data)}b.clearUpdateRanges(),e.pixelStorei(i.UNPACK_ROW_LENGTH,W),e.pixelStorei(i.UNPACK_SKIP_PIXELS,Y),e.pixelStorei(i.UNPACK_SKIP_ROWS,mt)}}function Tt(b,g,F){let q=i.TEXTURE_2D;(g.isDataArrayTexture||g.isCompressedArrayTexture)&&(q=i.TEXTURE_2D_ARRAY),g.isData3DTexture&&(q=i.TEXTURE_3D);let Q=J(b,g),nt=g.source;e.bindTexture(q,b.__webglTexture,i.TEXTURE0+F);let lt=n.get(nt);if(nt.version!==lt.__version||Q===!0){if(e.activeTexture(i.TEXTURE0+F),(typeof ImageBitmap<"u"&&g.image instanceof ImageBitmap)===!1){let X=$t.getPrimaries($t.workingColorSpace),gt=g.colorSpace===Qn?null:$t.getPrimaries(g.colorSpace),ot=g.colorSpace===Qn||X===gt?i.NONE:i.BROWSER_DEFAULT_WEBGL;e.pixelStorei(i.UNPACK_FLIP_Y_WEBGL,g.flipY),e.pixelStorei(i.UNPACK_PREMULTIPLY_ALPHA_WEBGL,g.premultiplyAlpha),e.pixelStorei(i.UNPACK_COLORSPACE_CONVERSION_WEBGL,ot)}e.pixelStorei(i.UNPACK_ALIGNMENT,g.unpackAlignment);let Y=m(g.image,!1,s.maxTextureSize);Y=ht(g,Y);let mt=r.convert(g.format,g.colorSpace),xt=r.convert(g.type),at=E(g.internalFormat,mt,xt,g.normalized,g.colorSpace,g.isVideoTexture);Ut(q,g);let it,It=g.mipmaps,Ft=g.isVideoTexture!==!0,Jt=lt.__version===void 0||Q===!0,C=nt.dataReady,st=w(g,Y);if(g.isDepthTexture)at=R(g.format===yi,g.type),Jt&&(Ft?e.texStorage2D(i.TEXTURE_2D,1,at,Y.width,Y.height):e.texImage2D(i.TEXTURE_2D,0,at,Y.width,Y.height,0,mt,xt,null));else if(g.isDataTexture)if(It.length>0){Ft&&Jt&&e.texStorage2D(i.TEXTURE_2D,st,at,It[0].width,It[0].height);for(let X=0,gt=It.length;X<gt;X++)it=It[X],Ft?C&&e.texSubImage2D(i.TEXTURE_2D,X,0,0,it.width,it.height,mt,xt,it.data):e.texImage2D(i.TEXTURE_2D,X,at,it.width,it.height,0,mt,xt,it.data);g.generateMipmaps=!1}else Ft?(Jt&&e.texStorage2D(i.TEXTURE_2D,st,at,Y.width,Y.height),C&&rt(g,Y,mt,xt)):e.texImage2D(i.TEXTURE_2D,0,at,Y.width,Y.height,0,mt,xt,Y.data);else if(g.isCompressedTexture)if(g.isCompressedArrayTexture){Ft&&Jt&&e.texStorage3D(i.TEXTURE_2D_ARRAY,st,at,It[0].width,It[0].height,Y.depth);for(let X=0,gt=It.length;X<gt;X++)if(it=It[X],g.format!==hn)if(mt!==null)if(Ft){if(C)if(g.layerUpdates.size>0){let ot=pc(it.width,it.height,g.format,g.type);for(let j of g.layerUpdates){let Mt=it.data.subarray(j*ot/it.data.BYTES_PER_ELEMENT,(j+1)*ot/it.data.BYTES_PER_ELEMENT);e.compressedTexSubImage3D(i.TEXTURE_2D_ARRAY,X,0,0,j,it.width,it.height,1,mt,Mt)}g.clearLayerUpdates()}else e.compressedTexSubImage3D(i.TEXTURE_2D_ARRAY,X,0,0,0,it.width,it.height,Y.depth,mt,it.data)}else e.compressedTexImage3D(i.TEXTURE_2D_ARRAY,X,at,it.width,it.height,Y.depth,0,it.data,0,0);else Et("WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()");else Ft?C&&e.texSubImage3D(i.TEXTURE_2D_ARRAY,X,0,0,0,it.width,it.height,Y.depth,mt,xt,it.data):e.texImage3D(i.TEXTURE_2D_ARRAY,X,at,it.width,it.height,Y.depth,0,mt,xt,it.data)}else{Ft&&Jt&&e.texStorage2D(i.TEXTURE_2D,st,at,It[0].width,It[0].height);for(let X=0,gt=It.length;X<gt;X++)it=It[X],g.format!==hn?mt!==null?Ft?C&&e.compressedTexSubImage2D(i.TEXTURE_2D,X,0,0,it.width,it.height,mt,it.data):e.compressedTexImage2D(i.TEXTURE_2D,X,at,it.width,it.height,0,it.data):Et("WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()"):Ft?C&&e.texSubImage2D(i.TEXTURE_2D,X,0,0,it.width,it.height,mt,xt,it.data):e.texImage2D(i.TEXTURE_2D,X,at,it.width,it.height,0,mt,xt,it.data)}else if(g.isDataArrayTexture)if(Ft){if(Jt&&e.texStorage3D(i.TEXTURE_2D_ARRAY,st,at,Y.width,Y.height,Y.depth),C)if(g.layerUpdates.size>0){let X=pc(Y.width,Y.height,g.format,g.type);for(let gt of g.layerUpdates){let ot=Y.data.subarray(gt*X/Y.data.BYTES_PER_ELEMENT,(gt+1)*X/Y.data.BYTES_PER_ELEMENT);e.texSubImage3D(i.TEXTURE_2D_ARRAY,0,0,0,gt,Y.width,Y.height,1,mt,xt,ot)}g.clearLayerUpdates()}else e.texSubImage3D(i.TEXTURE_2D_ARRAY,0,0,0,0,Y.width,Y.height,Y.depth,mt,xt,Y.data)}else e.texImage3D(i.TEXTURE_2D_ARRAY,0,at,Y.width,Y.height,Y.depth,0,mt,xt,Y.data);else if(g.isData3DTexture)Ft?(Jt&&e.texStorage3D(i.TEXTURE_3D,st,at,Y.width,Y.height,Y.depth),C&&e.texSubImage3D(i.TEXTURE_3D,0,0,0,0,Y.width,Y.height,Y.depth,mt,xt,Y.data)):e.texImage3D(i.TEXTURE_3D,0,at,Y.width,Y.height,Y.depth,0,mt,xt,Y.data);else if(g.isFramebufferTexture){if(Jt)if(Ft)e.texStorage2D(i.TEXTURE_2D,st,at,Y.width,Y.height);else{let X=Y.width,gt=Y.height;for(let ot=0;ot<st;ot++)e.texImage2D(i.TEXTURE_2D,ot,at,X,gt,0,mt,xt,null),X>>=1,gt>>=1}}else if(g.isHTMLTexture){if("texElementImage2D"in i){let X=i.canvas;if(X.hasAttribute("layoutsubtree")||X.setAttribute("layoutsubtree","true"),Y.parentNode!==X){X.appendChild(Y),u.add(g),X.onpaint=Dt=>{let _e=Dt.changedElements;for(let ne of u)_e.includes(ne.image)&&(ne.needsUpdate=!0)},X.requestPaint();return}let gt=0,ot=i.RGBA,j=i.RGBA,Mt=i.UNSIGNED_BYTE;i.texElementImage2D(i.TEXTURE_2D,gt,ot,j,Mt,Y),i.texParameteri(i.TEXTURE_2D,i.TEXTURE_MIN_FILTER,i.LINEAR),i.texParameteri(i.TEXTURE_2D,i.TEXTURE_WRAP_S,i.CLAMP_TO_EDGE),i.texParameteri(i.TEXTURE_2D,i.TEXTURE_WRAP_T,i.CLAMP_TO_EDGE)}}else if(It.length>0){if(Ft&&Jt){let X=me(It[0]);e.texStorage2D(i.TEXTURE_2D,st,at,X.width,X.height)}for(let X=0,gt=It.length;X<gt;X++)it=It[X],Ft?C&&e.texSubImage2D(i.TEXTURE_2D,X,0,0,mt,xt,it):e.texImage2D(i.TEXTURE_2D,X,at,mt,xt,it);g.generateMipmaps=!1}else if(Ft){if(Jt){let X=me(Y);e.texStorage2D(i.TEXTURE_2D,st,at,X.width,X.height)}C&&e.texSubImage2D(i.TEXTURE_2D,0,0,0,mt,xt,Y)}else e.texImage2D(i.TEXTURE_2D,0,at,mt,xt,Y);f(g)&&S(q),lt.__version=nt.version,g.onUpdate&&g.onUpdate(g)}b.__version=g.version}function Lt(b,g,F){if(g.image.length!==6)return;let q=J(b,g),Q=g.source;e.bindTexture(i.TEXTURE_CUBE_MAP,b.__webglTexture,i.TEXTURE0+F);let nt=n.get(Q);if(Q.version!==nt.__version||q===!0){e.activeTexture(i.TEXTURE0+F);let lt=$t.getPrimaries($t.workingColorSpace),W=g.colorSpace===Qn?null:$t.getPrimaries(g.colorSpace),Y=g.colorSpace===Qn||lt===W?i.NONE:i.BROWSER_DEFAULT_WEBGL;e.pixelStorei(i.UNPACK_FLIP_Y_WEBGL,g.flipY),e.pixelStorei(i.UNPACK_PREMULTIPLY_ALPHA_WEBGL,g.premultiplyAlpha),e.pixelStorei(i.UNPACK_ALIGNMENT,g.unpackAlignment),e.pixelStorei(i.UNPACK_COLORSPACE_CONVERSION_WEBGL,Y);let mt=g.isCompressedTexture||g.image[0].isCompressedTexture,xt=g.image[0]&&g.image[0].isDataTexture,at=[];for(let j=0;j<6;j++)!mt&&!xt?at[j]=m(g.image[j],!0,s.maxCubemapSize):at[j]=xt?g.image[j].image:g.image[j],at[j]=ht(g,at[j]);let it=at[0],It=r.convert(g.format,g.colorSpace),Ft=r.convert(g.type),Jt=E(g.internalFormat,It,Ft,g.normalized,g.colorSpace),C=g.isVideoTexture!==!0,st=nt.__version===void 0||q===!0,X=Q.dataReady,gt=w(g,it);Ut(i.TEXTURE_CUBE_MAP,g);let ot;if(mt){C&&st&&e.texStorage2D(i.TEXTURE_CUBE_MAP,gt,Jt,it.width,it.height);for(let j=0;j<6;j++){ot=at[j].mipmaps;for(let Mt=0;Mt<ot.length;Mt++){let Dt=ot[Mt];g.format!==hn?It!==null?C?X&&e.compressedTexSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt,0,0,Dt.width,Dt.height,It,Dt.data):e.compressedTexImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt,Jt,Dt.width,Dt.height,0,Dt.data):Et("WebGLRenderer: Attempt to load unsupported compressed texture format in .setTextureCube()"):C?X&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt,0,0,Dt.width,Dt.height,It,Ft,Dt.data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt,Jt,Dt.width,Dt.height,0,It,Ft,Dt.data)}}}else{if(ot=g.mipmaps,C&&st){ot.length>0&&gt++;let j=me(at[0]);e.texStorage2D(i.TEXTURE_CUBE_MAP,gt,Jt,j.width,j.height)}for(let j=0;j<6;j++)if(xt){C?X&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,0,0,0,at[j].width,at[j].height,It,Ft,at[j].data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,0,Jt,at[j].width,at[j].height,0,It,Ft,at[j].data);for(let Mt=0;Mt<ot.length;Mt++){let _e=ot[Mt].image[j].image;C?X&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt+1,0,0,_e.width,_e.height,It,Ft,_e.data):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt+1,Jt,_e.width,_e.height,0,It,Ft,_e.data)}}else{C?X&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,0,0,0,It,Ft,at[j]):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,0,Jt,It,Ft,at[j]);for(let Mt=0;Mt<ot.length;Mt++){let Dt=ot[Mt];C?X&&e.texSubImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt+1,0,0,It,Ft,Dt.image[j]):e.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+j,Mt+1,Jt,It,Ft,Dt.image[j])}}}f(g)&&S(i.TEXTURE_CUBE_MAP),nt.__version=Q.version,g.onUpdate&&g.onUpdate(g)}b.__version=g.version}function Rt(b,g,F,q,Q,nt){let lt=r.convert(F.format,F.colorSpace),W=r.convert(F.type),Y=E(F.internalFormat,lt,W,F.normalized,F.colorSpace),mt=n.get(g),xt=n.get(F);if(xt.__renderTarget=g,!mt.__hasExternalTextures){let at=Math.max(1,g.width>>nt),it=Math.max(1,g.height>>nt);Q===i.TEXTURE_3D||Q===i.TEXTURE_2D_ARRAY?e.texImage3D(Q,nt,Y,at,it,g.depth,0,lt,W,null):e.texImage2D(Q,nt,Y,at,it,0,lt,W,null)}e.bindFramebuffer(i.FRAMEBUFFER,b),Wt(g)?o.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,q,Q,xt.__webglTexture,0,we(g)):(Q===i.TEXTURE_2D||Q>=i.TEXTURE_CUBE_MAP_POSITIVE_X&&Q<=i.TEXTURE_CUBE_MAP_NEGATIVE_Z)&&i.framebufferTexture2D(i.FRAMEBUFFER,q,Q,xt.__webglTexture,nt),e.bindFramebuffer(i.FRAMEBUFFER,null)}function de(b,g,F){if(i.bindRenderbuffer(i.RENDERBUFFER,b),g.depthBuffer){let q=g.depthTexture,Q=q&&q.isDepthTexture?q.type:null,nt=R(g.stencilBuffer,Q),lt=g.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT;Wt(g)?o.renderbufferStorageMultisampleEXT(i.RENDERBUFFER,we(g),nt,g.width,g.height):F?i.renderbufferStorageMultisample(i.RENDERBUFFER,we(g),nt,g.width,g.height):i.renderbufferStorage(i.RENDERBUFFER,nt,g.width,g.height),i.framebufferRenderbuffer(i.FRAMEBUFFER,lt,i.RENDERBUFFER,b)}else{let q=g.textures;for(let Q=0;Q<q.length;Q++){let nt=q[Q],lt=r.convert(nt.format,nt.colorSpace),W=r.convert(nt.type),Y=E(nt.internalFormat,lt,W,nt.normalized,nt.colorSpace);Wt(g)?o.renderbufferStorageMultisampleEXT(i.RENDERBUFFER,we(g),Y,g.width,g.height):F?i.renderbufferStorageMultisample(i.RENDERBUFFER,we(g),Y,g.width,g.height):i.renderbufferStorage(i.RENDERBUFFER,Y,g.width,g.height)}}i.bindRenderbuffer(i.RENDERBUFFER,null)}function Ht(b,g,F){let q=g.isWebGLCubeRenderTarget===!0;if(e.bindFramebuffer(i.FRAMEBUFFER,b),!(g.depthTexture&&g.depthTexture.isDepthTexture))throw new Error("renderTarget.depthTexture must be an instance of THREE.DepthTexture");let Q=n.get(g.depthTexture);if(Q.__renderTarget=g,(!Q.__webglTexture||g.depthTexture.image.width!==g.width||g.depthTexture.image.height!==g.height)&&(g.depthTexture.image.width=g.width,g.depthTexture.image.height=g.height,g.depthTexture.needsUpdate=!0),q){if(Q.__webglInit===void 0&&(Q.__webglInit=!0,g.depthTexture.addEventListener("dispose",I)),Q.__webglTexture===void 0){Q.__webglTexture=i.createTexture(),e.bindTexture(i.TEXTURE_CUBE_MAP,Q.__webglTexture),Ut(i.TEXTURE_CUBE_MAP,g.depthTexture);let mt=r.convert(g.depthTexture.format),xt=r.convert(g.depthTexture.type),at;g.depthTexture.format===Ln?at=i.DEPTH_COMPONENT24:g.depthTexture.format===yi&&(at=i.DEPTH24_STENCIL8);for(let it=0;it<6;it++)i.texImage2D(i.TEXTURE_CUBE_MAP_POSITIVE_X+it,0,at,g.width,g.height,0,mt,xt,null)}}else K(g.depthTexture,0);let nt=Q.__webglTexture,lt=we(g),W=q?i.TEXTURE_CUBE_MAP_POSITIVE_X+F:i.TEXTURE_2D,Y=g.depthTexture.format===yi?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT;if(g.depthTexture.format===Ln)Wt(g)?o.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,Y,W,nt,0,lt):i.framebufferTexture2D(i.FRAMEBUFFER,Y,W,nt,0);else if(g.depthTexture.format===yi)Wt(g)?o.framebufferTexture2DMultisampleEXT(i.FRAMEBUFFER,Y,W,nt,0,lt):i.framebufferTexture2D(i.FRAMEBUFFER,Y,W,nt,0);else throw new Error("Unknown depthTexture format")}function ee(b){let g=n.get(b),F=b.isWebGLCubeRenderTarget===!0;if(g.__boundDepthTexture!==b.depthTexture){let q=b.depthTexture;if(g.__depthDisposeCallback&&g.__depthDisposeCallback(),q){let Q=()=>{delete g.__boundDepthTexture,delete g.__depthDisposeCallback,q.removeEventListener("dispose",Q)};q.addEventListener("dispose",Q),g.__depthDisposeCallback=Q}g.__boundDepthTexture=q}if(b.depthTexture&&!g.__autoAllocateDepthBuffer)if(F)for(let q=0;q<6;q++)Ht(g.__webglFramebuffer[q],b,q);else{let q=b.texture.mipmaps;q&&q.length>0?Ht(g.__webglFramebuffer[0],b,0):Ht(g.__webglFramebuffer,b,0)}else if(F){g.__webglDepthbuffer=[];for(let q=0;q<6;q++)if(e.bindFramebuffer(i.FRAMEBUFFER,g.__webglFramebuffer[q]),g.__webglDepthbuffer[q]===void 0)g.__webglDepthbuffer[q]=i.createRenderbuffer(),de(g.__webglDepthbuffer[q],b,!1);else{let Q=b.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,nt=g.__webglDepthbuffer[q];i.bindRenderbuffer(i.RENDERBUFFER,nt),i.framebufferRenderbuffer(i.FRAMEBUFFER,Q,i.RENDERBUFFER,nt)}}else{let q=b.texture.mipmaps;if(q&&q.length>0?e.bindFramebuffer(i.FRAMEBUFFER,g.__webglFramebuffer[0]):e.bindFramebuffer(i.FRAMEBUFFER,g.__webglFramebuffer),g.__webglDepthbuffer===void 0)g.__webglDepthbuffer=i.createRenderbuffer(),de(g.__webglDepthbuffer,b,!1);else{let Q=b.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,nt=g.__webglDepthbuffer;i.bindRenderbuffer(i.RENDERBUFFER,nt),i.framebufferRenderbuffer(i.FRAMEBUFFER,Q,i.RENDERBUFFER,nt)}}e.bindFramebuffer(i.FRAMEBUFFER,null)}function ce(b,g,F){let q=n.get(b);g!==void 0&&Rt(q.__webglFramebuffer,b,b.texture,i.COLOR_ATTACHMENT0,i.TEXTURE_2D,0),F!==void 0&&ee(b)}function Vt(b){let g=b.texture,F=n.get(b),q=n.get(g);b.addEventListener("dispose",v);let Q=b.textures,nt=b.isWebGLCubeRenderTarget===!0,lt=Q.length>1;if(lt||(q.__webglTexture===void 0&&(q.__webglTexture=i.createTexture()),q.__version=g.version,a.memory.textures++),nt){F.__webglFramebuffer=[];for(let W=0;W<6;W++)if(g.mipmaps&&g.mipmaps.length>0){F.__webglFramebuffer[W]=[];for(let Y=0;Y<g.mipmaps.length;Y++)F.__webglFramebuffer[W][Y]=i.createFramebuffer()}else F.__webglFramebuffer[W]=i.createFramebuffer()}else{if(g.mipmaps&&g.mipmaps.length>0){F.__webglFramebuffer=[];for(let W=0;W<g.mipmaps.length;W++)F.__webglFramebuffer[W]=i.createFramebuffer()}else F.__webglFramebuffer=i.createFramebuffer();if(lt)for(let W=0,Y=Q.length;W<Y;W++){let mt=n.get(Q[W]);mt.__webglTexture===void 0&&(mt.__webglTexture=i.createTexture(),a.memory.textures++)}if(b.samples>0&&Wt(b)===!1){F.__webglMultisampledFramebuffer=i.createFramebuffer(),F.__webglColorRenderbuffer=[],e.bindFramebuffer(i.FRAMEBUFFER,F.__webglMultisampledFramebuffer);for(let W=0;W<Q.length;W++){let Y=Q[W];F.__webglColorRenderbuffer[W]=i.createRenderbuffer(),i.bindRenderbuffer(i.RENDERBUFFER,F.__webglColorRenderbuffer[W]);let mt=r.convert(Y.format,Y.colorSpace),xt=r.convert(Y.type),at=E(Y.internalFormat,mt,xt,Y.normalized,Y.colorSpace,b.isXRRenderTarget===!0),it=we(b);i.renderbufferStorageMultisample(i.RENDERBUFFER,it,at,b.width,b.height),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+W,i.RENDERBUFFER,F.__webglColorRenderbuffer[W])}i.bindRenderbuffer(i.RENDERBUFFER,null),b.depthBuffer&&(F.__webglDepthRenderbuffer=i.createRenderbuffer(),de(F.__webglDepthRenderbuffer,b,!0)),e.bindFramebuffer(i.FRAMEBUFFER,null)}}if(nt){e.bindTexture(i.TEXTURE_CUBE_MAP,q.__webglTexture),Ut(i.TEXTURE_CUBE_MAP,g);for(let W=0;W<6;W++)if(g.mipmaps&&g.mipmaps.length>0)for(let Y=0;Y<g.mipmaps.length;Y++)Rt(F.__webglFramebuffer[W][Y],b,g,i.COLOR_ATTACHMENT0,i.TEXTURE_CUBE_MAP_POSITIVE_X+W,Y);else Rt(F.__webglFramebuffer[W],b,g,i.COLOR_ATTACHMENT0,i.TEXTURE_CUBE_MAP_POSITIVE_X+W,0);f(g)&&S(i.TEXTURE_CUBE_MAP),e.unbindTexture()}else if(lt){for(let W=0,Y=Q.length;W<Y;W++){let mt=Q[W],xt=n.get(mt),at=i.TEXTURE_2D;(b.isWebGL3DRenderTarget||b.isWebGLArrayRenderTarget)&&(at=b.isWebGL3DRenderTarget?i.TEXTURE_3D:i.TEXTURE_2D_ARRAY),e.bindTexture(at,xt.__webglTexture),Ut(at,mt),Rt(F.__webglFramebuffer,b,mt,i.COLOR_ATTACHMENT0+W,at,0),f(mt)&&S(at)}e.unbindTexture()}else{let W=i.TEXTURE_2D;if((b.isWebGL3DRenderTarget||b.isWebGLArrayRenderTarget)&&(W=b.isWebGL3DRenderTarget?i.TEXTURE_3D:i.TEXTURE_2D_ARRAY),e.bindTexture(W,q.__webglTexture),Ut(W,g),g.mipmaps&&g.mipmaps.length>0)for(let Y=0;Y<g.mipmaps.length;Y++)Rt(F.__webglFramebuffer[Y],b,g,i.COLOR_ATTACHMENT0,W,Y);else Rt(F.__webglFramebuffer,b,g,i.COLOR_ATTACHMENT0,W,0);f(g)&&S(W),e.unbindTexture()}b.depthBuffer&&ee(b)}function Me(b){let g=b.textures;for(let F=0,q=g.length;F<q;F++){let Q=g[F];if(f(Q)){let nt=M(b),lt=n.get(Q).__webglTexture;e.bindTexture(nt,lt),S(nt),e.unbindTexture()}}}let ue=[],We=[];function P(b){if(b.samples>0){if(Wt(b)===!1){let g=b.textures,F=b.width,q=b.height,Q=i.COLOR_BUFFER_BIT,nt=b.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT,lt=n.get(b),W=g.length>1;if(W)for(let mt=0;mt<g.length;mt++)e.bindFramebuffer(i.FRAMEBUFFER,lt.__webglMultisampledFramebuffer),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+mt,i.RENDERBUFFER,null),e.bindFramebuffer(i.FRAMEBUFFER,lt.__webglFramebuffer),i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0+mt,i.TEXTURE_2D,null,0);e.bindFramebuffer(i.READ_FRAMEBUFFER,lt.__webglMultisampledFramebuffer);let Y=b.texture.mipmaps;Y&&Y.length>0?e.bindFramebuffer(i.DRAW_FRAMEBUFFER,lt.__webglFramebuffer[0]):e.bindFramebuffer(i.DRAW_FRAMEBUFFER,lt.__webglFramebuffer);for(let mt=0;mt<g.length;mt++){if(b.resolveDepthBuffer&&(b.depthBuffer&&(Q|=i.DEPTH_BUFFER_BIT),b.stencilBuffer&&b.resolveStencilBuffer&&(Q|=i.STENCIL_BUFFER_BIT)),W){i.framebufferRenderbuffer(i.READ_FRAMEBUFFER,i.COLOR_ATTACHMENT0,i.RENDERBUFFER,lt.__webglColorRenderbuffer[mt]);let xt=n.get(g[mt]).__webglTexture;i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0,i.TEXTURE_2D,xt,0)}i.blitFramebuffer(0,0,F,q,0,0,F,q,Q,i.NEAREST),l===!0&&(ue.length=0,We.length=0,ue.push(i.COLOR_ATTACHMENT0+mt),b.depthBuffer&&b.resolveDepthBuffer===!1&&(ue.push(nt),We.push(nt),i.invalidateFramebuffer(i.DRAW_FRAMEBUFFER,We)),i.invalidateFramebuffer(i.READ_FRAMEBUFFER,ue))}if(e.bindFramebuffer(i.READ_FRAMEBUFFER,null),e.bindFramebuffer(i.DRAW_FRAMEBUFFER,null),W)for(let mt=0;mt<g.length;mt++){e.bindFramebuffer(i.FRAMEBUFFER,lt.__webglMultisampledFramebuffer),i.framebufferRenderbuffer(i.FRAMEBUFFER,i.COLOR_ATTACHMENT0+mt,i.RENDERBUFFER,lt.__webglColorRenderbuffer[mt]);let xt=n.get(g[mt]).__webglTexture;e.bindFramebuffer(i.FRAMEBUFFER,lt.__webglFramebuffer),i.framebufferTexture2D(i.DRAW_FRAMEBUFFER,i.COLOR_ATTACHMENT0+mt,i.TEXTURE_2D,xt,0)}e.bindFramebuffer(i.DRAW_FRAMEBUFFER,lt.__webglMultisampledFramebuffer)}else if(b.depthBuffer&&b.resolveDepthBuffer===!1&&l){let g=b.stencilBuffer?i.DEPTH_STENCIL_ATTACHMENT:i.DEPTH_ATTACHMENT;i.invalidateFramebuffer(i.DRAW_FRAMEBUFFER,[g])}}}function we(b){return Math.min(s.maxSamples,b.samples)}function Wt(b){let g=n.get(b);return b.samples>0&&t.has("WEBGL_multisampled_render_to_texture")===!0&&g.__useRenderToTexture!==!1}function ae(b){let g=a.render.frame;d.get(b)!==g&&(d.set(b,g),b.update())}function ht(b,g){let F=b.colorSpace,q=b.format,Q=b.type;return b.isCompressedTexture===!0||b.isVideoTexture===!0||F!==Gs&&F!==Qn&&($t.getTransfer(F)===te?(q!==hn||Q!==He)&&Et("WebGLTextures: sRGB encoded textures have to use RGBAFormat and UnsignedByteType."):Ct("WebGLTextures: Unsupported texture color space:",F)),g}function me(b){return typeof HTMLImageElement<"u"&&b instanceof HTMLImageElement?(c.width=b.naturalWidth||b.width,c.height=b.naturalHeight||b.height):typeof VideoFrame<"u"&&b instanceof VideoFrame?(c.width=b.displayWidth,c.height=b.displayHeight):(c.width=b.width,c.height=b.height),c}this.allocateTextureUnit=L,this.resetTextureUnits=H,this.getTextureUnits=$,this.setTextureUnits=U,this.setTexture2D=K,this.setTexture2DArray=tt,this.setTexture3D=ct,this.setTextureCube=bt,this.rebindTextures=ce,this.setupRenderTarget=Vt,this.updateRenderTargetMipmap=Me,this.updateMultisampleRenderTarget=P,this.setupDepthRenderbuffer=ee,this.setupFrameBufferTexture=Rt,this.useMultisampledRTT=Wt,this.isReversedDepthBuffer=function(){return e.buffers.depth.getReversed()}}function D_(i,t){function e(n,s=Qn){let r,a=$t.getTransfer(s);if(n===He)return i.UNSIGNED_BYTE;if(n===Xa)return i.UNSIGNED_SHORT_4_4_4_4;if(n===qa)return i.UNSIGNED_SHORT_5_5_5_1;if(n===sc)return i.UNSIGNED_INT_5_9_9_9_REV;if(n===rc)return i.UNSIGNED_INT_10F_11F_11F_REV;if(n===nc)return i.BYTE;if(n===ic)return i.SHORT;if(n===xs)return i.UNSIGNED_SHORT;if(n===$a)return i.INT;if(n===yn)return i.UNSIGNED_INT;if(n===bn)return i.FLOAT;if(n===On)return i.HALF_FLOAT;if(n===ac)return i.ALPHA;if(n===oc)return i.RGB;if(n===hn)return i.RGBA;if(n===Ln)return i.DEPTH_COMPONENT;if(n===yi)return i.DEPTH_STENCIL;if(n===lc)return i.RED;if(n===Ya)return i.RED_INTEGER;if(n===bi)return i.RG;if(n===Za)return i.RG_INTEGER;if(n===Ja)return i.RGBA_INTEGER;if(n===hr||n===dr||n===ur||n===fr)if(a===te)if(r=t.get("WEBGL_compressed_texture_s3tc_srgb"),r!==null){if(n===hr)return r.COMPRESSED_SRGB_S3TC_DXT1_EXT;if(n===dr)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT;if(n===ur)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT;if(n===fr)return r.COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT}else return null;else if(r=t.get("WEBGL_compressed_texture_s3tc"),r!==null){if(n===hr)return r.COMPRESSED_RGB_S3TC_DXT1_EXT;if(n===dr)return r.COMPRESSED_RGBA_S3TC_DXT1_EXT;if(n===ur)return r.COMPRESSED_RGBA_S3TC_DXT3_EXT;if(n===fr)return r.COMPRESSED_RGBA_S3TC_DXT5_EXT}else return null;if(n===Ka||n===ja||n===Qa||n===to)if(r=t.get("WEBGL_compressed_texture_pvrtc"),r!==null){if(n===Ka)return r.COMPRESSED_RGB_PVRTC_4BPPV1_IMG;if(n===ja)return r.COMPRESSED_RGB_PVRTC_2BPPV1_IMG;if(n===Qa)return r.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG;if(n===to)return r.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG}else return null;if(n===eo||n===no||n===io||n===so||n===ro||n===pr||n===ao)if(r=t.get("WEBGL_compressed_texture_etc"),r!==null){if(n===eo||n===no)return a===te?r.COMPRESSED_SRGB8_ETC2:r.COMPRESSED_RGB8_ETC2;if(n===io)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC:r.COMPRESSED_RGBA8_ETC2_EAC;if(n===so)return r.COMPRESSED_R11_EAC;if(n===ro)return r.COMPRESSED_SIGNED_R11_EAC;if(n===pr)return r.COMPRESSED_RG11_EAC;if(n===ao)return r.COMPRESSED_SIGNED_RG11_EAC}else return null;if(n===oo||n===lo||n===co||n===ho||n===uo||n===fo||n===po||n===mo||n===go||n===_o||n===vo||n===xo||n===yo||n===bo)if(r=t.get("WEBGL_compressed_texture_astc"),r!==null){if(n===oo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR:r.COMPRESSED_RGBA_ASTC_4x4_KHR;if(n===lo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR:r.COMPRESSED_RGBA_ASTC_5x4_KHR;if(n===co)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR:r.COMPRESSED_RGBA_ASTC_5x5_KHR;if(n===ho)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR:r.COMPRESSED_RGBA_ASTC_6x5_KHR;if(n===uo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR:r.COMPRESSED_RGBA_ASTC_6x6_KHR;if(n===fo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR:r.COMPRESSED_RGBA_ASTC_8x5_KHR;if(n===po)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR:r.COMPRESSED_RGBA_ASTC_8x6_KHR;if(n===mo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR:r.COMPRESSED_RGBA_ASTC_8x8_KHR;if(n===go)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR:r.COMPRESSED_RGBA_ASTC_10x5_KHR;if(n===_o)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR:r.COMPRESSED_RGBA_ASTC_10x6_KHR;if(n===vo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR:r.COMPRESSED_RGBA_ASTC_10x8_KHR;if(n===xo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR:r.COMPRESSED_RGBA_ASTC_10x10_KHR;if(n===yo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR:r.COMPRESSED_RGBA_ASTC_12x10_KHR;if(n===bo)return a===te?r.COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR:r.COMPRESSED_RGBA_ASTC_12x12_KHR}else return null;if(n===So||n===Mo||n===wo)if(r=t.get("EXT_texture_compression_bptc"),r!==null){if(n===So)return a===te?r.COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT:r.COMPRESSED_RGBA_BPTC_UNORM_EXT;if(n===Mo)return r.COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT;if(n===wo)return r.COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT}else return null;if(n===Ao||n===Eo||n===mr||n===To)if(r=t.get("EXT_texture_compression_rgtc"),r!==null){if(n===Ao)return r.COMPRESSED_RED_RGTC1_EXT;if(n===Eo)return r.COMPRESSED_SIGNED_RED_RGTC1_EXT;if(n===mr)return r.COMPRESSED_RED_GREEN_RGTC2_EXT;if(n===To)return r.COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT}else return null;return n===ys?i.UNSIGNED_INT_24_8:i[n]!==void 0?i[n]:null}return{convert:e}}var N_=`
void main() {

	gl_Position = vec4( position, 1.0 );

}`,U_=`
uniform sampler2DArray depthColor;
uniform float depthWidth;
uniform float depthHeight;

void main() {

	vec2 coord = vec2( gl_FragCoord.x / depthWidth, gl_FragCoord.y / depthHeight );

	if ( coord.x >= 1.0 ) {

		gl_FragDepth = texture( depthColor, vec3( coord.x - 1.0, coord.y, 1 ) ).r;

	} else {

		gl_FragDepth = texture( depthColor, vec3( coord.x, coord.y, 0 ) ).r;

	}

}`,Cc=class{constructor(){this.texture=null,this.mesh=null,this.depthNear=0,this.depthFar=0}init(t,e){if(this.texture===null){let n=new tr(t.texture);(t.depthNear!==e.depthNear||t.depthFar!==e.depthFar)&&(this.depthNear=t.depthNear,this.depthFar=t.depthFar),this.texture=n}}getMesh(t){if(this.texture!==null&&this.mesh===null){let e=t.cameras[0].viewport,n=new je({vertexShader:N_,fragmentShader:U_,uniforms:{depthColor:{value:this.texture},depthWidth:{value:e.z},depthHeight:{value:e.w}}});this.mesh=new ke(new Gi(20,20),n)}return this.mesh}reset(){this.texture=null,this.mesh=null}getDepthTexture(){return this.texture}},Rc=class extends Dn{constructor(t,e){super();let n=this,s=null,r=1,a=null,o="local-floor",l=1,c=null,d=null,u=null,h=null,p=null,_=null,y=typeof XRWebGLBinding<"u",m=new Cc,f={},S=e.getContextAttributes(),M=null,E=null,R=[],w=[],I=new jt,v=null,A=new Ue;A.viewport=new pe;let D=new Ue;D.viewport=new pe;let T=[A,D],O=new Va,H=null,$=null;this.cameraAutoUpdate=!0,this.enabled=!1,this.isPresenting=!1,this.getController=function(J){let pt=R[J];return pt===void 0&&(pt=new ps,R[J]=pt),pt.getTargetRaySpace()},this.getControllerGrip=function(J){let pt=R[J];return pt===void 0&&(pt=new ps,R[J]=pt),pt.getGripSpace()},this.getHand=function(J){let pt=R[J];return pt===void 0&&(pt=new ps,R[J]=pt),pt.getHandSpace()};function U(J){let pt=w.indexOf(J.inputSource);if(pt===-1)return;let rt=R[pt];rt!==void 0&&(rt.update(J.inputSource,J.frame,c||a),rt.dispatchEvent({type:J.type,data:J.inputSource}))}function L(){s.removeEventListener("select",U),s.removeEventListener("selectstart",U),s.removeEventListener("selectend",U),s.removeEventListener("squeeze",U),s.removeEventListener("squeezestart",U),s.removeEventListener("squeezeend",U),s.removeEventListener("end",L),s.removeEventListener("inputsourceschange",B);for(let J=0;J<R.length;J++){let pt=w[J];pt!==null&&(w[J]=null,R[J].disconnect(pt))}H=null,$=null,m.reset();for(let J in f)delete f[J];t.setRenderTarget(M),p=null,h=null,u=null,s=null,E=null,Ut.stop(),n.isPresenting=!1,t.setPixelRatio(v),t.setSize(I.width,I.height,!1),n.dispatchEvent({type:"sessionend"})}this.setFramebufferScaleFactor=function(J){r=J,n.isPresenting===!0&&Et("WebXRManager: Cannot change framebuffer scale while presenting.")},this.setReferenceSpaceType=function(J){o=J,n.isPresenting===!0&&Et("WebXRManager: Cannot change reference space type while presenting.")},this.getReferenceSpace=function(){return c||a},this.setReferenceSpace=function(J){c=J},this.getBaseLayer=function(){return h!==null?h:p},this.getBinding=function(){return u===null&&y&&(u=new XRWebGLBinding(s,e)),u},this.getFrame=function(){return _},this.getSession=function(){return s},this.setSession=async function(J){if(s=J,s!==null){if(M=t.getRenderTarget(),s.addEventListener("select",U),s.addEventListener("selectstart",U),s.addEventListener("selectend",U),s.addEventListener("squeeze",U),s.addEventListener("squeezestart",U),s.addEventListener("squeezeend",U),s.addEventListener("end",L),s.addEventListener("inputsourceschange",B),S.xrCompatible!==!0&&await e.makeXRCompatible(),v=t.getPixelRatio(),t.getSize(I),y&&"createProjectionLayer"in XRWebGLBinding.prototype){let rt=null,Tt=null,Lt=null;S.depth&&(Lt=S.stencil?e.DEPTH24_STENCIL8:e.DEPTH_COMPONENT24,rt=S.stencil?yi:Ln,Tt=S.stencil?ys:yn);let Rt={colorFormat:e.RGBA8,depthFormat:Lt,scaleFactor:r};u=this.getBinding(),h=u.createProjectionLayer(Rt),s.updateRenderState({layers:[h]}),t.setPixelRatio(1),t.setSize(h.textureWidth,h.textureHeight,!1),E=new Ke(h.textureWidth,h.textureHeight,{format:hn,type:He,depthTexture:new jn(h.textureWidth,h.textureHeight,Tt,void 0,void 0,void 0,void 0,void 0,void 0,rt),stencilBuffer:S.stencil,colorSpace:t.outputColorSpace,samples:S.antialias?4:0,resolveDepthBuffer:h.ignoreDepthValues===!1,resolveStencilBuffer:h.ignoreDepthValues===!1})}else{let rt={antialias:S.antialias,alpha:!0,depth:S.depth,stencil:S.stencil,framebufferScaleFactor:r};p=new XRWebGLLayer(s,e,rt),s.updateRenderState({baseLayer:p}),t.setPixelRatio(1),t.setSize(p.framebufferWidth,p.framebufferHeight,!1),E=new Ke(p.framebufferWidth,p.framebufferHeight,{format:hn,type:He,colorSpace:t.outputColorSpace,stencilBuffer:S.stencil,resolveDepthBuffer:p.ignoreDepthValues===!1,resolveStencilBuffer:p.ignoreDepthValues===!1})}E.isXRRenderTarget=!0,this.setFoveation(l),c=null,a=await s.requestReferenceSpace(o),Ut.setContext(s),Ut.start(),n.isPresenting=!0,n.dispatchEvent({type:"sessionstart"})}},this.getEnvironmentBlendMode=function(){if(s!==null)return s.environmentBlendMode},this.getDepthTexture=function(){return m.getDepthTexture()};function B(J){for(let pt=0;pt<J.removed.length;pt++){let rt=J.removed[pt],Tt=w.indexOf(rt);Tt>=0&&(w[Tt]=null,R[Tt].disconnect(rt))}for(let pt=0;pt<J.added.length;pt++){let rt=J.added[pt],Tt=w.indexOf(rt);if(Tt===-1){for(let Rt=0;Rt<R.length;Rt++)if(Rt>=w.length){w.push(rt),Tt=Rt;break}else if(w[Rt]===null){w[Rt]=rt,Tt=Rt;break}if(Tt===-1)break}let Lt=R[Tt];Lt&&Lt.connect(rt)}}let K=new k,tt=new k;function ct(J,pt,rt){K.setFromMatrixPosition(pt.matrixWorld),tt.setFromMatrixPosition(rt.matrixWorld);let Tt=K.distanceTo(tt),Lt=pt.projectionMatrix.elements,Rt=rt.projectionMatrix.elements,de=Lt[14]/(Lt[10]-1),Ht=Lt[14]/(Lt[10]+1),ee=(Lt[9]+1)/Lt[5],ce=(Lt[9]-1)/Lt[5],Vt=(Lt[8]-1)/Lt[0],Me=(Rt[8]+1)/Rt[0],ue=de*Vt,We=de*Me,P=Tt/(-Vt+Me),we=P*-Vt;if(pt.matrixWorld.decompose(J.position,J.quaternion,J.scale),J.translateX(we),J.translateZ(P),J.matrixWorld.compose(J.position,J.quaternion,J.scale),J.matrixWorldInverse.copy(J.matrixWorld).invert(),Lt[10]===-1)J.projectionMatrix.copy(pt.projectionMatrix),J.projectionMatrixInverse.copy(pt.projectionMatrixInverse);else{let Wt=de+P,ae=Ht+P,ht=ue-we,me=We+(Tt-we),b=ee*Ht/ae*Wt,g=ce*Ht/ae*Wt;J.projectionMatrix.makePerspective(ht,me,b,g,Wt,ae),J.projectionMatrixInverse.copy(J.projectionMatrix).invert()}}function bt(J,pt){pt===null?J.matrixWorld.copy(J.matrix):J.matrixWorld.multiplyMatrices(pt.matrixWorld,J.matrix),J.matrixWorldInverse.copy(J.matrixWorld).invert()}this.updateCamera=function(J){if(s===null)return;let pt=J.near,rt=J.far;m.texture!==null&&(m.depthNear>0&&(pt=m.depthNear),m.depthFar>0&&(rt=m.depthFar)),O.near=D.near=A.near=pt,O.far=D.far=A.far=rt,(H!==O.near||$!==O.far)&&(s.updateRenderState({depthNear:O.near,depthFar:O.far}),H=O.near,$=O.far),O.layers.mask=J.layers.mask|6,A.layers.mask=O.layers.mask&-5,D.layers.mask=O.layers.mask&-3;let Tt=J.parent,Lt=O.cameras;bt(O,Tt);for(let Rt=0;Rt<Lt.length;Rt++)bt(Lt[Rt],Tt);Lt.length===2?ct(O,A,D):O.projectionMatrix.copy(A.projectionMatrix),vt(J,O,Tt)};function vt(J,pt,rt){rt===null?J.matrix.copy(pt.matrixWorld):(J.matrix.copy(rt.matrixWorld),J.matrix.invert(),J.matrix.multiply(pt.matrixWorld)),J.matrix.decompose(J.position,J.quaternion,J.scale),J.updateMatrixWorld(!0),J.projectionMatrix.copy(pt.projectionMatrix),J.projectionMatrixInverse.copy(pt.projectionMatrixInverse),J.isPerspectiveCamera&&(J.fov=_a*2*Math.atan(1/J.projectionMatrix.elements[5]),J.zoom=1)}this.getCamera=function(){return O},this.getFoveation=function(){if(!(h===null&&p===null))return l},this.setFoveation=function(J){l=J,h!==null&&(h.fixedFoveation=J),p!==null&&p.fixedFoveation!==void 0&&(p.fixedFoveation=J)},this.hasDepthSensing=function(){return m.texture!==null},this.getDepthSensingMesh=function(){return m.getMesh(O)},this.getCameraTexture=function(J){return f[J]};let Gt=null;function Qt(J,pt){if(d=pt.getViewerPose(c||a),_=pt,d!==null){let rt=d.views;p!==null&&(t.setRenderTargetFramebuffer(E,p.framebuffer),t.setRenderTarget(E));let Tt=!1;rt.length!==O.cameras.length&&(O.cameras.length=0,Tt=!0);for(let Ht=0;Ht<rt.length;Ht++){let ee=rt[Ht],ce=null;if(p!==null)ce=p.getViewport(ee);else{let Me=u.getViewSubImage(h,ee);ce=Me.viewport,Ht===0&&(t.setRenderTargetTextures(E,Me.colorTexture,Me.depthStencilTexture),t.setRenderTarget(E))}let Vt=T[Ht];Vt===void 0&&(Vt=new Ue,Vt.layers.enable(Ht),Vt.viewport=new pe,T[Ht]=Vt),Vt.matrix.fromArray(ee.transform.matrix),Vt.matrix.decompose(Vt.position,Vt.quaternion,Vt.scale),Vt.projectionMatrix.fromArray(ee.projectionMatrix),Vt.projectionMatrixInverse.copy(Vt.projectionMatrix).invert(),Vt.viewport.set(ce.x,ce.y,ce.width,ce.height),Ht===0&&(O.matrix.copy(Vt.matrix),O.matrix.decompose(O.position,O.quaternion,O.scale)),Tt===!0&&O.cameras.push(Vt)}let Lt=s.enabledFeatures;if(Lt&&Lt.includes("depth-sensing")&&s.depthUsage=="gpu-optimized"&&y){u=n.getBinding();let Ht=u.getDepthInformation(rt[0]);Ht&&Ht.isValid&&Ht.texture&&m.init(Ht,s.renderState)}if(Lt&&Lt.includes("camera-access")&&y){t.state.unbindTexture(),u=n.getBinding();for(let Ht=0;Ht<rt.length;Ht++){let ee=rt[Ht].camera;if(ee){let ce=f[ee];ce||(ce=new tr,f[ee]=ce);let Vt=u.getCameraImage(ee);ce.sourceTexture=Vt}}}}for(let rt=0;rt<R.length;rt++){let Tt=w[rt],Lt=R[rt];Tt!==null&&Lt!==void 0&&Lt.update(Tt,pt,c||a)}Gt&&Gt(J,pt),pt.detectedPlanes&&n.dispatchEvent({type:"planesdetected",data:pt}),_=null}let Ut=new Bd;Ut.setAnimationLoop(Qt),this.setAnimationLoop=function(J){Gt=J},this.dispose=function(){}}},F_=new fe,Wd=new Pt;Wd.set(-1,0,0,0,1,0,0,0,1);function O_(i,t){function e(m,f){m.matrixAutoUpdate===!0&&m.updateMatrix(),f.value.copy(m.matrix)}function n(m,f){f.color.getRGB(m.fogColor.value,dc(i)),f.isFog?(m.fogNear.value=f.near,m.fogFar.value=f.far):f.isFogExp2&&(m.fogDensity.value=f.density)}function s(m,f,S,M,E){f.isNodeMaterial?f.uniformsNeedUpdate=!1:f.isMeshBasicMaterial?r(m,f):f.isMeshLambertMaterial?(r(m,f),f.envMap&&(m.envMapIntensity.value=f.envMapIntensity)):f.isMeshToonMaterial?(r(m,f),u(m,f)):f.isMeshPhongMaterial?(r(m,f),d(m,f),f.envMap&&(m.envMapIntensity.value=f.envMapIntensity)):f.isMeshStandardMaterial?(r(m,f),h(m,f),f.isMeshPhysicalMaterial&&p(m,f,E)):f.isMeshMatcapMaterial?(r(m,f),_(m,f)):f.isMeshDepthMaterial?r(m,f):f.isMeshDistanceMaterial?(r(m,f),y(m,f)):f.isMeshNormalMaterial?r(m,f):f.isLineBasicMaterial?(a(m,f),f.isLineDashedMaterial&&o(m,f)):f.isPointsMaterial?l(m,f,S,M):f.isSpriteMaterial?c(m,f):f.isShadowMaterial?(m.color.value.copy(f.color),m.opacity.value=f.opacity):f.isShaderMaterial&&(f.uniformsNeedUpdate=!1)}function r(m,f){m.opacity.value=f.opacity,f.color&&m.diffuse.value.copy(f.color),f.emissive&&m.emissive.value.copy(f.emissive).multiplyScalar(f.emissiveIntensity),f.map&&(m.map.value=f.map,e(f.map,m.mapTransform)),f.alphaMap&&(m.alphaMap.value=f.alphaMap,e(f.alphaMap,m.alphaMapTransform)),f.bumpMap&&(m.bumpMap.value=f.bumpMap,e(f.bumpMap,m.bumpMapTransform),m.bumpScale.value=f.bumpScale,f.side===ze&&(m.bumpScale.value*=-1)),f.normalMap&&(m.normalMap.value=f.normalMap,e(f.normalMap,m.normalMapTransform),m.normalScale.value.copy(f.normalScale),f.side===ze&&m.normalScale.value.negate()),f.displacementMap&&(m.displacementMap.value=f.displacementMap,e(f.displacementMap,m.displacementMapTransform),m.displacementScale.value=f.displacementScale,m.displacementBias.value=f.displacementBias),f.emissiveMap&&(m.emissiveMap.value=f.emissiveMap,e(f.emissiveMap,m.emissiveMapTransform)),f.specularMap&&(m.specularMap.value=f.specularMap,e(f.specularMap,m.specularMapTransform)),f.alphaTest>0&&(m.alphaTest.value=f.alphaTest);let S=t.get(f),M=S.envMap,E=S.envMapRotation;M&&(m.envMap.value=M,m.envMapRotation.value.setFromMatrix4(F_.makeRotationFromEuler(E)).transpose(),M.isCubeTexture&&M.isRenderTargetTexture===!1&&m.envMapRotation.value.premultiply(Wd),m.reflectivity.value=f.reflectivity,m.ior.value=f.ior,m.refractionRatio.value=f.refractionRatio),f.lightMap&&(m.lightMap.value=f.lightMap,m.lightMapIntensity.value=f.lightMapIntensity,e(f.lightMap,m.lightMapTransform)),f.aoMap&&(m.aoMap.value=f.aoMap,m.aoMapIntensity.value=f.aoMapIntensity,e(f.aoMap,m.aoMapTransform))}function a(m,f){m.diffuse.value.copy(f.color),m.opacity.value=f.opacity,f.map&&(m.map.value=f.map,e(f.map,m.mapTransform))}function o(m,f){m.dashSize.value=f.dashSize,m.totalSize.value=f.dashSize+f.gapSize,m.scale.value=f.scale}function l(m,f,S,M){m.diffuse.value.copy(f.color),m.opacity.value=f.opacity,m.size.value=f.size*S,m.scale.value=M*.5,f.map&&(m.map.value=f.map,e(f.map,m.uvTransform)),f.alphaMap&&(m.alphaMap.value=f.alphaMap,e(f.alphaMap,m.alphaMapTransform)),f.alphaTest>0&&(m.alphaTest.value=f.alphaTest)}function c(m,f){m.diffuse.value.copy(f.color),m.opacity.value=f.opacity,m.rotation.value=f.rotation,f.map&&(m.map.value=f.map,e(f.map,m.mapTransform)),f.alphaMap&&(m.alphaMap.value=f.alphaMap,e(f.alphaMap,m.alphaMapTransform)),f.alphaTest>0&&(m.alphaTest.value=f.alphaTest)}function d(m,f){m.specular.value.copy(f.specular),m.shininess.value=Math.max(f.shininess,1e-4)}function u(m,f){f.gradientMap&&(m.gradientMap.value=f.gradientMap)}function h(m,f){m.metalness.value=f.metalness,f.metalnessMap&&(m.metalnessMap.value=f.metalnessMap,e(f.metalnessMap,m.metalnessMapTransform)),m.roughness.value=f.roughness,f.roughnessMap&&(m.roughnessMap.value=f.roughnessMap,e(f.roughnessMap,m.roughnessMapTransform)),f.envMap&&(m.envMapIntensity.value=f.envMapIntensity)}function p(m,f,S){m.ior.value=f.ior,f.sheen>0&&(m.sheenColor.value.copy(f.sheenColor).multiplyScalar(f.sheen),m.sheenRoughness.value=f.sheenRoughness,f.sheenColorMap&&(m.sheenColorMap.value=f.sheenColorMap,e(f.sheenColorMap,m.sheenColorMapTransform)),f.sheenRoughnessMap&&(m.sheenRoughnessMap.value=f.sheenRoughnessMap,e(f.sheenRoughnessMap,m.sheenRoughnessMapTransform))),f.clearcoat>0&&(m.clearcoat.value=f.clearcoat,m.clearcoatRoughness.value=f.clearcoatRoughness,f.clearcoatMap&&(m.clearcoatMap.value=f.clearcoatMap,e(f.clearcoatMap,m.clearcoatMapTransform)),f.clearcoatRoughnessMap&&(m.clearcoatRoughnessMap.value=f.clearcoatRoughnessMap,e(f.clearcoatRoughnessMap,m.clearcoatRoughnessMapTransform)),f.clearcoatNormalMap&&(m.clearcoatNormalMap.value=f.clearcoatNormalMap,e(f.clearcoatNormalMap,m.clearcoatNormalMapTransform),m.clearcoatNormalScale.value.copy(f.clearcoatNormalScale),f.side===ze&&m.clearcoatNormalScale.value.negate())),f.dispersion>0&&(m.dispersion.value=f.dispersion),f.iridescence>0&&(m.iridescence.value=f.iridescence,m.iridescenceIOR.value=f.iridescenceIOR,m.iridescenceThicknessMinimum.value=f.iridescenceThicknessRange[0],m.iridescenceThicknessMaximum.value=f.iridescenceThicknessRange[1],f.iridescenceMap&&(m.iridescenceMap.value=f.iridescenceMap,e(f.iridescenceMap,m.iridescenceMapTransform)),f.iridescenceThicknessMap&&(m.iridescenceThicknessMap.value=f.iridescenceThicknessMap,e(f.iridescenceThicknessMap,m.iridescenceThicknessMapTransform))),f.transmission>0&&(m.transmission.value=f.transmission,m.transmissionSamplerMap.value=S.texture,m.transmissionSamplerSize.value.set(S.width,S.height),f.transmissionMap&&(m.transmissionMap.value=f.transmissionMap,e(f.transmissionMap,m.transmissionMapTransform)),m.thickness.value=f.thickness,f.thicknessMap&&(m.thicknessMap.value=f.thicknessMap,e(f.thicknessMap,m.thicknessMapTransform)),m.attenuationDistance.value=f.attenuationDistance,m.attenuationColor.value.copy(f.attenuationColor)),f.anisotropy>0&&(m.anisotropyVector.value.set(f.anisotropy*Math.cos(f.anisotropyRotation),f.anisotropy*Math.sin(f.anisotropyRotation)),f.anisotropyMap&&(m.anisotropyMap.value=f.anisotropyMap,e(f.anisotropyMap,m.anisotropyMapTransform))),m.specularIntensity.value=f.specularIntensity,m.specularColor.value.copy(f.specularColor),f.specularColorMap&&(m.specularColorMap.value=f.specularColorMap,e(f.specularColorMap,m.specularColorMapTransform)),f.specularIntensityMap&&(m.specularIntensityMap.value=f.specularIntensityMap,e(f.specularIntensityMap,m.specularIntensityMapTransform))}function _(m,f){f.matcap&&(m.matcap.value=f.matcap)}function y(m,f){let S=t.get(f).light;m.referencePosition.value.setFromMatrixPosition(S.matrixWorld),m.nearDistance.value=S.shadow.camera.near,m.farDistance.value=S.shadow.camera.far}return{refreshFogUniforms:n,refreshMaterialUniforms:s}}function B_(i,t,e,n){let s={},r={},a=[],o=i.getParameter(i.MAX_UNIFORM_BUFFER_BINDINGS);function l(S,M){let E=M.program;n.uniformBlockBinding(S,E)}function c(S,M){let E=s[S.id];E===void 0&&(_(S),E=d(S),s[S.id]=E,S.addEventListener("dispose",m));let R=M.program;n.updateUBOMapping(S,R);let w=t.render.frame;r[S.id]!==w&&(h(S),r[S.id]=w)}function d(S){let M=u();S.__bindingPointIndex=M;let E=i.createBuffer(),R=S.__size,w=S.usage;return i.bindBuffer(i.UNIFORM_BUFFER,E),i.bufferData(i.UNIFORM_BUFFER,R,w),i.bindBuffer(i.UNIFORM_BUFFER,null),i.bindBufferBase(i.UNIFORM_BUFFER,M,E),E}function u(){for(let S=0;S<o;S++)if(a.indexOf(S)===-1)return a.push(S),S;return Ct("WebGLRenderer: Maximum number of simultaneously usable uniforms groups reached."),0}function h(S){let M=s[S.id],E=S.uniforms,R=S.__cache;i.bindBuffer(i.UNIFORM_BUFFER,M);for(let w=0,I=E.length;w<I;w++){let v=Array.isArray(E[w])?E[w]:[E[w]];for(let A=0,D=v.length;A<D;A++){let T=v[A];if(p(T,w,A,R)===!0){let O=T.__offset,H=Array.isArray(T.value)?T.value:[T.value],$=0;for(let U=0;U<H.length;U++){let L=H[U],B=y(L);typeof L=="number"||typeof L=="boolean"?(T.__data[0]=L,i.bufferSubData(i.UNIFORM_BUFFER,O+$,T.__data)):L.isMatrix3?(T.__data[0]=L.elements[0],T.__data[1]=L.elements[1],T.__data[2]=L.elements[2],T.__data[3]=0,T.__data[4]=L.elements[3],T.__data[5]=L.elements[4],T.__data[6]=L.elements[5],T.__data[7]=0,T.__data[8]=L.elements[6],T.__data[9]=L.elements[7],T.__data[10]=L.elements[8],T.__data[11]=0):ArrayBuffer.isView(L)?T.__data.set(new L.constructor(L.buffer,L.byteOffset,T.__data.length)):(L.toArray(T.__data,$),$+=B.storage/Float32Array.BYTES_PER_ELEMENT)}i.bufferSubData(i.UNIFORM_BUFFER,O,T.__data)}}}i.bindBuffer(i.UNIFORM_BUFFER,null)}function p(S,M,E,R){let w=S.value,I=M+"_"+E;if(R[I]===void 0)return typeof w=="number"||typeof w=="boolean"?R[I]=w:ArrayBuffer.isView(w)?R[I]=w.slice():R[I]=w.clone(),!0;{let v=R[I];if(typeof w=="number"||typeof w=="boolean"){if(v!==w)return R[I]=w,!0}else{if(ArrayBuffer.isView(w))return!0;if(v.equals(w)===!1)return v.copy(w),!0}}return!1}function _(S){let M=S.uniforms,E=0,R=16;for(let I=0,v=M.length;I<v;I++){let A=Array.isArray(M[I])?M[I]:[M[I]];for(let D=0,T=A.length;D<T;D++){let O=A[D],H=Array.isArray(O.value)?O.value:[O.value];for(let $=0,U=H.length;$<U;$++){let L=H[$],B=y(L),K=E%R,tt=K%B.boundary,ct=K+tt;E+=tt,ct!==0&&R-ct<B.storage&&(E+=R-ct),O.__data=new Float32Array(B.storage/Float32Array.BYTES_PER_ELEMENT),O.__offset=E,E+=B.storage}}}let w=E%R;return w>0&&(E+=R-w),S.__size=E,S.__cache={},this}function y(S){let M={boundary:0,storage:0};return typeof S=="number"||typeof S=="boolean"?(M.boundary=4,M.storage=4):S.isVector2?(M.boundary=8,M.storage=8):S.isVector3||S.isColor?(M.boundary=16,M.storage=12):S.isVector4?(M.boundary=16,M.storage=16):S.isMatrix3?(M.boundary=48,M.storage=48):S.isMatrix4?(M.boundary=64,M.storage=64):S.isTexture?Et("WebGLRenderer: Texture samplers can not be part of an uniforms group."):ArrayBuffer.isView(S)?(M.boundary=16,M.storage=S.byteLength):Et("WebGLRenderer: Unsupported uniform value type.",S),M}function m(S){let M=S.target;M.removeEventListener("dispose",m);let E=a.indexOf(M.__bindingPointIndex);a.splice(E,1),i.deleteBuffer(s[M.id]),delete s[M.id],delete r[M.id]}function f(){for(let S in s)i.deleteBuffer(s[S]);a=[],s={},r={}}return{bind:l,update:c,dispose:f}}var k_=new Uint16Array([12469,15057,12620,14925,13266,14620,13807,14376,14323,13990,14545,13625,14713,13328,14840,12882,14931,12528,14996,12233,15039,11829,15066,11525,15080,11295,15085,10976,15082,10705,15073,10495,13880,14564,13898,14542,13977,14430,14158,14124,14393,13732,14556,13410,14702,12996,14814,12596,14891,12291,14937,11834,14957,11489,14958,11194,14943,10803,14921,10506,14893,10278,14858,9960,14484,14039,14487,14025,14499,13941,14524,13740,14574,13468,14654,13106,14743,12678,14818,12344,14867,11893,14889,11509,14893,11180,14881,10751,14852,10428,14812,10128,14765,9754,14712,9466,14764,13480,14764,13475,14766,13440,14766,13347,14769,13070,14786,12713,14816,12387,14844,11957,14860,11549,14868,11215,14855,10751,14825,10403,14782,10044,14729,9651,14666,9352,14599,9029,14967,12835,14966,12831,14963,12804,14954,12723,14936,12564,14917,12347,14900,11958,14886,11569,14878,11247,14859,10765,14828,10401,14784,10011,14727,9600,14660,9289,14586,8893,14508,8533,15111,12234,15110,12234,15104,12216,15092,12156,15067,12010,15028,11776,14981,11500,14942,11205,14902,10752,14861,10393,14812,9991,14752,9570,14682,9252,14603,8808,14519,8445,14431,8145,15209,11449,15208,11451,15202,11451,15190,11438,15163,11384,15117,11274,15055,10979,14994,10648,14932,10343,14871,9936,14803,9532,14729,9218,14645,8742,14556,8381,14461,8020,14365,7603,15273,10603,15272,10607,15267,10619,15256,10631,15231,10614,15182,10535,15118,10389,15042,10167,14963,9787,14883,9447,14800,9115,14710,8665,14615,8318,14514,7911,14411,7507,14279,7198,15314,9675,15313,9683,15309,9712,15298,9759,15277,9797,15229,9773,15166,9668,15084,9487,14995,9274,14898,8910,14800,8539,14697,8234,14590,7790,14479,7409,14367,7067,14178,6621,15337,8619,15337,8631,15333,8677,15325,8769,15305,8871,15264,8940,15202,8909,15119,8775,15022,8565,14916,8328,14804,8009,14688,7614,14569,7287,14448,6888,14321,6483,14088,6171,15350,7402,15350,7419,15347,7480,15340,7613,15322,7804,15287,7973,15229,8057,15148,8012,15046,7846,14933,7611,14810,7357,14682,7069,14552,6656,14421,6316,14251,5948,14007,5528,15356,5942,15356,5977,15353,6119,15348,6294,15332,6551,15302,6824,15249,7044,15171,7122,15070,7050,14949,6861,14818,6611,14679,6349,14538,6067,14398,5651,14189,5311,13935,4958,15359,4123,15359,4153,15356,4296,15353,4646,15338,5160,15311,5508,15263,5829,15188,6042,15088,6094,14966,6001,14826,5796,14678,5543,14527,5287,14377,4985,14133,4586,13869,4257,15360,1563,15360,1642,15358,2076,15354,2636,15341,3350,15317,4019,15273,4429,15203,4732,15105,4911,14981,4932,14836,4818,14679,4621,14517,4386,14359,4156,14083,3795,13808,3437,15360,122,15360,137,15358,285,15355,636,15344,1274,15322,2177,15281,2765,15215,3223,15120,3451,14995,3569,14846,3567,14681,3466,14511,3305,14344,3121,14037,2800,13753,2467,15360,0,15360,1,15359,21,15355,89,15346,253,15325,479,15287,796,15225,1148,15133,1492,15008,1749,14856,1882,14685,1886,14506,1783,14324,1608,13996,1398,13702,1183]),Bn=null;function z_(){return Bn===null&&(Bn=new ba(k_,16,16,bi,On),Bn.name="DFG_LUT",Bn.minFilter=Pe,Bn.magFilter=Pe,Bn.wrapS=Pn,Bn.wrapT=Pn,Bn.generateMipmaps=!1,Bn.needsUpdate=!0),Bn}var Uo=class{constructor(t={}){let{canvas:e=cd(),context:n=null,depth:s=!0,stencil:r=!1,alpha:a=!1,antialias:o=!1,premultipliedAlpha:l=!0,preserveDrawingBuffer:c=!1,powerPreference:d="default",failIfMajorPerformanceCaveat:u=!1,reversedDepthBuffer:h=!1,outputBufferType:p=He}=t;this.isWebGLRenderer=!0;let _;if(n!==null){if(typeof WebGLRenderingContext<"u"&&n instanceof WebGLRenderingContext)throw new Error("THREE.WebGLRenderer: WebGL 1 is not supported since r163.");_=n.getContextAttributes().alpha}else _=a;let y=p,m=new Set([Ja,Za,Ya]),f=new Set([He,yn,xs,ys,Xa,qa]),S=new Uint32Array(4),M=new Int32Array(4),E=new k,R=null,w=null,I=[],v=[],A=null;this.domElement=e,this.debug={checkShaderErrors:!0,onShaderError:null},this.autoClear=!0,this.autoClearColor=!0,this.autoClearDepth=!0,this.autoClearStencil=!0,this.sortObjects=!0,this.clippingPlanes=[],this.localClippingEnabled=!1,this.toneMapping=xn,this.toneMappingExposure=1,this.transmissionResolutionScale=1;let D=this,T=!1,O=null;this._outputColorSpace=Ze;let H=0,$=0,U=null,L=-1,B=null,K=new pe,tt=new pe,ct=null,bt=new zt(0),vt=0,Gt=e.width,Qt=e.height,Ut=1,J=null,pt=null,rt=new pe(0,0,Gt,Qt),Tt=new pe(0,0,Gt,Qt),Lt=!1,Rt=new ms,de=!1,Ht=!1,ee=new fe,ce=new k,Vt=new pe,Me={background:null,fog:null,environment:null,overrideMaterial:null,isScene:!0},ue=!1;function We(){return U===null?Ut:1}let P=n;function we(x,N){return e.getContext(x,N)}try{let x={alpha:!0,depth:s,stencil:r,antialias:o,premultipliedAlpha:l,preserveDrawingBuffer:c,powerPreference:d,failIfMajorPerformanceCaveat:u};if("setAttribute"in e&&e.setAttribute("data-engine",`three.js r${"184"}`),e.addEventListener("webglcontextlost",j,!1),e.addEventListener("webglcontextrestored",Mt,!1),e.addEventListener("webglcontextcreationerror",Dt,!1),P===null){let N="webgl2";if(P=we(N,x),P===null)throw we(N)?new Error("Error creating WebGL context with your selected attributes."):new Error("Error creating WebGL context.")}}catch(x){throw Ct("WebGLRenderer: "+x.message),x}let Wt,ae,ht,me,b,g,F,q,Q,nt,lt,W,Y,mt,xt,at,it,It,Ft,Jt,C,st,X;function gt(){Wt=new qm(P),Wt.init(),C=new D_(P,Wt),ae=new km(P,Wt,t,C),ht=new P_(P,Wt),ae.reversedDepthBuffer&&h&&ht.buffers.depth.setReversed(!0),me=new Jm(P),b=new __,g=new L_(P,Wt,ht,b,ae,C,me),F=new Xm(D),q=new Qu(P),st=new Om(P,q),Q=new Ym(P,q,me,st),nt=new jm(P,Q,q,st,me),It=new Km(P,ae,g),xt=new zm(b),lt=new g_(D,F,Wt,ae,st,xt),W=new O_(D,b),Y=new x_,mt=new A_(Wt),it=new Fm(D,F,ht,nt,_,l),at=new I_(D,nt,ae),X=new B_(P,me,ae,ht),Ft=new Bm(P,Wt,me),Jt=new Zm(P,Wt,me),me.programs=lt.programs,D.capabilities=ae,D.extensions=Wt,D.properties=b,D.renderLists=Y,D.shadowMap=at,D.state=ht,D.info=me}gt(),y!==He&&(A=new tg(y,e.width,e.height,s,r));let ot=new Rc(D,P);this.xr=ot,this.getContext=function(){return P},this.getContextAttributes=function(){return P.getContextAttributes()},this.forceContextLoss=function(){let x=Wt.get("WEBGL_lose_context");x&&x.loseContext()},this.forceContextRestore=function(){let x=Wt.get("WEBGL_lose_context");x&&x.restoreContext()},this.getPixelRatio=function(){return Ut},this.setPixelRatio=function(x){x!==void 0&&(Ut=x,this.setSize(Gt,Qt,!1))},this.getSize=function(x){return x.set(Gt,Qt)},this.setSize=function(x,N,G=!0){if(ot.isPresenting){Et("WebGLRenderer: Can't change size while VR device is presenting.");return}Gt=x,Qt=N,e.width=Math.floor(x*Ut),e.height=Math.floor(N*Ut),G===!0&&(e.style.width=x+"px",e.style.height=N+"px"),A!==null&&A.setSize(e.width,e.height),this.setViewport(0,0,x,N)},this.getDrawingBufferSize=function(x){return x.set(Gt*Ut,Qt*Ut).floor()},this.setDrawingBufferSize=function(x,N,G){Gt=x,Qt=N,Ut=G,e.width=Math.floor(x*G),e.height=Math.floor(N*G),this.setViewport(0,0,x,N)},this.setEffects=function(x){if(y===He){Ct("THREE.WebGLRenderer: setEffects() requires outputBufferType set to HalfFloatType or FloatType.");return}if(x){for(let N=0;N<x.length;N++)if(x[N].isOutputPass===!0){Et("THREE.WebGLRenderer: OutputPass is not needed in setEffects(). Tone mapping and color space conversion are applied automatically.");break}}A.setEffects(x||[])},this.getCurrentViewport=function(x){return x.copy(K)},this.getViewport=function(x){return x.copy(rt)},this.setViewport=function(x,N,G,z){x.isVector4?rt.set(x.x,x.y,x.z,x.w):rt.set(x,N,G,z),ht.viewport(K.copy(rt).multiplyScalar(Ut).round())},this.getScissor=function(x){return x.copy(Tt)},this.setScissor=function(x,N,G,z){x.isVector4?Tt.set(x.x,x.y,x.z,x.w):Tt.set(x,N,G,z),ht.scissor(tt.copy(Tt).multiplyScalar(Ut).round())},this.getScissorTest=function(){return Lt},this.setScissorTest=function(x){ht.setScissorTest(Lt=x)},this.setOpaqueSort=function(x){J=x},this.setTransparentSort=function(x){pt=x},this.getClearColor=function(x){return x.copy(it.getClearColor())},this.setClearColor=function(){it.setClearColor(...arguments)},this.getClearAlpha=function(){return it.getClearAlpha()},this.setClearAlpha=function(){it.setClearAlpha(...arguments)},this.clear=function(x=!0,N=!0,G=!0){let z=0;if(x){let V=!1;if(U!==null){let ft=U.texture.format;V=m.has(ft)}if(V){let ft=U.texture.type,yt=f.has(ft),ut=it.getClearColor(),St=it.getClearAlpha(),wt=ut.r,Nt=ut.g,Bt=ut.b;yt?(S[0]=wt,S[1]=Nt,S[2]=Bt,S[3]=St,P.clearBufferuiv(P.COLOR,0,S)):(M[0]=wt,M[1]=Nt,M[2]=Bt,M[3]=St,P.clearBufferiv(P.COLOR,0,M))}else z|=P.COLOR_BUFFER_BIT}N&&(z|=P.DEPTH_BUFFER_BIT,this.state.buffers.depth.setMask(!0)),G&&(z|=P.STENCIL_BUFFER_BIT,this.state.buffers.stencil.setMask(4294967295)),z!==0&&P.clear(z)},this.clearColor=function(){this.clear(!0,!1,!1)},this.clearDepth=function(){this.clear(!1,!0,!1)},this.clearStencil=function(){this.clear(!1,!1,!0)},this.setNodesHandler=function(x){x.setRenderer(this),O=x},this.dispose=function(){e.removeEventListener("webglcontextlost",j,!1),e.removeEventListener("webglcontextrestored",Mt,!1),e.removeEventListener("webglcontextcreationerror",Dt,!1),it.dispose(),Y.dispose(),mt.dispose(),b.dispose(),F.dispose(),nt.dispose(),st.dispose(),X.dispose(),lt.dispose(),ot.dispose(),ot.removeEventListener("sessionstart",Lc),ot.removeEventListener("sessionend",Dc),Mi.stop()};function j(x){x.preventDefault(),hc("WebGLRenderer: Context Lost."),T=!0}function Mt(){hc("WebGLRenderer: Context Restored."),T=!1;let x=me.autoReset,N=at.enabled,G=at.autoUpdate,z=at.needsUpdate,V=at.type;gt(),me.autoReset=x,at.enabled=N,at.autoUpdate=G,at.needsUpdate=z,at.type=V}function Dt(x){Ct("WebGLRenderer: A WebGL context could not be created. Reason: ",x.statusMessage)}function _e(x){let N=x.target;N.removeEventListener("dispose",_e),ne(N)}function ne(x){Vn(x),b.remove(x)}function Vn(x){let N=b.get(x).programs;N!==void 0&&(N.forEach(function(G){lt.releaseProgram(G)}),x.isShaderMaterial&&lt.releaseShaderCache(x))}this.renderBufferDirect=function(x,N,G,z,V,ft){N===null&&(N=Me);let yt=V.isMesh&&V.matrixWorld.determinant()<0,ut=Zd(x,N,G,z,V);ht.setMaterial(z,yt);let St=G.index,wt=1;if(z.wireframe===!0){if(St=Q.getWireframeAttribute(G),St===void 0)return;wt=2}let Nt=G.drawRange,Bt=G.attributes.position,At=Nt.start*wt,ie=(Nt.start+Nt.count)*wt;ft!==null&&(At=Math.max(At,ft.start*wt),ie=Math.min(ie,(ft.start+ft.count)*wt)),St!==null?(At=Math.max(At,0),ie=Math.min(ie,St.count)):Bt!=null&&(At=Math.max(At,0),ie=Math.min(ie,Bt.count));let ve=ie-At;if(ve<0||ve===1/0)return;st.setup(V,z,ut,G,St);let ge,se=Ft;if(St!==null&&(ge=q.get(St),se=Jt,se.setIndex(ge)),V.isMesh)z.wireframe===!0?(ht.setLineWidth(z.wireframeLinewidth*We()),se.setMode(P.LINES)):se.setMode(P.TRIANGLES);else if(V.isLine){let Le=z.linewidth;Le===void 0&&(Le=1),ht.setLineWidth(Le*We()),V.isLineSegments?se.setMode(P.LINES):V.isLineLoop?se.setMode(P.LINE_LOOP):se.setMode(P.LINE_STRIP)}else V.isPoints?se.setMode(P.POINTS):V.isSprite&&se.setMode(P.TRIANGLES);if(V.isBatchedMesh)if(Wt.get("WEBGL_multi_draw"))se.renderMultiDraw(V._multiDrawStarts,V._multiDrawCounts,V._multiDrawCount);else{let Le=V._multiDrawStarts,_t=V._multiDrawCounts,$e=V._multiDrawCount,Yt=St?q.get(St).bytesPerElement:1,tn=b.get(z).currentProgram.getUniforms();for(let wn=0;wn<$e;wn++)tn.setValue(P,"_gl_DrawID",wn),se.render(Le[wn]/Yt,_t[wn])}else if(V.isInstancedMesh)se.renderInstances(At,ve,V.count);else if(G.isInstancedBufferGeometry){let Le=G._maxInstanceCount!==void 0?G._maxInstanceCount:1/0,_t=Math.min(G.instanceCount,Le);se.renderInstances(At,ve,_t)}else se.render(At,ve)};function Mn(x,N,G){x.transparent===!0&&x.side===Un&&x.forceSinglePass===!1?(x.side=ze,x.needsUpdate=!0,Sr(x,N,G),x.side=Zn,x.needsUpdate=!0,Sr(x,N,G),x.side=Un):Sr(x,N,G)}this.compile=function(x,N,G=null){G===null&&(G=x),w=mt.get(G),w.init(N),v.push(w),G.traverseVisible(function(V){V.isLight&&V.layers.test(N.layers)&&(w.pushLight(V),V.castShadow&&w.pushShadow(V))}),x!==G&&x.traverseVisible(function(V){V.isLight&&V.layers.test(N.layers)&&(w.pushLight(V),V.castShadow&&w.pushShadow(V))}),w.setupLights();let z=new Set;return x.traverse(function(V){if(!(V.isMesh||V.isPoints||V.isLine||V.isSprite))return;let ft=V.material;if(ft)if(Array.isArray(ft))for(let yt=0;yt<ft.length;yt++){let ut=ft[yt];Mn(ut,G,V),z.add(ut)}else Mn(ft,G,V),z.add(ft)}),w=v.pop(),z},this.compileAsync=function(x,N,G=null){let z=this.compile(x,N,G);return new Promise(V=>{function ft(){if(z.forEach(function(yt){b.get(yt).currentProgram.isReady()&&z.delete(yt)}),z.size===0){V(x);return}setTimeout(ft,10)}Wt.get("KHR_parallel_shader_compile")!==null?ft():setTimeout(ft,10)})};let ko=null;function qd(x){ko&&ko(x)}function Lc(){Mi.stop()}function Dc(){Mi.start()}let Mi=new Bd;Mi.setAnimationLoop(qd),typeof self<"u"&&Mi.setContext(self),this.setAnimationLoop=function(x){ko=x,ot.setAnimationLoop(x),x===null?Mi.stop():Mi.start()},ot.addEventListener("sessionstart",Lc),ot.addEventListener("sessionend",Dc),this.render=function(x,N){if(N!==void 0&&N.isCamera!==!0){Ct("WebGLRenderer.render: camera is not an instance of THREE.Camera.");return}if(T===!0)return;O!==null&&O.renderStart(x,N);let G=ot.enabled===!0&&ot.isPresenting===!0,z=A!==null&&(U===null||G)&&A.begin(D,U);if(x.matrixWorldAutoUpdate===!0&&x.updateMatrixWorld(),N.parent===null&&N.matrixWorldAutoUpdate===!0&&N.updateMatrixWorld(),ot.enabled===!0&&ot.isPresenting===!0&&(A===null||A.isCompositing()===!1)&&(ot.cameraAutoUpdate===!0&&ot.updateCamera(N),N=ot.getCamera()),x.isScene===!0&&x.onBeforeRender(D,x,N,U),w=mt.get(x,v.length),w.init(N),w.state.textureUnits=g.getTextureUnits(),v.push(w),ee.multiplyMatrices(N.projectionMatrix,N.matrixWorldInverse),Rt.setFromProjectionMatrix(ee,vn,N.reversedDepth),Ht=this.localClippingEnabled,de=xt.init(this.clippingPlanes,Ht),R=Y.get(x,I.length),R.init(),I.push(R),ot.enabled===!0&&ot.isPresenting===!0){let yt=D.xr.getDepthSensingMesh();yt!==null&&zo(yt,N,-1/0,D.sortObjects)}zo(x,N,0,D.sortObjects),R.finish(),D.sortObjects===!0&&R.sort(J,pt),ue=ot.enabled===!1||ot.isPresenting===!1||ot.hasDepthSensing()===!1,ue&&it.addToRenderList(R,x),this.info.render.frame++,de===!0&&xt.beginShadows();let V=w.state.shadowsArray;if(at.render(V,x,N),de===!0&&xt.endShadows(),this.info.autoReset===!0&&this.info.reset(),(z&&A.hasRenderPass())===!1){let yt=R.opaque,ut=R.transmissive;if(w.setupLights(),N.isArrayCamera){let St=N.cameras;if(ut.length>0)for(let wt=0,Nt=St.length;wt<Nt;wt++){let Bt=St[wt];Uc(yt,ut,x,Bt)}ue&&it.render(x);for(let wt=0,Nt=St.length;wt<Nt;wt++){let Bt=St[wt];Nc(R,x,Bt,Bt.viewport)}}else ut.length>0&&Uc(yt,ut,x,N),ue&&it.render(x),Nc(R,x,N)}U!==null&&$===0&&(g.updateMultisampleRenderTarget(U),g.updateRenderTargetMipmap(U)),z&&A.end(D),x.isScene===!0&&x.onAfterRender(D,x,N),st.resetDefaultState(),L=-1,B=null,v.pop(),v.length>0?(w=v[v.length-1],g.setTextureUnits(w.state.textureUnits),de===!0&&xt.setGlobalState(D.clippingPlanes,w.state.camera)):w=null,I.pop(),I.length>0?R=I[I.length-1]:R=null,O!==null&&O.renderEnd()};function zo(x,N,G,z){if(x.visible===!1)return;if(x.layers.test(N.layers)){if(x.isGroup)G=x.renderOrder;else if(x.isLOD)x.autoUpdate===!0&&x.update(N);else if(x.isLightProbeGrid)w.pushLightProbeGrid(x);else if(x.isLight)w.pushLight(x),x.castShadow&&w.pushShadow(x);else if(x.isSprite){if(!x.frustumCulled||Rt.intersectsSprite(x)){z&&Vt.setFromMatrixPosition(x.matrixWorld).applyMatrix4(ee);let yt=nt.update(x),ut=x.material;ut.visible&&R.push(x,yt,ut,G,Vt.z,null)}}else if((x.isMesh||x.isLine||x.isPoints)&&(!x.frustumCulled||Rt.intersectsObject(x))){let yt=nt.update(x),ut=x.material;if(z&&(x.boundingSphere!==void 0?(x.boundingSphere===null&&x.computeBoundingSphere(),Vt.copy(x.boundingSphere.center)):(yt.boundingSphere===null&&yt.computeBoundingSphere(),Vt.copy(yt.boundingSphere.center)),Vt.applyMatrix4(x.matrixWorld).applyMatrix4(ee)),Array.isArray(ut)){let St=yt.groups;for(let wt=0,Nt=St.length;wt<Nt;wt++){let Bt=St[wt],At=ut[Bt.materialIndex];At&&At.visible&&R.push(x,yt,At,G,Vt.z,Bt)}}else ut.visible&&R.push(x,yt,ut,G,Vt.z,null)}}let ft=x.children;for(let yt=0,ut=ft.length;yt<ut;yt++)zo(ft[yt],N,G,z)}function Nc(x,N,G,z){let{opaque:V,transmissive:ft,transparent:yt}=x;w.setupLightsView(G),de===!0&&xt.setGlobalState(D.clippingPlanes,G),z&&ht.viewport(K.copy(z)),V.length>0&&br(V,N,G),ft.length>0&&br(ft,N,G),yt.length>0&&br(yt,N,G),ht.buffers.depth.setTest(!0),ht.buffers.depth.setMask(!0),ht.buffers.color.setMask(!0),ht.setPolygonOffset(!1)}function Uc(x,N,G,z){if((G.isScene===!0?G.overrideMaterial:null)!==null)return;if(w.state.transmissionRenderTarget[z.id]===void 0){let At=Wt.has("EXT_color_buffer_half_float")||Wt.has("EXT_color_buffer_float");w.state.transmissionRenderTarget[z.id]=new Ke(1,1,{generateMipmaps:!0,type:At?On:He,minFilter:xi,samples:Math.max(4,ae.samples),stencilBuffer:r,resolveDepthBuffer:!1,resolveStencilBuffer:!1,colorSpace:$t.workingColorSpace})}let ft=w.state.transmissionRenderTarget[z.id],yt=z.viewport||K;ft.setSize(yt.z*D.transmissionResolutionScale,yt.w*D.transmissionResolutionScale);let ut=D.getRenderTarget(),St=D.getActiveCubeFace(),wt=D.getActiveMipmapLevel();D.setRenderTarget(ft),D.getClearColor(bt),vt=D.getClearAlpha(),vt<1&&D.setClearColor(16777215,.5),D.clear(),ue&&it.render(G);let Nt=D.toneMapping;D.toneMapping=xn;let Bt=z.viewport;if(z.viewport!==void 0&&(z.viewport=void 0),w.setupLightsView(z),de===!0&&xt.setGlobalState(D.clippingPlanes,z),br(x,G,z),g.updateMultisampleRenderTarget(ft),g.updateRenderTargetMipmap(ft),Wt.has("WEBGL_multisampled_render_to_texture")===!1){let At=!1;for(let ie=0,ve=N.length;ie<ve;ie++){let ge=N[ie],{object:se,geometry:Le,material:_t,group:$e}=ge;if(_t.side===Un&&se.layers.test(z.layers)){let Yt=_t.side;_t.side=ze,_t.needsUpdate=!0,Fc(se,G,z,Le,_t,$e),_t.side=Yt,_t.needsUpdate=!0,At=!0}}At===!0&&(g.updateMultisampleRenderTarget(ft),g.updateRenderTargetMipmap(ft))}D.setRenderTarget(ut,St,wt),D.setClearColor(bt,vt),Bt!==void 0&&(z.viewport=Bt),D.toneMapping=Nt}function br(x,N,G){let z=N.isScene===!0?N.overrideMaterial:null;for(let V=0,ft=x.length;V<ft;V++){let yt=x[V],{object:ut,geometry:St,group:wt}=yt,Nt=yt.material;Nt.allowOverride===!0&&z!==null&&(Nt=z),ut.layers.test(G.layers)&&Fc(ut,N,G,St,Nt,wt)}}function Fc(x,N,G,z,V,ft){x.onBeforeRender(D,N,G,z,V,ft),x.modelViewMatrix.multiplyMatrices(G.matrixWorldInverse,x.matrixWorld),x.normalMatrix.getNormalMatrix(x.modelViewMatrix),V.onBeforeRender(D,N,G,z,x,ft),V.transparent===!0&&V.side===Un&&V.forceSinglePass===!1?(V.side=ze,V.needsUpdate=!0,D.renderBufferDirect(G,N,z,V,x,ft),V.side=Zn,V.needsUpdate=!0,D.renderBufferDirect(G,N,z,V,x,ft),V.side=Un):D.renderBufferDirect(G,N,z,V,x,ft),x.onAfterRender(D,N,G,z,V,ft)}function Sr(x,N,G){N.isScene!==!0&&(N=Me);let z=b.get(x),V=w.state.lights,ft=w.state.shadowsArray,yt=V.state.version,ut=lt.getParameters(x,V.state,ft,N,G,w.state.lightProbeGridArray),St=lt.getProgramCacheKey(ut),wt=z.programs;z.environment=x.isMeshStandardMaterial||x.isMeshLambertMaterial||x.isMeshPhongMaterial?N.environment:null,z.fog=N.fog;let Nt=x.isMeshStandardMaterial||x.isMeshLambertMaterial&&!x.envMap||x.isMeshPhongMaterial&&!x.envMap;z.envMap=F.get(x.envMap||z.environment,Nt),z.envMapRotation=z.environment!==null&&x.envMap===null?N.environmentRotation:x.envMapRotation,wt===void 0&&(x.addEventListener("dispose",_e),wt=new Map,z.programs=wt);let Bt=wt.get(St);if(Bt!==void 0){if(z.currentProgram===Bt&&z.lightsStateVersion===yt)return Bc(x,ut),Bt}else ut.uniforms=lt.getUniforms(x),O!==null&&x.isNodeMaterial&&O.build(x,G,ut),x.onBeforeCompile(ut,D),Bt=lt.acquireProgram(ut,St),wt.set(St,Bt),z.uniforms=ut.uniforms;let At=z.uniforms;return(!x.isShaderMaterial&&!x.isRawShaderMaterial||x.clipping===!0)&&(At.clippingPlanes=xt.uniform),Bc(x,ut),z.needsLights=Kd(x),z.lightsStateVersion=yt,z.needsLights&&(At.ambientLightColor.value=V.state.ambient,At.lightProbe.value=V.state.probe,At.directionalLights.value=V.state.directional,At.directionalLightShadows.value=V.state.directionalShadow,At.spotLights.value=V.state.spot,At.spotLightShadows.value=V.state.spotShadow,At.rectAreaLights.value=V.state.rectArea,At.ltc_1.value=V.state.rectAreaLTC1,At.ltc_2.value=V.state.rectAreaLTC2,At.pointLights.value=V.state.point,At.pointLightShadows.value=V.state.pointShadow,At.hemisphereLights.value=V.state.hemi,At.directionalShadowMatrix.value=V.state.directionalShadowMatrix,At.spotLightMatrix.value=V.state.spotLightMatrix,At.spotLightMap.value=V.state.spotLightMap,At.pointShadowMatrix.value=V.state.pointShadowMatrix),z.lightProbeGrid=w.state.lightProbeGridArray.length>0,z.currentProgram=Bt,z.uniformsList=null,Bt}function Oc(x){if(x.uniformsList===null){let N=x.currentProgram.getUniforms();x.uniformsList=Ss.seqWithValue(N.seq,x.uniforms)}return x.uniformsList}function Bc(x,N){let G=b.get(x);G.outputColorSpace=N.outputColorSpace,G.batching=N.batching,G.batchingColor=N.batchingColor,G.instancing=N.instancing,G.instancingColor=N.instancingColor,G.instancingMorph=N.instancingMorph,G.skinning=N.skinning,G.morphTargets=N.morphTargets,G.morphNormals=N.morphNormals,G.morphColors=N.morphColors,G.morphTargetsCount=N.morphTargetsCount,G.numClippingPlanes=N.numClippingPlanes,G.numIntersection=N.numClipIntersection,G.vertexAlphas=N.vertexAlphas,G.vertexTangents=N.vertexTangents,G.toneMapping=N.toneMapping}function Yd(x,N){if(x.length===0)return null;if(x.length===1)return x[0].texture!==null?x[0]:null;E.setFromMatrixPosition(N.matrixWorld);for(let G=0,z=x.length;G<z;G++){let V=x[G];if(V.texture!==null&&V.boundingBox.containsPoint(E))return V}return null}function Zd(x,N,G,z,V){N.isScene!==!0&&(N=Me),g.resetTextureUnits();let ft=N.fog,yt=z.isMeshStandardMaterial||z.isMeshLambertMaterial||z.isMeshPhongMaterial?N.environment:null,ut=U===null?D.outputColorSpace:U.isXRRenderTarget===!0?U.texture.colorSpace:$t.workingColorSpace,St=z.isMeshStandardMaterial||z.isMeshLambertMaterial&&!z.envMap||z.isMeshPhongMaterial&&!z.envMap,wt=F.get(z.envMap||yt,St),Nt=z.vertexColors===!0&&!!G.attributes.color&&G.attributes.color.itemSize===4,Bt=!!G.attributes.tangent&&(!!z.normalMap||z.anisotropy>0),At=!!G.morphAttributes.position,ie=!!G.morphAttributes.normal,ve=!!G.morphAttributes.color,ge=xn;z.toneMapped&&(U===null||U.isXRRenderTarget===!0)&&(ge=D.toneMapping);let se=G.morphAttributes.position||G.morphAttributes.normal||G.morphAttributes.color,Le=se!==void 0?se.length:0,_t=b.get(z),$e=w.state.lights;if(de===!0&&(Ht===!0||x!==B)){let oe=x===B&&z.id===L;xt.setState(z,x,oe)}let Yt=!1;z.version===_t.__version?(_t.needsLights&&_t.lightsStateVersion!==$e.state.version||_t.outputColorSpace!==ut||V.isBatchedMesh&&_t.batching===!1||!V.isBatchedMesh&&_t.batching===!0||V.isBatchedMesh&&_t.batchingColor===!0&&V.colorTexture===null||V.isBatchedMesh&&_t.batchingColor===!1&&V.colorTexture!==null||V.isInstancedMesh&&_t.instancing===!1||!V.isInstancedMesh&&_t.instancing===!0||V.isSkinnedMesh&&_t.skinning===!1||!V.isSkinnedMesh&&_t.skinning===!0||V.isInstancedMesh&&_t.instancingColor===!0&&V.instanceColor===null||V.isInstancedMesh&&_t.instancingColor===!1&&V.instanceColor!==null||V.isInstancedMesh&&_t.instancingMorph===!0&&V.morphTexture===null||V.isInstancedMesh&&_t.instancingMorph===!1&&V.morphTexture!==null||_t.envMap!==wt||z.fog===!0&&_t.fog!==ft||_t.numClippingPlanes!==void 0&&(_t.numClippingPlanes!==xt.numPlanes||_t.numIntersection!==xt.numIntersection)||_t.vertexAlphas!==Nt||_t.vertexTangents!==Bt||_t.morphTargets!==At||_t.morphNormals!==ie||_t.morphColors!==ve||_t.toneMapping!==ge||_t.morphTargetsCount!==Le||!!_t.lightProbeGrid!=w.state.lightProbeGridArray.length>0)&&(Yt=!0):(Yt=!0,_t.__version=z.version);let tn=_t.currentProgram;Yt===!0&&(tn=Sr(z,N,V),O&&z.isNodeMaterial&&O.onUpdateProgram(z,tn,_t));let wn=!1,ti=!1,Xi=!1,re=tn.getUniforms(),xe=_t.uniforms;if(ht.useProgram(tn.program)&&(wn=!0,ti=!0,Xi=!0),z.id!==L&&(L=z.id,ti=!0),_t.needsLights){let oe=Yd(w.state.lightProbeGridArray,V);_t.lightProbeGrid!==oe&&(_t.lightProbeGrid=oe,ti=!0)}if(wn||B!==x){ht.buffers.depth.getReversed()&&x.reversedDepth!==!0&&(x._reversedDepth=!0,x.updateProjectionMatrix()),re.setValue(P,"projectionMatrix",x.projectionMatrix),re.setValue(P,"viewMatrix",x.matrixWorldInverse);let ni=re.map.cameraPosition;ni!==void 0&&ni.setValue(P,ce.setFromMatrixPosition(x.matrixWorld)),ae.logarithmicDepthBuffer&&re.setValue(P,"logDepthBufFC",2/(Math.log(x.far+1)/Math.LN2)),(z.isMeshPhongMaterial||z.isMeshToonMaterial||z.isMeshLambertMaterial||z.isMeshBasicMaterial||z.isMeshStandardMaterial||z.isShaderMaterial)&&re.setValue(P,"isOrthographic",x.isOrthographicCamera===!0),B!==x&&(B=x,ti=!0,Xi=!0)}if(_t.needsLights&&($e.state.directionalShadowMap.length>0&&re.setValue(P,"directionalShadowMap",$e.state.directionalShadowMap,g),$e.state.spotShadowMap.length>0&&re.setValue(P,"spotShadowMap",$e.state.spotShadowMap,g),$e.state.pointShadowMap.length>0&&re.setValue(P,"pointShadowMap",$e.state.pointShadowMap,g)),V.isSkinnedMesh){re.setOptional(P,V,"bindMatrix"),re.setOptional(P,V,"bindMatrixInverse");let oe=V.skeleton;oe&&(oe.boneTexture===null&&oe.computeBoneTexture(),re.setValue(P,"boneTexture",oe.boneTexture,g))}V.isBatchedMesh&&(re.setOptional(P,V,"batchingTexture"),re.setValue(P,"batchingTexture",V._matricesTexture,g),re.setOptional(P,V,"batchingIdTexture"),re.setValue(P,"batchingIdTexture",V._indirectTexture,g),re.setOptional(P,V,"batchingColorTexture"),V._colorsTexture!==null&&re.setValue(P,"batchingColorTexture",V._colorsTexture,g));let ei=G.morphAttributes;if((ei.position!==void 0||ei.normal!==void 0||ei.color!==void 0)&&It.update(V,G,tn),(ti||_t.receiveShadow!==V.receiveShadow)&&(_t.receiveShadow=V.receiveShadow,re.setValue(P,"receiveShadow",V.receiveShadow)),(z.isMeshStandardMaterial||z.isMeshLambertMaterial||z.isMeshPhongMaterial)&&z.envMap===null&&N.environment!==null&&(xe.envMapIntensity.value=N.environmentIntensity),xe.dfgLUT!==void 0&&(xe.dfgLUT.value=z_()),ti){if(re.setValue(P,"toneMappingExposure",D.toneMappingExposure),_t.needsLights&&Jd(xe,Xi),ft&&z.fog===!0&&W.refreshFogUniforms(xe,ft),W.refreshMaterialUniforms(xe,z,Ut,Qt,w.state.transmissionRenderTarget[x.id]),_t.needsLights&&_t.lightProbeGrid){let oe=_t.lightProbeGrid;xe.probesSH.value=oe.texture,xe.probesMin.value.copy(oe.boundingBox.min),xe.probesMax.value.copy(oe.boundingBox.max),xe.probesResolution.value.copy(oe.resolution)}Ss.upload(P,Oc(_t),xe,g)}if(z.isShaderMaterial&&z.uniformsNeedUpdate===!0&&(Ss.upload(P,Oc(_t),xe,g),z.uniformsNeedUpdate=!1),z.isSpriteMaterial&&re.setValue(P,"center",V.center),re.setValue(P,"modelViewMatrix",V.modelViewMatrix),re.setValue(P,"normalMatrix",V.normalMatrix),re.setValue(P,"modelMatrix",V.matrixWorld),z.uniformsGroups!==void 0){let oe=z.uniformsGroups;for(let ni=0,qi=oe.length;ni<qi;ni++){let kc=oe[ni];X.update(kc,tn),X.bind(kc,tn)}}return tn}function Jd(x,N){x.ambientLightColor.needsUpdate=N,x.lightProbe.needsUpdate=N,x.directionalLights.needsUpdate=N,x.directionalLightShadows.needsUpdate=N,x.pointLights.needsUpdate=N,x.pointLightShadows.needsUpdate=N,x.spotLights.needsUpdate=N,x.spotLightShadows.needsUpdate=N,x.rectAreaLights.needsUpdate=N,x.hemisphereLights.needsUpdate=N}function Kd(x){return x.isMeshLambertMaterial||x.isMeshToonMaterial||x.isMeshPhongMaterial||x.isMeshStandardMaterial||x.isShadowMaterial||x.isShaderMaterial&&x.lights===!0}this.getActiveCubeFace=function(){return H},this.getActiveMipmapLevel=function(){return $},this.getRenderTarget=function(){return U},this.setRenderTargetTextures=function(x,N,G){let z=b.get(x);z.__autoAllocateDepthBuffer=x.resolveDepthBuffer===!1,z.__autoAllocateDepthBuffer===!1&&(z.__useRenderToTexture=!1),b.get(x.texture).__webglTexture=N,b.get(x.depthTexture).__webglTexture=z.__autoAllocateDepthBuffer?void 0:G,z.__hasExternalTextures=!0},this.setRenderTargetFramebuffer=function(x,N){let G=b.get(x);G.__webglFramebuffer=N,G.__useDefaultFramebuffer=N===void 0};let jd=P.createFramebuffer();this.setRenderTarget=function(x,N=0,G=0){U=x,H=N,$=G;let z=null,V=!1,ft=!1;if(x){let ut=b.get(x);if(ut.__useDefaultFramebuffer!==void 0){ht.bindFramebuffer(P.FRAMEBUFFER,ut.__webglFramebuffer),K.copy(x.viewport),tt.copy(x.scissor),ct=x.scissorTest,ht.viewport(K),ht.scissor(tt),ht.setScissorTest(ct),L=-1;return}else if(ut.__webglFramebuffer===void 0)g.setupRenderTarget(x);else if(ut.__hasExternalTextures)g.rebindTextures(x,b.get(x.texture).__webglTexture,b.get(x.depthTexture).__webglTexture);else if(x.depthBuffer){let Nt=x.depthTexture;if(ut.__boundDepthTexture!==Nt){if(Nt!==null&&b.has(Nt)&&(x.width!==Nt.image.width||x.height!==Nt.image.height))throw new Error("WebGLRenderTarget: Attached DepthTexture is initialized to the incorrect size.");g.setupDepthRenderbuffer(x)}}let St=x.texture;(St.isData3DTexture||St.isDataArrayTexture||St.isCompressedArrayTexture)&&(ft=!0);let wt=b.get(x).__webglFramebuffer;x.isWebGLCubeRenderTarget?(Array.isArray(wt[N])?z=wt[N][G]:z=wt[N],V=!0):x.samples>0&&g.useMultisampledRTT(x)===!1?z=b.get(x).__webglMultisampledFramebuffer:Array.isArray(wt)?z=wt[G]:z=wt,K.copy(x.viewport),tt.copy(x.scissor),ct=x.scissorTest}else K.copy(rt).multiplyScalar(Ut).floor(),tt.copy(Tt).multiplyScalar(Ut).floor(),ct=Lt;if(G!==0&&(z=jd),ht.bindFramebuffer(P.FRAMEBUFFER,z)&&ht.drawBuffers(x,z),ht.viewport(K),ht.scissor(tt),ht.setScissorTest(ct),V){let ut=b.get(x.texture);P.framebufferTexture2D(P.FRAMEBUFFER,P.COLOR_ATTACHMENT0,P.TEXTURE_CUBE_MAP_POSITIVE_X+N,ut.__webglTexture,G)}else if(ft){let ut=N;for(let St=0;St<x.textures.length;St++){let wt=b.get(x.textures[St]);P.framebufferTextureLayer(P.FRAMEBUFFER,P.COLOR_ATTACHMENT0+St,wt.__webglTexture,G,ut)}}else if(x!==null&&G!==0){let ut=b.get(x.texture);P.framebufferTexture2D(P.FRAMEBUFFER,P.COLOR_ATTACHMENT0,P.TEXTURE_2D,ut.__webglTexture,G)}L=-1},this.readRenderTargetPixels=function(x,N,G,z,V,ft,yt,ut=0){if(!(x&&x.isWebGLRenderTarget)){Ct("WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");return}let St=b.get(x).__webglFramebuffer;if(x.isWebGLCubeRenderTarget&&yt!==void 0&&(St=St[yt]),St){ht.bindFramebuffer(P.FRAMEBUFFER,St);try{let wt=x.textures[ut],Nt=wt.format,Bt=wt.type;if(x.textures.length>1&&P.readBuffer(P.COLOR_ATTACHMENT0+ut),!ae.textureFormatReadable(Nt)){Ct("WebGLRenderer.readRenderTargetPixels: renderTarget is not in RGBA or implementation defined format.");return}if(!ae.textureTypeReadable(Bt)){Ct("WebGLRenderer.readRenderTargetPixels: renderTarget is not in UnsignedByteType or implementation defined type.");return}N>=0&&N<=x.width-z&&G>=0&&G<=x.height-V&&P.readPixels(N,G,z,V,C.convert(Nt),C.convert(Bt),ft)}finally{let wt=U!==null?b.get(U).__webglFramebuffer:null;ht.bindFramebuffer(P.FRAMEBUFFER,wt)}}},this.readRenderTargetPixelsAsync=async function(x,N,G,z,V,ft,yt,ut=0){if(!(x&&x.isWebGLRenderTarget))throw new Error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");let St=b.get(x).__webglFramebuffer;if(x.isWebGLCubeRenderTarget&&yt!==void 0&&(St=St[yt]),St)if(N>=0&&N<=x.width-z&&G>=0&&G<=x.height-V){ht.bindFramebuffer(P.FRAMEBUFFER,St);let wt=x.textures[ut],Nt=wt.format,Bt=wt.type;if(x.textures.length>1&&P.readBuffer(P.COLOR_ATTACHMENT0+ut),!ae.textureFormatReadable(Nt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in RGBA or implementation defined format.");if(!ae.textureTypeReadable(Bt))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in UnsignedByteType or implementation defined type.");let At=P.createBuffer();P.bindBuffer(P.PIXEL_PACK_BUFFER,At),P.bufferData(P.PIXEL_PACK_BUFFER,ft.byteLength,P.STREAM_READ),P.readPixels(N,G,z,V,C.convert(Nt),C.convert(Bt),0);let ie=U!==null?b.get(U).__webglFramebuffer:null;ht.bindFramebuffer(P.FRAMEBUFFER,ie);let ve=P.fenceSync(P.SYNC_GPU_COMMANDS_COMPLETE,0);return P.flush(),await dd(P,ve,4),P.bindBuffer(P.PIXEL_PACK_BUFFER,At),P.getBufferSubData(P.PIXEL_PACK_BUFFER,0,ft),P.deleteBuffer(At),P.deleteSync(ve),ft}else throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: requested read bounds are out of range.")},this.copyFramebufferToTexture=function(x,N=null,G=0){let z=Math.pow(2,-G),V=Math.floor(x.image.width*z),ft=Math.floor(x.image.height*z),yt=N!==null?N.x:0,ut=N!==null?N.y:0;g.setTexture2D(x,0),P.copyTexSubImage2D(P.TEXTURE_2D,G,0,0,yt,ut,V,ft),ht.unbindTexture()};let Qd=P.createFramebuffer(),tu=P.createFramebuffer();this.copyTextureToTexture=function(x,N,G=null,z=null,V=0,ft=0){let yt,ut,St,wt,Nt,Bt,At,ie,ve,ge=x.isCompressedTexture?x.mipmaps[ft]:x.image;if(G!==null)yt=G.max.x-G.min.x,ut=G.max.y-G.min.y,St=G.isBox3?G.max.z-G.min.z:1,wt=G.min.x,Nt=G.min.y,Bt=G.isBox3?G.min.z:0;else{let xe=Math.pow(2,-V);yt=Math.floor(ge.width*xe),ut=Math.floor(ge.height*xe),x.isDataArrayTexture?St=ge.depth:x.isData3DTexture?St=Math.floor(ge.depth*xe):St=1,wt=0,Nt=0,Bt=0}z!==null?(At=z.x,ie=z.y,ve=z.z):(At=0,ie=0,ve=0);let se=C.convert(N.format),Le=C.convert(N.type),_t;N.isData3DTexture?(g.setTexture3D(N,0),_t=P.TEXTURE_3D):N.isDataArrayTexture||N.isCompressedArrayTexture?(g.setTexture2DArray(N,0),_t=P.TEXTURE_2D_ARRAY):(g.setTexture2D(N,0),_t=P.TEXTURE_2D),ht.activeTexture(P.TEXTURE0),ht.pixelStorei(P.UNPACK_FLIP_Y_WEBGL,N.flipY),ht.pixelStorei(P.UNPACK_PREMULTIPLY_ALPHA_WEBGL,N.premultiplyAlpha),ht.pixelStorei(P.UNPACK_ALIGNMENT,N.unpackAlignment);let $e=ht.getParameter(P.UNPACK_ROW_LENGTH),Yt=ht.getParameter(P.UNPACK_IMAGE_HEIGHT),tn=ht.getParameter(P.UNPACK_SKIP_PIXELS),wn=ht.getParameter(P.UNPACK_SKIP_ROWS),ti=ht.getParameter(P.UNPACK_SKIP_IMAGES);ht.pixelStorei(P.UNPACK_ROW_LENGTH,ge.width),ht.pixelStorei(P.UNPACK_IMAGE_HEIGHT,ge.height),ht.pixelStorei(P.UNPACK_SKIP_PIXELS,wt),ht.pixelStorei(P.UNPACK_SKIP_ROWS,Nt),ht.pixelStorei(P.UNPACK_SKIP_IMAGES,Bt);let Xi=x.isDataArrayTexture||x.isData3DTexture,re=N.isDataArrayTexture||N.isData3DTexture;if(x.isDepthTexture){let xe=b.get(x),ei=b.get(N),oe=b.get(xe.__renderTarget),ni=b.get(ei.__renderTarget);ht.bindFramebuffer(P.READ_FRAMEBUFFER,oe.__webglFramebuffer),ht.bindFramebuffer(P.DRAW_FRAMEBUFFER,ni.__webglFramebuffer);for(let qi=0;qi<St;qi++)Xi&&(P.framebufferTextureLayer(P.READ_FRAMEBUFFER,P.COLOR_ATTACHMENT0,b.get(x).__webglTexture,V,Bt+qi),P.framebufferTextureLayer(P.DRAW_FRAMEBUFFER,P.COLOR_ATTACHMENT0,b.get(N).__webglTexture,ft,ve+qi)),P.blitFramebuffer(wt,Nt,yt,ut,At,ie,yt,ut,P.DEPTH_BUFFER_BIT,P.NEAREST);ht.bindFramebuffer(P.READ_FRAMEBUFFER,null),ht.bindFramebuffer(P.DRAW_FRAMEBUFFER,null)}else if(V!==0||x.isRenderTargetTexture||b.has(x)){let xe=b.get(x),ei=b.get(N);ht.bindFramebuffer(P.READ_FRAMEBUFFER,Qd),ht.bindFramebuffer(P.DRAW_FRAMEBUFFER,tu);for(let oe=0;oe<St;oe++)Xi?P.framebufferTextureLayer(P.READ_FRAMEBUFFER,P.COLOR_ATTACHMENT0,xe.__webglTexture,V,Bt+oe):P.framebufferTexture2D(P.READ_FRAMEBUFFER,P.COLOR_ATTACHMENT0,P.TEXTURE_2D,xe.__webglTexture,V),re?P.framebufferTextureLayer(P.DRAW_FRAMEBUFFER,P.COLOR_ATTACHMENT0,ei.__webglTexture,ft,ve+oe):P.framebufferTexture2D(P.DRAW_FRAMEBUFFER,P.COLOR_ATTACHMENT0,P.TEXTURE_2D,ei.__webglTexture,ft),V!==0?P.blitFramebuffer(wt,Nt,yt,ut,At,ie,yt,ut,P.COLOR_BUFFER_BIT,P.NEAREST):re?P.copyTexSubImage3D(_t,ft,At,ie,ve+oe,wt,Nt,yt,ut):P.copyTexSubImage2D(_t,ft,At,ie,wt,Nt,yt,ut);ht.bindFramebuffer(P.READ_FRAMEBUFFER,null),ht.bindFramebuffer(P.DRAW_FRAMEBUFFER,null)}else re?x.isDataTexture||x.isData3DTexture?P.texSubImage3D(_t,ft,At,ie,ve,yt,ut,St,se,Le,ge.data):N.isCompressedArrayTexture?P.compressedTexSubImage3D(_t,ft,At,ie,ve,yt,ut,St,se,ge.data):P.texSubImage3D(_t,ft,At,ie,ve,yt,ut,St,se,Le,ge):x.isDataTexture?P.texSubImage2D(P.TEXTURE_2D,ft,At,ie,yt,ut,se,Le,ge.data):x.isCompressedTexture?P.compressedTexSubImage2D(P.TEXTURE_2D,ft,At,ie,ge.width,ge.height,se,ge.data):P.texSubImage2D(P.TEXTURE_2D,ft,At,ie,yt,ut,se,Le,ge);ht.pixelStorei(P.UNPACK_ROW_LENGTH,$e),ht.pixelStorei(P.UNPACK_IMAGE_HEIGHT,Yt),ht.pixelStorei(P.UNPACK_SKIP_PIXELS,tn),ht.pixelStorei(P.UNPACK_SKIP_ROWS,wn),ht.pixelStorei(P.UNPACK_SKIP_IMAGES,ti),ft===0&&N.generateMipmaps&&P.generateMipmap(_t),ht.unbindTexture()},this.initRenderTarget=function(x){b.get(x).__webglFramebuffer===void 0&&g.setupRenderTarget(x)},this.initTexture=function(x){x.isCubeTexture?g.setTextureCube(x,0):x.isData3DTexture?g.setTexture3D(x,0):x.isDataArrayTexture||x.isCompressedArrayTexture?g.setTexture2DArray(x,0):g.setTexture2D(x,0),ht.unbindTexture()},this.resetState=function(){H=0,$=0,U=null,ht.reset(),st.reset()},typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}get coordinateSystem(){return vn}get outputColorSpace(){return this._outputColorSpace}set outputColorSpace(t){this._outputColorSpace=t;let e=this.getContext();e.drawingBufferColorSpace=$t._getDrawingBufferColorSpace(t),e.unpackColorSpace=$t._getUnpackColorSpace()}};var $d=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Ic=class extends kt{constructor(){super(...arguments),this._ready=!1}static{this.styles=qt`
    :host { display: block; height: 100%; position: relative; }
    .placeholder {
      display: flex; align-items: center; justify-content: center;
      height: 100%; color: var(--muted); font-size: 14px; flex-direction: column; gap: 8px;
    }
    canvas { display: block; width: 100%; height: 100%; }
  `}connectedCallback(){super.connectedCallback(),this._ready=!0}disconnectedCallback(){super.disconnectedCallback(),this._frameId&&cancelAnimationFrame(this._frameId),this._renderer?.dispose()}firstUpdated(){this._initThree()}_initThree(){let t=this.shadowRoot?.querySelector(".three-container");if(!t)return;let e=t.getBoundingClientRect();if(e.width===0||e.height===0)return;let n=new qs;n.background=new zt("#09090b"),this._scene=n;let s=new Ue(60,e.width/e.height,.1,100);s.position.set(5,5,5),s.lookAt(0,0,0),this._camera=s;let r=new Uo({antialias:!0});r.setSize(e.width,e.height),r.setPixelRatio(devicePixelRatio),t.appendChild(r.domElement),this._renderer=r,n.add(new rr(16777215,.4));let a=new sr(16777215,.8);a.position.set(5,10,7),n.add(a);let o=new ke(new Gi(20,20),new gs({color:"#0b1210"}));o.rotation.x=-Math.PI/2,o.position.y=-.01,n.add(o),n.add(new ar(20,20,"#1a3324","#152820")),this._addBox([0,.5,0],"#22c55e","Inverter"),this._addBox([3,.5,2],"#eab308","Solar Panel"),this._addBox([-2,.5,3],"#3b82f6","Gateway");let l=!1,c=0,d=0,u=r.domElement;u.addEventListener("mousedown",p=>{l=!0,c=p.clientX,d=p.clientY}),u.addEventListener("mouseup",()=>{l=!1}),u.addEventListener("mousemove",p=>{if(!l)return;let _=p.clientX-c,y=p.clientY-d;c=p.clientX,d=p.clientY;let m=Math.sqrt(s.position.x**2+s.position.z**2),S=Math.atan2(s.position.z,s.position.x)-_*.01;s.position.x=m*Math.cos(S),s.position.z=m*Math.sin(S),s.position.y=Math.max(.5,s.position.y-y*.05),s.lookAt(0,0,0)}),u.addEventListener("wheel",p=>{p.preventDefault();let _=p.deltaY>0?1.1:.9;s.position.multiplyScalar(_)});let h=()=>{this._frameId=requestAnimationFrame(h),r.render(n,s)};h()}_addBox(t,e,n){if(!this._scene)return;let s=new pi(1,1,1),r=new gs({color:e}),a=new ke(s,r);a.position.set(...t),a.castShadow=!0,this._scene.add(a)}render(){return Z`
      <div class="three-container" style="width:100%;height:100%">
        ${this._ready?"":Z`<div class="placeholder">Loading 3D scene...</div>`}
      </div>
    `}};$d([et()],Ic.prototype,"_ready",void 0);Ic=$d([Zt("vlt-twin-view")],Ic);var Xd=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Pc=class extends kt{constructor(){super(...arguments),this._info=null}static{this.styles=qt`
    :host { display: block; padding: 8px; overflow-y: auto; height: 100%; box-sizing: border-box; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; }
    .card {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 20px;
    }
    .card.wide { grid-column: span 2; }
    .label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
    .value { font-size: 28px; font-weight: 700; color: var(--fg); }
    .value.ok { color: var(--ok); }
    .value.warn { color: var(--warn); }
    .value.crit { color: var(--crit); }
    .sub { font-size: 13px; color: var(--muted); margin-top: 4px; }
    .bar { margin-top: 8px; height: 6px; border-radius: 3px; background: var(--border); }
    .bar-fill { height: 100%; border-radius: 3px; transition: width 0.3s; }
    .process-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; }
    .process-card { text-align: center; }
    .process-value { font-size: 24px; font-weight: 700; color: #22c55e; }
    .process-label { font-size: 11px; color: var(--muted); margin-top: 4px; }
  `}connectedCallback(){super.connectedCallback(),this._load(),this._interval=setInterval(()=>this._load(),5e3)}disconnectedCallback(){super.disconnectedCallback(),this._interval&&clearInterval(this._interval)}async _load(){try{this._info=await Kt("/system")}catch{this._info=null}}_barColor(t){return t>90?"var(--crit)":t>70?"var(--warn)":"var(--ok)"}_fmtUptime(t){let e=Math.floor(t/86400),n=Math.floor(t%86400/3600),s=Math.floor(t%3600/60);return`${e}d ${n}h ${s}m`}render(){if(!this._info)return Z`<div style="display:flex;align-items:center;justify-content:center;height:300px;color:var(--muted)">Connecting to server...</div>`;let t=this._info;return Z`
      <div class="grid">
        <div class="card">
          <div class="label">Host</div>
          <div class="value">${t.hostname}</div>
          <div class="sub">${t.os} — up ${this._fmtUptime(t.uptime_secs)}</div>
        </div>
        <div class="card">
          <div class="label">CPU Usage</div>
          <div class="value ${t.cpu_usage>80?"crit":t.cpu_usage>50?"warn":"ok"}">${t.cpu_usage.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(t.cpu_usage)};width:${Math.min(t.cpu_usage,100)}%"></div></div>
        </div>
        <div class="card">
          <div class="label">Memory</div>
          <div class="value">${t.memory_used_mb.toFixed(0)} <span style="font-size:14px;color:var(--muted);font-weight:400">/ ${t.memory_total_mb.toFixed(0)} MB</span></div>
          <div class="sub">${t.memory_percent.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(t.memory_percent)};width:${Math.min(t.memory_percent,100)}%"></div></div>
        </div>
        <div class="card">
          <div class="label">Disk</div>
          <div class="value">${t.disk_used_gb.toFixed(1)} <span style="font-size:14px;color:var(--muted);font-weight:400">/ ${t.disk_total_gb.toFixed(1)} GB</span></div>
          <div class="sub">${t.disk_percent.toFixed(1)}%</div>
          <div class="bar"><div class="bar-fill" style="background:${this._barColor(t.disk_percent)};width:${Math.min(t.disk_percent,100)}%"></div></div>
        </div>
        <div class="card wide" style="background:linear-gradient(135deg,#1a1a2e 0%,#16213e 100%);border-color:#22c55e">
          <div class="label" style="color:#22c55e;font-weight:700;letter-spacing:0.08em">Rust Service (PID ${t.process.pid})</div>
          <div class="process-grid">
            <div class="process-card">
              <div class="process-value">${t.process.memory_mb.toFixed(1)} MB</div>
              <div class="process-label">RSS Memory</div>
            </div>
            <div class="process-card">
              <div class="process-value">${t.process.cpu_percent.toFixed(1)}%</div>
              <div class="process-label">CPU Usage</div>
            </div>
            <div class="process-card">
              <div class="process-value">${this._fmtUptime(t.process.uptime_secs)}</div>
              <div class="process-label">Uptime</div>
            </div>
            <div class="process-card">
              <div class="process-value">${t.process.binary_size_mb.toFixed(1)} MB</div>
              <div class="process-label">Binary Size</div>
            </div>
          </div>
        </div>
      </div>
    `}};Xd([et()],Pc.prototype,"_info",void 0);Pc=Xd([Zt("vlt-system-view")],Pc);var Bo=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},yr=class extends kt{constructor(){super(...arguments),this._tab="general",this._form={},this._srcConfig={}}static{this.styles=qt`
    :host {
      position: fixed; inset: 0; background: rgba(0,0,0,.6);
      display: flex; align-items: center; justify-content: center; z-index: 60;
    }
    .modal {
      background: var(--card); border: 1px solid var(--border); border-radius: 10px;
      width: 480px; max-height: 80vh; display: flex; flex-direction: column;
    }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 12px 16px; border-bottom: 1px solid var(--border);
    }
    .header span { font-size: 13px; font-weight: 600; color: var(--fg); }
    .close { background: none; border: none; color: var(--muted); cursor: pointer; font-size: 18; line-height: 1; }
    .tabs { display: flex; border-bottom: 1px solid var(--border); }
    .tabs button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      background: transparent; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em;
      border-bottom: 2px solid transparent;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .body { overflow: auto; flex: 1; padding: 16px; }
    .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .field { display: flex; flex-direction: column; gap: 4px; }
    .field.wide { grid-column: 1 / -1; }
    .field label { font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    .field input, .field select {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
      padding: 6px 8px; color: var(--fg); font-size: 13px; font-family: inherit; outline: none; width: 100%; box-sizing: border-box;
    }
    .field input:focus, .field select:focus { border-color: var(--border-light); }
    .footer { display: flex; gap: 8px; padding: 0 16px 16px; }
    .footer button {
      padding: 6px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; font-family: inherit;
    }
    .save { border: 1px solid var(--cyan); background: transparent; color: var(--cyan); }
    .delete { border: 1px solid var(--crit); background: transparent; color: var(--crit); }
  `}connectedCallback(){if(super.connectedCallback(),this.sensor){this._form={...this.sensor};try{this._srcConfig=JSON.parse(this.sensor.source_config||"{}")}catch{this._srcConfig={}}}}_save(){if(!this.sensor)return;let t={...this.sensor,...this._form,source_config:JSON.stringify(this._srcConfig)};this.dispatchEvent(new CustomEvent("save",{detail:t}))}_delete(){this.sensor&&this.dispatchEvent(new CustomEvent("delete",{detail:this.sensor.id}))}_close(){this.dispatchEvent(new CustomEvent("close"))}render(){if(!this.sensor)return Z``;let t=this._form,e=(t.chart_type||"sparkline")==="image",n=(t.chart_type||"sparkline")==="text",s=e||n,r=si[t.source_type||"none"];return Z`
      <div @click="${this._close}">
        <div class="modal" @click="${a=>a.stopPropagation()}">
          <div class="header">
            <span>Edit: ${this.sensor.name}</span>
            <div style="display:flex;align-items:center;gap:8px">
              <span style="font-size:10px;padding:2px 8px;border-radius:4px;background:${r?.color??"var(--muted)"}18;color:${r?.color??"var(--muted)"};font-weight:700">${r?.tag??t.source_type} — ${r?.label??t.source_type}</span>
              <button class="close" @click="${this._close}">&times;</button>
            </div>
          </div>
          <div class="tabs">
            <button class="${this._tab==="general"?"active":""}" @click="${()=>this._tab="general"}">General</button>
            <button class="${this._tab==="source"?"active":""}" @click="${()=>this._tab="source"}">Source</button>
          </div>
          <div class="body">
            ${this._tab==="general"?Z`
              <div class="grid2">
                <div class="field"><label>Name</label><input .value="${t.name||""}" @input="${a=>this._form={...this._form,name:a.target.value}}"></div>
                <div class="field"><label>Key</label><input .value="${t.key||""}" @input="${a=>this._form={...this._form,key:a.target.value}}"></div>
                ${s?"":Z`
                  <div class="field"><label>Unit</label><input .value="${t.unit||""}" @input="${a=>this._form={...this._form,unit:a.target.value}}"></div>
                  <div class="field"><label>Min</label><input type="number" step="any" .value="${String(t.min_val??0)}" @input="${a=>this._form={...this._form,min_val:parseFloat(a.target.value)||0}}"></div>
                  <div class="field"><label>Max</label><input type="number" step="any" .value="${String(t.max_val??0)}" @input="${a=>this._form={...this._form,max_val:parseFloat(a.target.value)||0}}"></div>
                  <div class="field"><label>Warn</label><input type="number" step="any" .value="${String(t.warn_val??0)}" @input="${a=>this._form={...this._form,warn_val:parseFloat(a.target.value)||0}}"></div>
                  <div class="field"><label>Crit</label><input type="number" step="any" .value="${String(t.crit_val??0)}" @input="${a=>this._form={...this._form,crit_val:parseFloat(a.target.value)||0}}"></div>
                  <div class="field"><label>Base</label><input type="number" step="any" .value="${String(t.base_val??0)}" @input="${a=>this._form={...this._form,base_val:parseFloat(a.target.value)||0}}"></div>
                  <div class="field"><label>Decimals</label><input type="number" .value="${String(t.decimals??0)}" @input="${a=>this._form={...this._form,decimals:parseInt(a.target.value)||0}}"></div>
                `}
                <div class="field"><label>Color</label><input type="color" .value="${t.color||"#22c55e"}" @input="${a=>this._form={...this._form,color:a.target.value}}"></div>
                <div class="field"><label>Chart Type</label>
                  <select .value="${t.chart_type||"sparkline"}" @change="${a=>this._form={...this._form,chart_type:a.target.value}}">
                    ${ih.map(a=>Z`<option value="${a}">${a}</option>`)}
                  </select>
                </div>
              </div>
            `:Z`
              <div class="field wide"><label>Data Source</label>
                <select .value="${t.source_type||"none"}" @change="${a=>{this._form={...this._form,source_type:a.target.value},this._srcConfig={}}}">
                  <optgroup label="Real Data"><option value="modbus">Modbus (Live Device)</option></optgroup>
                  <optgroup label="Simulate">${sh.map(a=>Z`<option value="${a}">${si[a]?.label??a}</option>`)}</optgroup>
                  <optgroup label="Other"><option value="api">API (HTTP Poll)</option><option value="none">None (Static)</option></optgroup>
                </select>
              </div>
              <div class="grid2" style="margin-top:8px">
                ${t.source_type==="api"?Z`
                  <div class="field wide"><label>URL</label><input .value="${this._srcConfig.url||""}" @input="${a=>this._srcConfig={...this._srcConfig,url:a.target.value}}"></div>
                `:""}
                ${t.source_type==="modbus"?Z`
                  <div class="field"><label>Device ID</label><input .value="${this._srcConfig.modbus_device_id||""}" @input="${a=>this._srcConfig={...this._srcConfig,modbus_device_id:a.target.value}}"></div>
                  <div class="field"><label>Register</label><input .value="${this._srcConfig.modbus_register_name||""}" @input="${a=>this._srcConfig={...this._srcConfig,modbus_register_name:a.target.value}}"></div>
                `:""}
              </div>
            `}
          </div>
          <div class="footer">
            <button class="save" @click="${this._save}">Save</button>
            <button class="delete" @click="${this._delete}">Delete</button>
          </div>
        </div>
      </div>
    `}};Bo([et()],yr.prototype,"_tab",void 0);Bo([et()],yr.prototype,"_form",void 0);Bo([et()],yr.prototype,"_srcConfig",void 0);yr=Bo([Zt("vlt-edit-panel")],yr);var zn=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},Sn=class extends kt{constructor(){super(...arguments),this.isAdmin=!1,this._tab="sources",this._sensors=[],this._editingId=null,this._filter="",this._modbusDevices=[],this._modbusFilter="",this._users=[],this._userFilter="",this._onKey=t=>{t.key==="Escape"&&this._close()}}static{this.styles=qt`
    :host {
      position: fixed; inset: 0; background: var(--bg); z-index: 50;
      display: flex; flex-direction: column; padding: 24px;
    }
    .container { max-width: 1200px; width: 100%; margin: 0 auto; display: flex; flex-direction: column; height: 100%; }
    .header { display: flex; align-items: center; gap: 12px; margin-bottom: 0; flex-shrink: 0; }
    .header h1 { font-size: 20px; font-weight: 700; margin: 0; color: var(--fg); }
    .header button {
      display: flex; align-items: center; gap: 6px;
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--border);
      background: var(--card); color: var(--fg); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .tabs { display: flex; border-bottom: 1px solid var(--border); margin-bottom: 16px; }
    .tabs button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      background: transparent; color: var(--muted); border-bottom: 2px solid transparent;
      text-transform: uppercase; letter-spacing: 0.04em;
    }
    .tabs button.active { border-bottom-color: var(--cyan); color: var(--cyan); background: rgba(34,197,94,.1); }
    .table-wrap { flex: 1; overflow: auto; border: 1px solid var(--border); border-radius: 8px; }
    table { width: 100%; border-collapse: collapse; font-size: 13px; }
    th { text-align: left; padding: 10px 12px; background: var(--card); font-size: 11px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; border-bottom: 1px solid var(--border); position: sticky; top: 0; }
    td { padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--fg); }
    tr:hover td { background: rgba(255,255,255,0.02); }
    .tag { font-size: 9px; padding: 1px 5px; border-radius: 3px; font-weight: 700; letter-spacing: 0.03em; }
    .actions { display: flex; gap: 6px; }
    .actions button {
      padding: 3px 8px; border-radius: 4px; border: 1px solid var(--border);
      background: transparent; color: var(--fg); cursor: pointer; font-size: 11px; font-family: inherit;
    }
    .actions button.edit { color: var(--cyan); border-color: var(--cyan); }
    .actions button.del { color: var(--crit); border-color: var(--crit); }
    .add-btn {
      padding: 7px 14px; border-radius: 6px; border: 1px solid var(--green);
      background: transparent; color: var(--green); cursor: pointer; font-family: inherit; font-size: 13px;
    }
    .toolbar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-shrink: 0; }
    .toolbar span { color: var(--muted); font-size: 13px; }
    .toolbar input {
      background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px;
      color: var(--fg); font-size: 13px; font-family: inherit; width: 200px;
    }
    .empty { padding: 40px; text-align: center; color: var(--muted); }
  `}connectedCallback(){super.connectedCallback(),this._loadSensors(),this._loadModbus(),this._loadUsers(),window.addEventListener("keydown",this._onKey)}disconnectedCallback(){super.disconnectedCallback(),window.removeEventListener("keydown",this._onKey)}async _loadSensors(){try{this._sensors=await Kt("/data-sources")}catch{this._sensors=[]}}async _loadModbus(){try{this._modbusDevices=await Kt("/modbus-devices")}catch{this._modbusDevices=[]}}async _loadUsers(){try{this._users=await Kt("/users")}catch{this._users=[]}}async _saveSensor(t){let e=t.detail;try{await Ie(`/api/data-sources/${e.id}`,{method:"PUT",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),this._editingId=null,this._loadSensors()}catch{}}async _deleteSensor(t){if(confirm("Delete this sensor?"))try{await Ie(`/api/data-sources/${t}`,{method:"DELETE"}),this._editingId=null,this._loadSensors()}catch{}}async _duplicateSensor(t){try{await Ie("/api/data-sources",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:t.key+"_copy",name:t.name+" (Copy)",unit:t.unit,min_val:t.min_val,max_val:t.max_val,warn_val:t.warn_val,crit_val:t.crit_val,base_val:t.base_val,decimals:t.decimals,source_type:t.source_type,color:t.color,chart_type:t.chart_type,source_config:t.source_config})}),this._loadSensors()}catch{}}async _deleteUser(t){if(confirm("Delete this user?"))try{await Ie(`/api/users/${t}`,{method:"DELETE"}),this._loadUsers()}catch{}}async _deleteModbus(t){if(confirm("Delete this Modbus device?"))try{await Ie(`/api/modbus-devices/${t}`,{method:"DELETE"}),this._loadModbus()}catch{}}_close(){this.dispatchEvent(new CustomEvent("close"))}_sourcesTab(){let t=this._sensors.filter(e=>e.name.toLowerCase().includes(this._filter.toLowerCase())||e.key.toLowerCase().includes(this._filter.toLowerCase()));return Z`
      <div class="toolbar">
        <span>${this._sensors.length} sources</span>
        <div style="display:flex;gap:8px">
          <input placeholder="Search..." .value="${this._filter}" @input="${e=>this._filter=e.target.value}" />
          <button class="add-btn" @click="${()=>this._duplicateSensor({...this._sensors[0],id:0,key:"new",name:"New Sensor"})}">+ Create Source</button>
        </div>
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr>
            <th>Name</th><th>Key</th><th>Type</th><th>Unit</th><th>Source</th><th>Chart</th><th>Actions</th>
          </tr></thead>
          <tbody>
            ${t.map(e=>{let n=si[e.source_type];return Z`
                <tr>
                  <td><strong>${e.name}</strong></td>
                  <td style="font-family:'JetBrains Mono',monospace;font-size:12px">${e.key}</td>
                  <td>${e.unit||"\u2014"}</td>
                  <td><span class="tag" style="background:${n?.color??"var(--muted)"}18;color:${n?.color??"var(--muted)"}">${n?.tag??e.source_type}</span></td>
                  <td><span class="tag" style="background:rgba(34,197,94,.1);color:var(--cyan)">${e.source_type}</span></td>
                  <td><span class="tag" style="background:rgba(139,92,246,.1);color:#a78bfa">${e.chart_type||"sparkline"}</span></td>
                  <td>
                    <div class="actions">
                      <button class="edit" @click="${()=>this._editingId=e.id}">Edit</button>
                      <button @click="${()=>this._duplicateSensor(e)}">Dup</button>
                      <button class="del" @click="${()=>this._deleteSensor(e.id)}">Del</button>
                    </div>
                  </td>
                </tr>
              `})}
          </tbody>
        </table>
      </div>
    `}_modbusTab(){let t=this._modbusDevices.filter(e=>(e.name||e.id).toLowerCase().includes(this._modbusFilter.toLowerCase())||e.ip.toLowerCase().includes(this._modbusFilter.toLowerCase()));return Z`
      <div class="toolbar">
        <span>${this._modbusDevices.length} devices</span>
        <input placeholder="Search..." .value="${this._modbusFilter}" @input="${e=>this._modbusFilter=e.target.value}" />
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr>
            <th>Name</th><th>IP</th><th>Port</th><th>Unit</th><th>Interval</th><th>Endian</th><th>Status</th><th>Actions</th>
          </tr></thead>
          <tbody>
            ${t.map(e=>Z`
              <tr>
                <td>${e.name||e.id}</td>
                <td style="font-family:'JetBrains Mono',monospace;font-size:12px">${e.ip}</td>
                <td>${e.port}</td>
                <td>${e.unit_id}</td>
                <td>${e.interval_ms}ms</td>
                <td><span class="tag" style="background:rgba(139,92,246,.1);color:#a78bfa">${e.endianness}</span></td>
                <td><span class="tag" style="background:${e.enabled?"rgba(34,197,94,.1)":"rgba(239,68,68,.1)"};color:${e.enabled?"var(--cyan)":"var(--crit)"}">${e.enabled?"enabled":"disabled"}</span></td>
                <td>
                  <div class="actions">
                    <button class="del" @click="${()=>this._deleteModbus(e.id)}">Del</button>
                  </div>
                </td>
              </tr>
            `)}
          </tbody>
        </table>
      </div>
    `}_usersTab(){let t=this._users.filter(e=>e.username.toLowerCase().includes(this._userFilter.toLowerCase()));return Z`
      <div class="toolbar">
        <span>${this._users.length} users</span>
        <input placeholder="Search..." .value="${this._userFilter}" @input="${e=>this._userFilter=e.target.value}" />
      </div>
      <div class="table-wrap">
        <table>
          <thead><tr><th>Username</th><th>Role</th><th>Actions</th></tr></thead>
          <tbody>
            ${t.map(e=>Z`
              <tr>
                <td><strong>${e.username}</strong></td>
                <td><span class="tag" style="background:${e.role==="admin"?"rgba(168,85,247,.15)":"rgba(34,197,94,.1)"};color:${e.role==="admin"?"#a855f7":"var(--green)"}">${e.role}</span></td>
                <td>
                  <div class="actions">
                    <button class="del" @click="${()=>this._deleteUser(e.id)}">Del</button>
                  </div>
                </td>
              </tr>
            `)}
          </tbody>
        </table>
      </div>
    `}render(){let t=[{key:"sources",label:"Sources"},{key:"modbus",label:"Modbus"}];return this.isAdmin&&t.push({key:"users",label:"Users"}),Z`
      <div class="container">
        <div class="header">
          <button @click="${this._close}">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
            Back
          </button>
          <h1>Data Manager</h1>
        </div>
        <div class="tabs">
          ${t.map(e=>Z`
            <button class="${this._tab===e.key?"active":""}" @click="${()=>this._tab=e.key}">${e.label}</button>
          `)}
        </div>
        ${this._tab==="sources"?this._sourcesTab():""}
        ${this._tab==="modbus"?this._modbusTab():""}
        ${this._tab==="users"?this._usersTab():""}
      </div>
      ${this._editingId!=null?Z`
        <vlt-edit-panel
          .sensor="${this._sensors.find(e=>e.id===this._editingId)}"
          @save="${this._saveSensor}"
          @delete="${e=>this._deleteSensor(e.detail)}"
          @close="${()=>this._editingId=null}"
        ></vlt-edit-panel>
      `:""}
    `}};zn([le({type:Boolean})],Sn.prototype,"isAdmin",void 0);zn([et()],Sn.prototype,"_tab",void 0);zn([et()],Sn.prototype,"_sensors",void 0);zn([et()],Sn.prototype,"_editingId",void 0);zn([et()],Sn.prototype,"_filter",void 0);zn([et()],Sn.prototype,"_modbusDevices",void 0);zn([et()],Sn.prototype,"_modbusFilter",void 0);zn([et()],Sn.prototype,"_users",void 0);zn([et()],Sn.prototype,"_userFilter",void 0);Sn=zn([Zt("vlt-data-manager")],Sn);var Se=function(i,t,e,n){var s=arguments.length,r=s<3?t:n===null?n=Object.getOwnPropertyDescriptor(t,e):n,a;if(typeof Reflect=="object"&&typeof Reflect.decorate=="function")r=Reflect.decorate(i,t,e,n);else for(var o=i.length-1;o>=0;o--)(a=i[o])&&(r=(s<3?a(r):s>3?a(t,e,r):a(t,e))||r);return s>3&&r&&Object.defineProperty(t,e,r),r},ye=class extends kt{constructor(){super(...arguments),this._authed=!1,this._authLoading=!0,this._view="grid",this._activeScreen="all",this._screens=[],this._sseConnected=!1,this._showAlarmPanel=!1,this._selectedSensorId=null,this._showSettings=!1,this._showDataManager=!1,this._allSensors=[],this._currentValues={},this._alerts=0,this._branding={title:"Voltrus Lite",subtitle:"Industrial monitoring",logo:""},this._version="",this._editMode=!1,this._dialog=null,this._authRole="user",this._systemInfo={},this._onAuth=()=>{let t=An.state;this._authed=t.authed,this._authLoading=t.loading,this._authRole=t.role,t.authed&&(this._loadScreens(),this._loadAllSensors(),this._loadBranding(),this._loadShortcuts())},this._onKeyDown=t=>{if(t.target instanceof HTMLInputElement||t.target instanceof HTMLTextAreaElement||t.target?.isContentEditable)return;let e=t.key.toLowerCase(),n=En.navShortcuts;for(let[r,a]of Object.entries(n))if(a.toLowerCase()===e){t.preventDefault(),["grid","canvas","scada","twin","system"].includes(r)&&this._setView(r),r==="alarms"&&(this._showAlarmPanel=!0),r==="settings"&&(this._showSettings=!0),r==="data"&&(this._showDataManager=!0);return}let s=En.shortcuts;for(let[r,a]of Object.entries(s))if(a.toLowerCase()===e){t.preventDefault(),this._selectedSensorId=Number(r);return}}}static{this.styles=qt`
    :host { display: flex; flex-direction: column; height: 100vh; }
    .header {
      display: flex; align-items: center; justify-content: space-between;
      padding: 12px 20px; flex-shrink: 0; flex-wrap: wrap; gap: 16px;
    }
    .brand { display: flex; align-items: center; gap: 12px; }
    .brand img { width: 32px; height: 32px; border-radius: 6px; object-fit: contain; }
    .brand h1 { font-size: 20px; font-weight: 700; margin: 0; color: var(--fg); }
    .brand span { color: var(--muted); font-weight: 400; font-size: 14px; }
    .toolbar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
    .toolbar button {
      font-family: Inter, sans-serif; font-size: 13px; font-weight: 500; padding: 7px 14px;
      border-radius: 6px; border: 1px solid var(--border); background: var(--card);
      color: var(--fg); cursor: pointer; display: flex; align-items: center; gap: 6px;
    }
    .toolbar button.active { background: rgba(34,197,94,.1); color: var(--cyan); border-color: var(--cyan); }
    .screens {
      display: flex; align-items: center; gap: 0; margin: 0 20px 20px;
      border-bottom: 1px solid var(--border);
    }
    .screens button {
      padding: 8px 16px; font-size: 12px; font-weight: 600; cursor: pointer; border: none;
      border-bottom: 2px solid transparent; color: var(--muted); background: transparent;
      text-transform: uppercase; letter-spacing: 0.04em;
      display: flex; align-items: center; gap: 6px;
    }
    .screens button.active { border-bottom-color: var(--cyan); color: var(--cyan); }
    .screens button .actions { display: none; gap: 4px; margin-left: 4px; }
    .screens button:hover .actions, .screens button.active .actions { display: flex; }
    .screens button .actions span { font-size: 10px; opacity: 0.6; cursor: pointer; }
    .screens button .actions span:hover { opacity: 1; color: var(--cyan); }
    .screen-add { padding: 8px 12px; font-size: 16px; font-weight: 700; cursor: pointer; border: none; background: transparent; color: var(--muted); }
    .screen-add:hover { color: var(--cyan); }
    main { flex: 1; overflow: hidden; position: relative; }
    main > * { position: absolute; inset: 0; }
    .loading {
      height: 100vh; display: flex; align-items: center; justify-content: center;
      background: var(--bg); color: var(--muted); font-family: Inter, sans-serif;
    }
    .sse-badge {
      font-family: 'JetBrains Mono', monospace; font-size: 11px;
      background: var(--card); border: 1px solid var(--border); padding: 4px 8px;
      border-radius: 4px; display: flex; align-items: center; gap: 4px;
    }
    .sse-dot { width: 6px; height: 6px; border-radius: 50%; }
    .footer {
      display: flex; align-items: center; justify-content: space-between;
      padding: 8px 20px; border-top: 1px solid var(--border);
      font-size: 11px; color: var(--muted); flex-shrink: 0;
      font-family: 'JetBrains Mono', monospace;
    }
    .footer .left { display: flex; align-items: center; gap: 12px; }
    .footer .right { display: flex; align-items: center; gap: 12px; }
    .footer .dot { width: 6px; height: 6px; border-radius: 50%; display: inline-block; margin-right: 4px; }
  `}_readUrlParams(){let t=new URLSearchParams(window.location.search),e=t.get("view");e&&["grid","canvas","scada","twin","system"].includes(e)&&(this._view=e);let n=t.get("screen");n&&(this._activeScreen=n)}_syncUrl(){let t=new URLSearchParams(window.location.search);t.set("view",this._view),t.set("screen",this._activeScreen),window.history.replaceState(null,"",`${window.location.pathname}?${t}${window.location.hash}`)}connectedCallback(){super.connectedCallback(),this._readUrlParams(),An.addEventListener("change",this._onAuth),An.check(),this._loadScreens(),this._loadAllSensors(),this._loadBranding(),this._loadShortcuts(),this._pollAlarms(),this._pollSystem(),this._fetchVersion(),document.addEventListener("keydown",this._onKeyDown)}disconnectedCallback(){super.disconnectedCallback(),An.removeEventListener("change",this._onAuth),document.removeEventListener("keydown",this._onKeyDown)}async _loadScreens(){try{this._screens=await Kt("/screens")}catch{this._screens=[]}}async _loadAllSensors(){try{this._allSensors=await Kt("/data-sources")}catch{this._allSensors=[]}}async _loadBranding(){try{this._branding=await Kt("/settings/branding"),En.setBranding(this._branding)}catch{}}async _loadShortcuts(){try{let t=await Kt("/shortcuts"),e=await Kt("/nav-shortcuts");En.setShortcuts(t),En.setNavShortcuts(e)}catch{}}async _pollAlarms(){let t=async()=>{try{let n=(await Kt("/alarms/active/count")).count??0;n>this._alerts&&n>0&&Ri.start(),n===0&&Ri.stop(),this._alerts=n}catch{}};t(),setInterval(t,5e3)}async _pollSystem(){let t=async()=>{try{this._systemInfo=await Kt("/system")}catch{}};t(),setInterval(t,3e4)}async _fetchVersion(){try{let e=await(await fetch(`${window.location.origin}/api/version`)).json();this._version=e.version||""}catch{}}_setScreen(t){this._activeScreen=t,this._editMode=!1,this._syncUrl()}_setView(t){this._view=t,this._editMode=!1,this._syncUrl()}_handleLogout(){Ri.stop(),An.logout()}_onSensorClick(t){this._selectedSensorId=t.detail}_closeDetail(){this._selectedSensorId=null}_openAddScreen(){this._dialog={type:"add-screen",title:"Add Screen",label:"Screen Name",okLabel:"Create",initialValue:""}}_openRenameScreen(){let t=this._screens.find(e=>e.id===this._activeScreen);t&&(this._dialog={type:"rename-screen",title:"Rename Screen",label:"Screen Name",okLabel:"Save",initialValue:t.name})}_openDeleteScreen(){this._screens.find(e=>e.id===this._activeScreen)&&(this._dialog={type:"delete-screen",title:"Delete Screen",label:"",okLabel:"Delete",initialValue:""})}async _onDialogSubmit(t){let e=this._dialog;if(this._dialog=null,!e)return;let n=t.detail;try{if(e.type==="add-screen"){let s=await Kt("/screens",{method:"POST",body:JSON.stringify({name:n})});this._activeScreen=s.id,await this._loadScreens()}else e.type==="rename-screen"&&(await Ie(`/api/screens/${this._activeScreen}`,{method:"PUT",body:JSON.stringify({name:n})}),await this._loadScreens())}catch{}}async _onDialogConfirm(){let t=this._dialog;if(this._dialog=null,t?.type==="delete-screen")try{await Ie(`/api/screens/${this._activeScreen}`,{method:"DELETE"}),this._activeScreen="all",await this._loadScreens()}catch{}}_toggleEditMode(){this._editMode=!this._editMode}async _saveLayouts(){if(this._view==="grid"){let t=this.querySelector("vlt-grid-view");t?.saveLayouts&&await t.saveLayouts()}else if(this._view==="canvas"){let t=this.querySelector("vlt-canvas-view");t?.saveLayouts&&await t.saveLayouts()}this._editMode=!1}render(){if(this._authLoading)return Z`<div class="loading">Loading...</div>`;if(!this._authed)return Z`<vlt-login-screen></vlt-login-screen>`;let t=this._allSensors.find(s=>s.id===this._selectedSensorId),e=this._screens.find(s=>s.id===this._activeScreen),n=this._activeScreen!=="all";return Z`
      <div class="header">
        <div class="brand">
          ${this._branding.logo?Z`<img src="${this._branding.logo}" alt="${this._branding.title}" />`:Z`<img src="favicon-32.png" alt="${this._branding.title}" />`}
          <h1>${this._branding.title} <span>${this._branding.subtitle}</span></h1>
        </div>
        <div class="toolbar">
          <button style="position:relative;border-color:${this._alerts>0?"var(--crit)":"var(--border)"};color:${this._alerts>0?"var(--crit)":"var(--fg)"}" @click="${()=>this._showAlarmPanel=!0}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
            ${this._alerts>0?Z`<span style="position:absolute;top:-4px;right:-4px;background:var(--crit);color:#fff;border-radius:50%;width:16px;height:16px;font-size:10px;display:flex;align-items:center;justify-content:center;font-weight:700">${this._alerts}</span>`:""}
          </button>
          <button class="${this._view==="grid"?"active":""}" @click="${()=>this._setView("grid")}">Grid</button>
          <button class="${this._view==="canvas"?"active":""}" @click="${()=>this._setView("canvas")}">Canvas</button>
          <button class="${this._view==="scada"?"active":""}" @click="${()=>this._setView("scada")}">2D SCADA</button>
          <button class="${this._view==="twin"?"active":""}" @click="${()=>this._setView("twin")}">3D Twin</button>
          <button class="${this._view==="system"?"active":""}" @click="${()=>this._setView("system")}">System</button>
          ${this._view==="grid"||this._view==="canvas"?Z`
            <button @click="${this._toggleEditMode}">${this._editMode?"Done":"Edit Layout"}</button>
            ${this._editMode?Z`<button @click="${this._saveLayouts}">Save Layout</button>`:""}
          `:""}
          <div class="sse-badge">
            <span class="sse-dot" style="background:${this._sseConnected?"var(--ok)":"var(--warn)"}"></span>
            ${this._sseConnected?"LIVE":"SSE..."}
          </div>
          <button @click="${()=>this._showSettings=!0}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            Settings
          </button>
          <button @click="${()=>this._showDataManager=!0}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
            Data
          </button>
          <button @click="${this._handleLogout}">Logout</button>
        </div>
      </div>

      <div class="screens">
        <button class="${this._activeScreen==="all"?"active":""}" @click="${()=>this._setScreen("all")}">Overview</button>
        ${this._screens.map(s=>Z`
          <button class="${this._activeScreen===s.id?"active":""}" @click="${()=>this._setScreen(s.id)}">
            ${s.name}
            ${this._activeScreen===s.id?Z`
              <span class="actions">
                <span title="Rename" @click="${r=>{r.stopPropagation(),this._openRenameScreen()}}">✎</span>
                <span title="Delete" @click="${r=>{r.stopPropagation(),this._openDeleteScreen()}}">✕</span>
              </span>
            `:""}
          </button>
        `)}
        <button class="screen-add" title="Add screen" @click="${this._openAddScreen}">+</button>
      </div>

      <main>
        ${this._view==="grid"?Z`<vlt-grid-view .screenId="${this._activeScreen}" .editMode="${this._editMode}" @sensor-click="${this._onSensorClick}"></vlt-grid-view>`:""}
        ${this._view==="canvas"?Z`<vlt-canvas-view .screenId="${this._activeScreen}"></vlt-canvas-view>`:""}
        ${this._view==="scada"?Z`<vlt-scada-view></vlt-scada-view>`:""}
        ${this._view==="twin"?Z`<vlt-twin-view></vlt-twin-view>`:""}
        ${this._view==="system"?Z`<vlt-system-view></vlt-system-view>`:""}
      </main>

      <div class="footer">
        <div class="left">
          <span><span class="dot" style="background:${this._sseConnected?"var(--ok)":"var(--warn)"}"></span>${this._sseConnected?"LIVE":"OFFLINE"}</span>
          <span>${this._systemInfo.hostname||"\u2014"} · ${this._systemInfo.os||"\u2014"}</span>
        </div>
        <div class="right">
          <span>${this._allSensors.length} sensors</span>
          <span>${this._screens.length} screens</span>
          ${this._version?Z`<span>v${this._version}</span>`:""}
        </div>
      </div>

      ${this._showAlarmPanel?Z`<vlt-alarm-panel @close="${()=>this._showAlarmPanel=!1}"></vlt-alarm-panel>`:""}
      ${t?Z`<vlt-sensor-detail .sensor="${t}" .currentValue="${this._currentValues[String(t.id)]}" .allSensors="${this._allSensors}" @back="${this._closeDetail}"></vlt-sensor-detail>`:""}
      ${this._showSettings?Z`<vlt-dashboard-settings @close="${()=>this._showSettings=!1}"></vlt-dashboard-settings>`:""}
      ${this._showDataManager?Z`<vlt-data-manager .isAdmin="${this._authRole==="admin"}" @close="${()=>this._showDataManager=!1}"></vlt-data-manager>`:""}
      ${this._dialog?.type==="add-screen"||this._dialog?.type==="rename-screen"?Z`
        <vlt-prompt-dialog
          .title="${this._dialog.title}"
          .label="${this._dialog.label}"
          .okLabel="${this._dialog.okLabel}"
          .initialValue="${this._dialog.initialValue}"
          @submit="${this._onDialogSubmit}"
          @close="${()=>this._dialog=null}"
        ></vlt-prompt-dialog>
      `:""}
      ${this._dialog?.type==="delete-screen"?Z`
        <vlt-confirm-dialog
          .title="${"Delete Screen"}"
          .message="${`Delete "${e?.name}"? Sensors will remain but be removed from this screen.`}"
          .confirmLabel="${"Delete"}"
          @confirm="${this._onDialogConfirm}"
          @close="${()=>this._dialog=null}"
        ></vlt-confirm-dialog>
      `:""}
    `}};Se([et()],ye.prototype,"_authed",void 0);Se([et()],ye.prototype,"_authLoading",void 0);Se([et()],ye.prototype,"_view",void 0);Se([et()],ye.prototype,"_activeScreen",void 0);Se([et()],ye.prototype,"_screens",void 0);Se([et()],ye.prototype,"_sseConnected",void 0);Se([et()],ye.prototype,"_showAlarmPanel",void 0);Se([et()],ye.prototype,"_selectedSensorId",void 0);Se([et()],ye.prototype,"_showSettings",void 0);Se([et()],ye.prototype,"_showDataManager",void 0);Se([et()],ye.prototype,"_allSensors",void 0);Se([et()],ye.prototype,"_currentValues",void 0);Se([et()],ye.prototype,"_alerts",void 0);Se([et()],ye.prototype,"_branding",void 0);Se([et()],ye.prototype,"_version",void 0);Se([et()],ye.prototype,"_editMode",void 0);Se([et()],ye.prototype,"_dialog",void 0);Se([et()],ye.prototype,"_authRole",void 0);Se([et()],ye.prototype,"_systemInfo",void 0);ye=Se([Zt("vlt-app-shell")],ye);
/*! Bundled license information:

@lit/reactive-element/css-tag.js:
  (**
   * @license
   * Copyright 2019 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/reactive-element.js:
lit-html/lit-html.js:
lit-element/lit-element.js:
@lit/reactive-element/decorators/custom-element.js:
@lit/reactive-element/decorators/property.js:
@lit/reactive-element/decorators/state.js:
@lit/reactive-element/decorators/event-options.js:
@lit/reactive-element/decorators/base.js:
@lit/reactive-element/decorators/query.js:
@lit/reactive-element/decorators/query-all.js:
@lit/reactive-element/decorators/query-async.js:
@lit/reactive-element/decorators/query-assigned-nodes.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-html/is-server.js:
  (**
   * @license
   * Copyright 2022 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query-assigned-elements.js:
  (**
   * @license
   * Copyright 2021 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

three/build/three.core.js:
three/build/three.module.js:
  (**
   * @license
   * Copyright 2010-2026 Three.js Authors
   * SPDX-License-Identifier: MIT
   *)
*/
