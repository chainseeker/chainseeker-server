(window.webpackJsonp=window.webpackJsonp||[]).push([[18],{353:function(t,e,n){"use strict";var r=n(0);e.a=r.default.extend({name:"transitionable",props:{mode:String,origin:String,transition:String}})},357:function(t,e,n){var content=n(358);content.__esModule&&(content=content.default),"string"==typeof content&&(content=[[t.i,content,""]]),content.locals&&(t.exports=content.locals);(0,n(15).default)("5276b867",content,!0,{sourceMap:!1})},358:function(t,e,n){var r=n(14)(!1);r.push([t.i,'.theme--light.v-badge .v-badge__badge:after{border-color:#fff}.theme--dark.v-badge .v-badge__badge:after{border-color:#1e1e1e}.v-badge{position:relative}.v-badge,.v-badge__badge{display:inline-block;line-height:1}.v-badge__badge{border-radius:10px;color:#fff;font-size:12px;height:20px;letter-spacing:0;min-width:20px;padding:4px 6px;pointer-events:auto;position:absolute;text-align:center;text-indent:0;top:auto;transition:.3s cubic-bezier(.25,.8,.5,1);white-space:nowrap}.v-application--is-ltr .v-badge__badge{right:auto}.v-application--is-rtl .v-badge__badge{left:auto}.v-badge__badge .v-icon{color:inherit;font-size:12px;height:12px;margin:0 -2px;width:12px}.v-badge__badge .v-img{height:12px;width:12px}.v-badge__wrapper{flex:0 1;height:100%;left:0;pointer-events:none;position:absolute;top:0;width:100%}.v-badge--avatar .v-badge__badge{padding:0}.v-badge--avatar .v-badge__badge .v-avatar{height:20px!important;min-width:0!important;max-width:20px!important}.v-badge--bordered .v-badge__badge:after{border-radius:inherit;border-width:2px;border-style:solid;bottom:0;content:"";left:0;position:absolute;right:0;top:0;transform:scale(1.15)}.v-badge--dot .v-badge__badge{border-radius:4.5px;height:9px;min-width:0;padding:0;width:9px}.v-badge--dot .v-badge__badge:after{border-width:1.5px}.v-badge--icon .v-badge__badge{padding:4px 6px}.v-badge--inline{align-items:center;display:inline-flex;justify-content:center}.v-badge--inline .v-badge__badge,.v-badge--inline .v-badge__wrapper{position:relative}.v-badge--inline .v-badge__wrapper{margin:0 4px}.v-badge--tile .v-badge__badge{border-radius:0}',""]),t.exports=r},382:function(t,e,n){"use strict";n.r(e);var r=n(383),o=n.n(r);for(var c in r)["default"].indexOf(c)<0&&function(t){n.d(e,t,(function(){return r[t]}))}(c);e.default=o.a},383:function(t,e,n){"use strict";n(47);var r=n(210),o=n(155),c=n(209),l=n(156),d=n(157),v=n(158),f=n(77);function h(t){var e=function(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Boolean.prototype.valueOf.call(Reflect.construct(Boolean,[],(function(){}))),!0}catch(t){return!1}}();return function(){var n,r=v(t);if(e){var o=v(this).constructor;n=Reflect.construct(r,arguments,o)}else n=r.apply(this,arguments);return d(this,n)}}n(70),n(12);var m=this&&this.__decorate||function(t,e,n,desc){var r,o=arguments.length,c=o<3?e:null===desc?desc=Object.getOwnPropertyDescriptor(e,n):desc;if("object"===("undefined"==typeof Reflect?"undefined":f(Reflect))&&"function"==typeof Reflect.decorate)c=Reflect.decorate(t,e,n,desc);else for(var i=t.length-1;i>=0;i--)(r=t[i])&&(c=(o<3?r(c):o>3?r(e,n,c):r(e,n))||c);return o>3&&c&&Object.defineProperty(e,n,c),c};Object.defineProperty(e,"__esModule",{value:!0});var _=n(159),x=n(325),y=function(t){l(d,t);var e,n=h(d);function d(){var t;return o(this,d),(t=n.apply(this,arguments)).status=null,t.tx=null,t.blockHeader=null,t.confirmations=null,t}return c(d,[{key:"head",value:function(){return{title:"Transaction ".concat(this.$route.params.id," - chainseeker")}}},{key:"asyncData",value:(e=r(regeneratorRuntime.mark((function t(e){var n,r,o,c,l,d,v,f;return regeneratorRuntime.wrap((function(t){for(;;)switch(t.prev=t.next){case 0:return n=e.params,r=e.error,o=e.$config,c=new x.Chainseeker(o.coinConfig.apiEndpoint),t.next=4,c.getStatus();case 4:return l=t.sent,t.prev=5,t.next=8,c.getTransaction(n.id);case 8:if(d=t.sent,v=null,!d.confirmedHeight){t.next=14;break}return t.next=13,c.getBlockHeader(d.confirmedHeight);case 13:v=t.sent;case 14:return f=d.confirmedHeight?l.blocks-d.confirmedHeight+1:null,t.abrupt("return",{status:l,tx:d,blockHeader:v,confirmations:f});case 18:t.prev=18,t.t0=t.catch(5),r({statusCode:404,message:"Transaction Not Found."});case 21:case"end":return t.stop()}}),t,null,[[5,18]])}))),function(t){return e.apply(this,arguments)})}]),d}(_.Vue);y=m([_.Component],y),e.default=y},401:function(t,e,n){"use strict";n(6),n(5),n(7),n(12),n(13);var r=n(69),o=n(2),c=(n(23),n(357),n(124)),l=n(30),d=n(18),v=n(162),f=n(353),h=n(94),m=n(8),_=n(1),x=["aria-atomic","aria-label","aria-live","role","title"];function y(object,t){var e=Object.keys(object);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(object);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(object,t).enumerable}))),e.push.apply(e,n)}return e}e.a=Object(m.a)(l.a,Object(h.b)(["left","bottom"]),d.a,v.a,f.a).extend({name:"v-badge",props:{avatar:Boolean,bordered:Boolean,color:{type:String,default:"primary"},content:{required:!1},dot:Boolean,label:{type:String,default:"$vuetify.badge"},icon:String,inline:Boolean,offsetX:[Number,String],offsetY:[Number,String],overlap:Boolean,tile:Boolean,transition:{type:String,default:"scale-rotate-transition"},value:{default:!0}},computed:{classes:function(){return function(t){for(var i=1;i<arguments.length;i++){var source=null!=arguments[i]?arguments[i]:{};i%2?y(Object(source),!0).forEach((function(e){Object(o.a)(t,e,source[e])})):Object.getOwnPropertyDescriptors?Object.defineProperties(t,Object.getOwnPropertyDescriptors(source)):y(Object(source)).forEach((function(e){Object.defineProperty(t,e,Object.getOwnPropertyDescriptor(source,e))}))}return t}({"v-badge--avatar":this.avatar,"v-badge--bordered":this.bordered,"v-badge--bottom":this.bottom,"v-badge--dot":this.dot,"v-badge--icon":null!=this.icon,"v-badge--inline":this.inline,"v-badge--left":this.left,"v-badge--overlap":this.overlap,"v-badge--tile":this.tile},this.themeClasses)},computedBottom:function(){return this.bottom?"auto":this.computedYOffset},computedLeft:function(){return this.isRtl?this.left?this.computedXOffset:"auto":this.left?"auto":this.computedXOffset},computedRight:function(){return this.isRtl?this.left?"auto":this.computedXOffset:this.left?this.computedXOffset:"auto"},computedTop:function(){return this.bottom?this.computedYOffset:"auto"},computedXOffset:function(){return this.calcPosition(this.offsetX)},computedYOffset:function(){return this.calcPosition(this.offsetY)},isRtl:function(){return this.$vuetify.rtl},offset:function(){return this.overlap?this.dot?8:12:this.dot?2:4},styles:function(){return this.inline?{}:{bottom:this.computedBottom,left:this.computedLeft,right:this.computedRight,top:this.computedTop}}},methods:{calcPosition:function(t){return"calc(100% - ".concat(Object(_.e)(t||this.offset),")")},genBadge:function(){var t=this.$vuetify.lang,label=this.$attrs["aria-label"]||t.t(this.label),data=this.setBackgroundColor(this.color,{staticClass:"v-badge__badge",style:this.styles,attrs:{"aria-atomic":this.$attrs["aria-atomic"]||"true","aria-label":label,"aria-live":this.$attrs["aria-live"]||"polite",title:this.$attrs.title,role:this.$attrs.role||"status"},directives:[{name:"show",value:this.isActive}]}),e=this.$createElement("span",data,[this.genBadgeContent()]);return this.transition?this.$createElement("transition",{props:{name:this.transition,origin:this.origin,mode:this.mode}},[e]):e},genBadgeContent:function(){if(!this.dot){var slot=Object(_.p)(this,"badge");return slot||(this.content?String(this.content):this.icon?this.$createElement(c.a,this.icon):void 0)}},genBadgeWrapper:function(){return this.$createElement("span",{staticClass:"v-badge__wrapper"},[this.genBadge()])}},render:function(t){var e=[this.genBadgeWrapper()],n=[Object(_.p)(this)],o=this.$attrs,c=(o["aria-atomic"],o["aria-label"],o["aria-live"],o.role,o.title,Object(r.a)(o,x));return this.inline&&this.left?n.unshift(e):n.push(e),t("span",{staticClass:"v-badge",attrs:c,class:this.classes},n)}})},448:function(t,e,n){"use strict";n.d(e,"a",(function(){return r})),n.d(e,"b",(function(){return o}));var r=function(){var t=this,e=t.$createElement,n=t._self._c||e;return t.tx?n("div",[n("h1",[t._v("\n\t\t\tTransaction\n\t\t\t"),n("small",[t._v("["+t._s(t.tx.txid.slice(0,16))+"...]")])]),t._v(" "),n("div",{staticClass:"text-center"},[n("v-badge",{attrs:{color:null===t.confirmations?"red":t.confirmations>=6?"green":"yellow darken-3",content:null===t.confirmations?"unconfirmed":t.confirmations+" confirmations"}})],1),t._v(" "),n("div",{staticClass:"my-4"},[n("v-tooltip",{attrs:{bottom:""},scopedSlots:t._u([{key:"activator",fn:function(e){var r=e.on,o=e.attrs;return[n("v-row",t._g(t._b({staticClass:"my-2"},"v-row",o,!1),r),[n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Transaction ID")])]),t._v(" "),n("v-col",{attrs:{md:"10"}},[t._v(t._s(t.tx.txid))])],1)]}}],null,!1,3566530037)},[t._v(" "),n("span",[t._v("The transaction ID (reverse of transaction hash).")])]),t._v(" "),n("v-tooltip",{attrs:{bottom:""},scopedSlots:t._u([{key:"activator",fn:function(e){var r=e.on,o=e.attrs;return[n("v-row",t._g(t._b({staticClass:"my-2"},"v-row",o,!1),r),[n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Hash")])]),t._v(" "),n("v-col",{attrs:{md:"10"}},[t._v(t._s(t.tx.hash))])],1)]}}],null,!1,3623467837)},[t._v(" "),n("span",[t._v("The hash of the transaction including witness (will coincides with txid if a transaction has no witness data).")])]),t._v(" "),n("v-row",{staticClass:"my-2"},[n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Size")])]),t._v(" "),n("v-col",{attrs:{md:"2"}},[t._v(t._s(t.tx.size.toLocaleString())+" bytes")]),t._v(" "),n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Virtual Size")])]),t._v(" "),n("v-col",{attrs:{md:"2"}},[t._v(t._s(t.tx.vsize.toLocaleString())+" bytes")]),t._v(" "),n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Weight")])]),t._v(" "),n("v-col",{attrs:{md:"2"}},[t._v(t._s(t.tx.weight.toLocaleString())+" WU")])],1),t._v(" "),n("v-row",{staticClass:"my-2"},[n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Version")])]),t._v(" "),n("v-col",{attrs:{md:"4"}},[t._v(t._s(t.tx.version.toLocaleString()))]),t._v(" "),n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Lock Time")])]),t._v(" "),n("v-col",{attrs:{md:"4"}},[t._v(t._s(t.tx.locktime.toLocaleString()))])],1),t._v(" "),n("v-row",{staticClass:"my-2"},[n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Confirmed Height")])]),t._v(" "),n("v-col",{attrs:{md:"4"}},[t.tx.confirmedHeight?n("span",[n("NuxtLink",{attrs:{to:"../block/"+t.tx.confirmedHeight}},[t._v("\n\t\t\t\t\t\t"+t._s(t.tx.confirmedHeight.toLocaleString())+"\n\t\t\t\t\t")]),t._v(" "),n("span",{staticClass:"ml-4"},[t._v("("+t._s(new Date(1e3*t.blockHeader.time).toLocaleString())+")")])],1):n("span",[t._v("\n\t\t\t\t\t"+t._s(t.tx.confirmedHeight?t.tx.confirmedHeight.toLocaleString():"unconfirmed")+"\n\t\t\t\t")])]),t._v(" "),n("v-col",{attrs:{md:"2"}},[n("strong",[t._v("Fee")])]),t._v(" "),n("v-col",{attrs:{md:"4"}},[n("Amount",{attrs:{value:t.tx.fee}}),t._v("\n\t\t\t\t("),n("Amount",{attrs:{value:Math.floor(t.tx.fee/t.tx.size),symbol:t.$config.coinConfig.coin.satoshi+" / byte",unitInSatoshi:!0}}),t._v(")\n\t\t\t")],1)],1)],1),t._v(" "),n("h2",[t._v("Transaction Details")]),t._v(" "),n("div",{staticClass:"my-4"},[n("v-row",[n("v-col",{staticClass:"text-center"},[n("strong",[t._v("Input")])]),t._v(" "),n("v-col",{staticClass:"text-center"},[n("strong",[t._v("Output")])])],1),t._v(" "),n("TxMovement",{attrs:{tx:t.tx}})],1),t._v(" "),n("API",{attrs:{path:"tx/"+t.tx.txid}})],1):t._e()},o=[]},463:function(t,e,n){"use strict";n.r(e);var r=n(448),o=n(382);for(var c in o)["default"].indexOf(c)<0&&function(t){n.d(e,t,(function(){return o[t]}))}(c);var l=n(44),d=n(71),v=n.n(d),f=n(401),h=n(309),m=n(311),_=n(464),component=Object(l.a)(o.default,r.a,r.b,!1,null,null,null);e.default=component.exports,v()(component,{Amount:n(322).default,TxMovement:n(346).default,API:n(345).default}),v()(component,{VBadge:f.a,VCol:h.a,VRow:m.a,VTooltip:_.a})}}]);