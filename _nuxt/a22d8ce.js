(window.webpackJsonp=window.webpackJsonp||[]).push([[15,8,9,10],{310:function(t,e,r){"use strict";r.r(e);var n=r(311),o=r.n(n);for(var l in n)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return n[t]}))}(l);e.default=o.a},311:function(t,e,r){"use strict";r(47);var n=r(154),o=r(155),l=r(156),d=r(157),c=r(76);function f(t){var e=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(t){return!1}}();return function(){var r,n=d(t);if(e){var o=d(this).constructor;r=Reflect.construct(n,arguments,o)}else r=n.apply(this,arguments);return l(this,r)}}r(12),r(23);var v=this&&this.__decorate||function(t,e,r,desc){var n,o=arguments.length,l=o<3?e:null===desc?desc=Object.getOwnPropertyDescriptor(e,r):desc;if("object"===("undefined"==typeof Reflect?"undefined":c(Reflect))&&"function"==typeof Reflect.decorate)l=Reflect.decorate(t,e,r,desc);else for(var i=t.length-1;i>=0;i--)(n=t[i])&&(l=(o<3?n(l):o>3?n(e,r,l):n(e,r))||l);return o>3&&l&&Object.defineProperty(e,r,l),l};Object.defineProperty(e,"__esModule",{value:!0});var h=r(158),_=function(t){o(r,t);var e=f(r);function r(){return n(this,r),e.apply(this,arguments)}return r}(h.Vue);v([h.Prop({type:Number,required:!0})],_.prototype,"value",void 0),v([h.Prop({type:String,required:!1,default:""})],_.prototype,"symbol",void 0),v([h.Prop({type:Boolean,required:!1,default:!1})],_.prototype,"unitInSatoshi",void 0),_=v([h.Component],_),e.default=_},314:function(t,e){function r(t,e,r,n,o,l,d){try{var c=t[l](d),f=c.value}catch(t){return void r(t)}c.done?e(f):Promise.resolve(f).then(n,o)}t.exports=function(t){return function(){var e=this,n=arguments;return new Promise((function(o,l){var d=t.apply(e,n);function c(t){r(d,o,l,c,f,"next",t)}function f(t){r(d,o,l,c,f,"throw",t)}c(void 0)}))}},t.exports.default=t.exports,t.exports.__esModule=!0},315:function(t,e,r){"use strict";r.r(e);var n=r(319),o=r(310);for(var l in o)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return o[t]}))}(l);var d=r(44),component=Object(d.a)(o.default,n.a,n.b,!1,null,null,null);e.default=component.exports},318:function(t,e,r){"use strict";Object.defineProperty(e,"__esModule",{value:!0}),e.Chainseeker=e.DEFAULT_API_ENDPOINT=void 0,e.DEFAULT_API_ENDPOINT="https://btc.chainseeker.info/api";const n=()=>"undefined"!=typeof fetch?fetch:r(320);e.Chainseeker=class{constructor(t=e.DEFAULT_API_ENDPOINT){this.endpoint=t}async getRestV1(path){const t=await n()(`${this.endpoint}/v1/${path.join("/")}`);if(!t.ok)throw new Error("Failed to call API: "+t.statusText);const e=await t.json();if(e.error)throw new Error("API server responded as an error: "+e.error.toString());return e}getStatus(){return this.getRestV1(["status"])}getBlockHeader(t){return this.getRestV1(["block","number"==typeof t?t.toString():t])}getBlockWithTxids(t){return this.getRestV1(["block_with_txids","number"==typeof t?t.toString():t])}getBlockWithTxs(t){return this.getRestV1(["block_with_txs","number"==typeof t?t.toString():t])}getTransaction(t){return this.getRestV1(["tx",t])}getTxids(address){return this.getRestV1(["txids",address])}getTxs(address){return this.getRestV1(["txs",address])}getUtxos(address){return this.getRestV1(["utxos",address])}async putTransaction(t){const e=await n()(`${this.endpoint}/v1/tx/broadcast`,{method:"PUT",headers:{Accept:"application/json","Content-Type":"text/plain"},body:t});if(!e.ok)throw new Error("Failed to call API: "+e.statusText);const r=await e.json();if(r.error)throw new Error("API server responded as an error: "+r.error.toString());return r}getBlockSummary(t,e){return this.getRestV1(["block_summary",t.toString(),e.toString()])}async getRichListCount(){return(await this.getRestV1(["rich_list_count"])).count}getRichList(t,e){return this.getRestV1(["rich_list",t.toString(),e.toString()])}}},319:function(t,e,r){"use strict";r.d(e,"a",(function(){return n})),r.d(e,"b",(function(){return o}));var n=function(){var t=this,e=t.$createElement,r=t._self._c||e;return r("span",[t._v(t._s(t.unitInSatoshi?t.value:(1e-8*t.value).toFixed(8))+" "),r("small",[t._v(t._s(t.symbol?t.symbol:t.$config.coinConfig.coin.symbol))])])},o=[]},320:function(t,e,r){"use strict";var n=function(){if("undefined"!=typeof self)return self;if("undefined"!=typeof window)return window;if(void 0!==n)return n;throw new Error("unable to locate global object")}();t.exports=e=n.fetch,n.fetch&&(e.default=n.fetch.bind(n)),e.Headers=n.Headers,e.Request=n.Request,e.Response=n.Response},321:function(t,e,r){"use strict";r.r(e);var n=r(322),o=r.n(n);for(var l in n)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return n[t]}))}(l);e.default=o.a},322:function(t,e,r){"use strict";r(47);var n=r(154),o=r(209),l=r(155),d=r(156),c=r(157),f=r(76);function v(t){var e=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(t){return!1}}();return function(){var r,n=c(t);if(e){var o=c(this).constructor;r=Reflect.construct(n,arguments,o)}else r=n.apply(this,arguments);return d(this,r)}}r(12),r(23);var h=this&&this.__decorate||function(t,e,r,desc){var n,o=arguments.length,l=o<3?e:null===desc?desc=Object.getOwnPropertyDescriptor(e,r):desc;if("object"===("undefined"==typeof Reflect?"undefined":f(Reflect))&&"function"==typeof Reflect.decorate)l=Reflect.decorate(t,e,r,desc);else for(var i=t.length-1;i>=0;i--)(n=t[i])&&(l=(o<3?n(l):o>3?n(e,r,l):n(e,r))||l);return o>3&&l&&Object.defineProperty(e,r,l),l};Object.defineProperty(e,"__esModule",{value:!0});var _=r(158),y=function(t){l(r,t);var e=v(r);function r(){return n(this,r),e.apply(this,arguments)}return o(r,[{key:"format",value:function(time){var t=function(t,text){return t>1?"".concat(text,"s"):text},e=Math.abs(time);if(e<1e3)return"".concat(time," ms");if(e<6e4){var r=Math.floor(time/1e3);return"".concat(r," ").concat(t(r,"sec"))}if(e<36e5){var n=Math.floor(time/60/1e3),o=Math.floor(time%6e4/1e3);return"".concat(n," ").concat(t(n,"min")," ").concat(o," ").concat(t(o,"sec"))}if(e<864e5){var l=Math.floor(time/60/60/1e3),d=Math.floor(time%36e5/60/1e3);return"".concat(l," ").concat(t(l,"hour")," ").concat(d," ").concat(t(d,"min"))}var c=Math.floor(time/24/60/60/1e3),f=Math.floor(time%864e5/60/60/1e3);return"".concat(c," ").concat(t(c,"day")," ").concat(f," ").concat(t(f,"hour"))}}]),r}(_.Vue);h([_.Prop({type:Number,required:!0})],y.prototype,"duration",void 0),y=h([_.Component],y),e.default=y},330:function(t,e,r){"use strict";r.r(e);var n=r(331),o=r.n(n);for(var l in n)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return n[t]}))}(l);e.default=o.a},331:function(t,e,r){"use strict";r(47);var n=r(154),o=r(209),l=r(155),d=r(156),c=r(157),f=r(76);function v(t){var e=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(t){return!1}}();return function(){var r,n=c(t);if(e){var o=c(this).constructor;r=Reflect.construct(n,arguments,o)}else r=n.apply(this,arguments);return d(this,r)}}r(12),r(23);var h=this&&this.__decorate||function(t,e,r,desc){var n,o=arguments.length,l=o<3?e:null===desc?desc=Object.getOwnPropertyDescriptor(e,r):desc;if("object"===("undefined"==typeof Reflect?"undefined":f(Reflect))&&"function"==typeof Reflect.decorate)l=Reflect.decorate(t,e,r,desc);else for(var i=t.length-1;i>=0;i--)(n=t[i])&&(l=(o<3?n(l):o>3?n(e,r,l):n(e,r))||l);return o>3&&l&&Object.defineProperty(e,r,l),l};Object.defineProperty(e,"__esModule",{value:!0});var _=r(158),y=function(t){l(r,t);var e=v(r);function r(){var t;return n(this,r),(t=e.call(this)).elapsedTime=0,setInterval(t.update,100),t}return o(r,[{key:"update",value:function(){this.elapsedTime=Date.now()-this.time}}]),r}(_.Vue);h([_.Prop({type:Number,required:!0})],y.prototype,"time",void 0),y=h([_.Component],y),e.default=y},332:function(t,e,r){var content=r(333);content.__esModule&&(content=content.default),"string"==typeof content&&(content=[[t.i,content,""]]),content.locals&&(t.exports=content.locals);(0,r(15).default)("7c06aa28",content,!0,{sourceMap:!1})},333:function(t,e,r){var n=r(14)(!1);n.push([t.i,".theme--light.v-data-table{background-color:#fff;color:rgba(0,0,0,.87)}.theme--light.v-data-table .v-data-table__divider{border-right:thin solid rgba(0,0,0,.12)}.theme--light.v-data-table.v-data-table--fixed-header thead th{background:#fff;box-shadow:inset 0 -1px 0 rgba(0,0,0,.12)}.theme--light.v-data-table>.v-data-table__wrapper>table>thead>tr>th{color:rgba(0,0,0,.6)}.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>td:last-child,.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>td:not(.v-data-table__mobile-row),.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>th:last-child,.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>th:not(.v-data-table__mobile-row),.theme--light.v-data-table>.v-data-table__wrapper>table>thead>tr:last-child>th{border-bottom:thin solid rgba(0,0,0,.12)}.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr.active{background:#f5f5f5}.theme--light.v-data-table>.v-data-table__wrapper>table>tbody>tr:hover:not(.v-data-table__expanded__content):not(.v-data-table__empty-wrapper){background:#eee}.theme--light.v-data-table>.v-data-table__wrapper>table>tfoot>tr>td:not(.v-data-table__mobile-row),.theme--light.v-data-table>.v-data-table__wrapper>table>tfoot>tr>th:not(.v-data-table__mobile-row){border-top:thin solid rgba(0,0,0,.12)}.theme--dark.v-data-table{background-color:#1e1e1e;color:#fff}.theme--dark.v-data-table .v-data-table__divider{border-right:thin solid hsla(0,0%,100%,.12)}.theme--dark.v-data-table.v-data-table--fixed-header thead th{background:#1e1e1e;box-shadow:inset 0 -1px 0 hsla(0,0%,100%,.12)}.theme--dark.v-data-table>.v-data-table__wrapper>table>thead>tr>th{color:hsla(0,0%,100%,.7)}.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>td:last-child,.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>td:not(.v-data-table__mobile-row),.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>th:last-child,.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr:not(:last-child)>th:not(.v-data-table__mobile-row),.theme--dark.v-data-table>.v-data-table__wrapper>table>thead>tr:last-child>th{border-bottom:thin solid hsla(0,0%,100%,.12)}.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr.active{background:#505050}.theme--dark.v-data-table>.v-data-table__wrapper>table>tbody>tr:hover:not(.v-data-table__expanded__content):not(.v-data-table__empty-wrapper){background:#616161}.theme--dark.v-data-table>.v-data-table__wrapper>table>tfoot>tr>td:not(.v-data-table__mobile-row),.theme--dark.v-data-table>.v-data-table__wrapper>table>tfoot>tr>th:not(.v-data-table__mobile-row){border-top:thin solid hsla(0,0%,100%,.12)}.v-data-table{line-height:1.5;max-width:100%}.v-data-table>.v-data-table__wrapper>table{width:100%;border-spacing:0}.v-data-table>.v-data-table__wrapper>table>tbody>tr>td,.v-data-table>.v-data-table__wrapper>table>tbody>tr>th,.v-data-table>.v-data-table__wrapper>table>tfoot>tr>td,.v-data-table>.v-data-table__wrapper>table>tfoot>tr>th,.v-data-table>.v-data-table__wrapper>table>thead>tr>td,.v-data-table>.v-data-table__wrapper>table>thead>tr>th{padding:0 16px;transition:height .2s cubic-bezier(.4,0,.6,1)}.v-data-table>.v-data-table__wrapper>table>tbody>tr>th,.v-data-table>.v-data-table__wrapper>table>tfoot>tr>th,.v-data-table>.v-data-table__wrapper>table>thead>tr>th{-webkit-user-select:none;-moz-user-select:none;-ms-user-select:none;user-select:none;font-size:.75rem;height:48px}.v-application--is-ltr .v-data-table>.v-data-table__wrapper>table>tbody>tr>th,.v-application--is-ltr .v-data-table>.v-data-table__wrapper>table>tfoot>tr>th,.v-application--is-ltr .v-data-table>.v-data-table__wrapper>table>thead>tr>th{text-align:left}.v-application--is-rtl .v-data-table>.v-data-table__wrapper>table>tbody>tr>th,.v-application--is-rtl .v-data-table>.v-data-table__wrapper>table>tfoot>tr>th,.v-application--is-rtl .v-data-table>.v-data-table__wrapper>table>thead>tr>th{text-align:right}.v-data-table>.v-data-table__wrapper>table>tbody>tr>td,.v-data-table>.v-data-table__wrapper>table>tfoot>tr>td,.v-data-table>.v-data-table__wrapper>table>thead>tr>td{font-size:.875rem;height:48px}.v-data-table__wrapper{overflow-x:auto;overflow-y:hidden}.v-data-table__progress{height:auto!important}.v-data-table__progress th{height:auto!important;border:none!important;padding:0;position:relative}.v-data-table--dense>.v-data-table__wrapper>table>tbody>tr>td,.v-data-table--dense>.v-data-table__wrapper>table>tbody>tr>th,.v-data-table--dense>.v-data-table__wrapper>table>tfoot>tr>td,.v-data-table--dense>.v-data-table__wrapper>table>tfoot>tr>th,.v-data-table--dense>.v-data-table__wrapper>table>thead>tr>td,.v-data-table--dense>.v-data-table__wrapper>table>thead>tr>th{height:32px}.v-data-table--has-top>.v-data-table__wrapper>table>tbody>tr:first-child:hover>td:first-child{border-top-left-radius:0}.v-data-table--has-top>.v-data-table__wrapper>table>tbody>tr:first-child:hover>td:last-child{border-top-right-radius:0}.v-data-table--has-bottom>.v-data-table__wrapper>table>tbody>tr:last-child:hover>td:first-child{border-bottom-left-radius:0}.v-data-table--has-bottom>.v-data-table__wrapper>table>tbody>tr:last-child:hover>td:last-child{border-bottom-right-radius:0}.v-data-table--fixed-header>.v-data-table__wrapper,.v-data-table--fixed-height .v-data-table__wrapper{overflow-y:auto}.v-data-table--fixed-header>.v-data-table__wrapper>table>thead>tr>th{border-bottom:0!important;position:sticky;top:0;z-index:2}.v-data-table--fixed-header>.v-data-table__wrapper>table>thead>tr:nth-child(2)>th{top:48px}.v-application--is-ltr .v-data-table--fixed-header .v-data-footer{margin-right:17px}.v-application--is-rtl .v-data-table--fixed-header .v-data-footer{margin-left:17px}.v-data-table--fixed-header.v-data-table--dense>.v-data-table__wrapper>table>thead>tr:nth-child(2)>th{top:32px}",""]),t.exports=n},340:function(t,e,r){"use strict";r.d(e,"a",(function(){return n})),r.d(e,"b",(function(){return o}));var n=function(){var t=this,e=t.$createElement;return(t._self._c||e)("span",[t._v("\n\t"+t._s(t.format(t.duration))+"\n")])},o=[]},344:function(t,e,r){"use strict";r.r(e);var n=r(340),o=r(321);for(var l in o)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return o[t]}))}(l);var d=r(44),component=Object(d.a)(o.default,n.a,n.b,!1,null,null,null);e.default=component.exports},345:function(t,e,r){"use strict";r(6),r(5),r(7),r(12),r(13);var n=r(2),o=(r(23),r(332),r(1)),l=r(18),d=r(8);function c(object,t){var e=Object.keys(object);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(object);t&&(r=r.filter((function(t){return Object.getOwnPropertyDescriptor(object,t).enumerable}))),e.push.apply(e,r)}return e}e.a=Object(d.a)(l.a).extend({name:"v-simple-table",props:{dense:Boolean,fixedHeader:Boolean,height:[Number,String]},computed:{classes:function(){return function(t){for(var i=1;i<arguments.length;i++){var source=null!=arguments[i]?arguments[i]:{};i%2?c(Object(source),!0).forEach((function(e){Object(n.a)(t,e,source[e])})):Object.getOwnPropertyDescriptors?Object.defineProperties(t,Object.getOwnPropertyDescriptors(source)):c(Object(source)).forEach((function(e){Object.defineProperty(t,e,Object.getOwnPropertyDescriptor(source,e))}))}return t}({"v-data-table--dense":this.dense,"v-data-table--fixed-height":!!this.height&&!this.fixedHeader,"v-data-table--fixed-header":this.fixedHeader,"v-data-table--has-top":!!this.$slots.top,"v-data-table--has-bottom":!!this.$slots.bottom},this.themeClasses)}},methods:{genWrapper:function(){return this.$slots.wrapper||this.$createElement("div",{staticClass:"v-data-table__wrapper",style:{height:Object(o.e)(this.height)}},[this.$createElement("table",this.$slots.default)])}},render:function(t){return t("div",{staticClass:"v-data-table",class:this.classes},[this.$slots.top,this.genWrapper(),this.$slots.bottom])}})},352:function(t,e,r){"use strict";r.r(e);var n=r(353),o=r.n(n);for(var l in n)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return n[t]}))}(l);e.default=o.a},353:function(t,e,r){"use strict";r(47);var n=r(314),o=r(154),l=r(209),d=r(155),c=r(156),f=r(157),v=r(76);function h(t){var e=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(t){return!1}}();return function(){var r,n=f(t);if(e){var o=f(this).constructor;r=Reflect.construct(n,arguments,o)}else r=n.apply(this,arguments);return c(this,r)}}r(78),r(12);var _=this&&this.__decorate||function(t,e,r,desc){var n,o=arguments.length,l=o<3?e:null===desc?desc=Object.getOwnPropertyDescriptor(e,r):desc;if("object"===("undefined"==typeof Reflect?"undefined":v(Reflect))&&"function"==typeof Reflect.decorate)l=Reflect.decorate(t,e,r,desc);else for(var i=t.length-1;i>=0;i--)(n=t[i])&&(l=(o<3?n(l):o>3?n(e,r,l):n(e,r))||l);return o>3&&l&&Object.defineProperty(e,r,l),l};Object.defineProperty(e,"__esModule",{value:!0});var y=r(158),m=r(318),w=function(t){d(c,t);var e,r=h(c);function c(){var t;return o(this,c),(t=r.apply(this,arguments)).recentBlocks=[],t.recentTxs=[],t}return l(c,[{key:"initWebSocket",value:function(){var t=this,e=new m.Chainseeker(this.$config.coinConfig.apiEndpoint);new WebSocket(this.$config.coinConfig.wsEndpoint).onmessage=function(){var r=n(regeneratorRuntime.mark((function r(n){var data;return regeneratorRuntime.wrap((function(r){for(;;)switch(r.prev=r.next){case 0:data=JSON.parse(n.data),r.t0=data[0],r.next="hashtx"===r.t0?4:"hashblock"===r.t0?13:20;break;case 4:return r.t1=t.recentTxs,r.t2=Date.now(),r.next=8,e.getTransaction(data[1]);case 8:return r.t3=r.sent,r.t4={received:r.t2,tx:r.t3},r.t1.unshift.call(r.t1,r.t4),t.recentTxs.length>5&&t.recentTxs.splice(0,t.recentTxs.length-5),r.abrupt("break",20);case 13:return r.t5=t.recentBlocks,r.next=16,e.getBlockHeader(data[1]);case 16:return r.t6=r.sent,r.t5.unshift.call(r.t5,r.t6),t.recentBlocks.pop(),r.abrupt("break",20);case 20:case"end":return r.stop()}}),r)})));return function(t){return r.apply(this,arguments)}}()}},{key:"asyncData",value:(e=n(regeneratorRuntime.mark((function t(e){var r,n,o,l,d;return regeneratorRuntime.wrap((function(t){for(;;)switch(t.prev=t.next){case 0:return e.params,e.error,r=e.$config,n=new m.Chainseeker(r.coinConfig.apiEndpoint),t.next=4,n.getStatus();case 4:o=t.sent,l=[],d=o.blocks;case 7:if(!(d>=o.blocks-5)){t.next=16;break}return t.t0=l,t.next=11,n.getBlockHeader(d);case 11:t.t1=t.sent,t.t0.push.call(t.t0,t.t1);case 13:d--,t.next=7;break;case 16:return t.abrupt("return",{status:o,recentBlocks:l});case 17:case"end":return t.stop()}}),t)}))),function(t){return e.apply(this,arguments)})},{key:"mounted",value:function(){this.initWebSocket()}}]),c}(y.Vue);w=_([y.Component],w),e.default=w},377:function(t,e,r){"use strict";r.d(e,"a",(function(){return n})),r.d(e,"b",(function(){return o}));var n=function(){var t=this,e=t.$createElement,r=t._self._c||e;return r("span",[r("Duration",{attrs:{duration:t.elapsedTime}})],1)},o=[]},380:function(t,e,r){"use strict";r.r(e);var n=r(377),o=r(330);for(var l in o)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return o[t]}))}(l);var d=r(44),component=Object(d.a)(o.default,n.a,n.b,!1,null,null,null);e.default=component.exports,installComponents(component,{Duration:r(344).default})},434:function(t,e,r){"use strict";r.d(e,"a",(function(){return n})),r.d(e,"b",(function(){return o}));var n=function(){var t=this,e=t.$createElement,r=t._self._c||e;return r("div",[r("h1",[t._v("Recent Blocks")]),t._v(" "),r("v-simple-table",{scopedSlots:t._u([{key:"default",fn:function(){return[r("thead",[r("tr",[r("th",[t._v("Height")]),t._v(" "),r("th",[t._v("Time")]),t._v(" "),r("th",[t._v("# of txs")]),t._v(" "),r("th",[t._v("Size")]),t._v(" "),r("th",[t._v("ID")])])]),t._v(" "),r("tbody",t._l(t.recentBlocks,(function(e){return r("tr",[r("td",[r("NuxtLink",{attrs:{to:"./block/"+e.height,append:""}},[t._v(t._s(e.height))])],1),t._v(" "),r("td",[r("ElapsedTime",{attrs:{time:1e3*e.time}}),t._v(" ago")],1),t._v(" "),r("td",[t._v(t._s(e.ntxs))]),t._v(" "),r("td",[t._v(t._s(e.size.toLocaleString())+" bytes")]),t._v(" "),r("td",[t._v(t._s(e.hash)+" bytes")])])})),0)]},proxy:!0}])}),t._v(" "),r("h1",[t._v("Recent Transactions")]),t._v(" "),r("v-simple-table",{scopedSlots:t._u([{key:"default",fn:function(){return[r("thead",[r("tr",[r("th",[t._v("Received at")]),t._v(" "),r("th",[t._v("Transaction ID")]),t._v(" "),r("th",[t._v("# of vins")]),t._v(" "),r("th",[t._v("# of vouts")]),t._v(" "),r("th",[t._v("Size")]),t._v(" "),r("th",[t._v("Value Transacted")])])]),t._v(" "),r("tbody",t._l(t.recentTxs,(function(e){var n=e.received,o=e.tx;return r("tr",[r("td",[r("ElapsedTime",{attrs:{time:n}}),t._v(" ago")],1),t._v(" "),r("td",[r("NuxtLink",{attrs:{to:"./tx/"+o.txid,append:""}},[t._v(t._s(o.txid))])],1),t._v(" "),r("td",[t._v(t._s(o.vin.length.toLocaleString()))]),t._v(" "),r("td",[t._v(t._s(o.vout.length.toLocaleString()))]),t._v(" "),r("td",[t._v(t._s(o.size.toLocaleString())+" bytes")]),t._v(" "),r("td",[r("Amount",{attrs:{value:o.vout.reduce((function(t,e){return t+e.value}),0)}})],1)])})),0)]},proxy:!0}])})],1)},o=[]},447:function(t,e,r){"use strict";r.r(e);var n=r(434),o=r(352);for(var l in o)["default"].indexOf(l)<0&&function(t){r.d(e,t,(function(){return o[t]}))}(l);var d=r(44),c=r(55),f=r.n(c),v=r(345),component=Object(d.a)(o.default,n.a,n.b,!1,null,null,null);e.default=component.exports,f()(component,{ElapsedTime:r(380).default,Amount:r(315).default}),f()(component,{VSimpleTable:v.a})}}]);