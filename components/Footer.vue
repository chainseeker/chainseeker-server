<template>
	<div>
		<hr style="margin-top:5ex;" />
		<footer class="my-2">
			<div class="float-left">
				<img src="~/assets/img/logo-square.png" alt="site logo" style="max-height: 100px;" />
			</div>
			<div class="float-left" style="padding: 15px 20px;">
				<p class="my-0">
					Copyright &copy; chainseeker 2017-{{ new Date().getFullYear() }}. All rights reserved.
				</p>
				<p class="my-0">
					Created &amp; mainted by <NuxtLink to="https://twitter.com/visvirial" target="_blank">@visvirial</NuxtLink>.
				</p>
				<p class="my-0">
					Source code is available at <a href="https://github.com/chainseeker" target="_blank">our GitHub repo</a>.
				</p>
			</div>
		</footer>
	</div>
</template>
