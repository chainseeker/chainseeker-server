<template>
	<div>
		<h1>Block Size</h1>
		<p>This is a plot of the block size in KiB.</p>
		<ul>
			<li>Size: the block size in KB.</li>
			<li>Stripped Size: the block size excluding witness data in KB.</li>
			<li>Weight: the block weight.</li>
		</ul>
		<div id="plot" style="width:100%; height:600px;"></div>
	</div>
</template>

<script lang="ts">
import { Context } from '@nuxt/types';
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';
import * as d3 from 'd3';

@Component
export default class Home extends Vue {
	blockSummary: cs.BlockSummaryEntry[] = [];
	head() {
		return { title: 'Block Size Stat - chainseeker' };
	}
	drawGraph() {
		// Fetch statistical data and compute averages and standard deviations.
		const computeStat = (arr: number[]): { avg: number, stddev: number } => {
			const avg = arr.reduce((i, a) => i+a, 0) / arr.length;
			const variance = arr.reduce((i, a) => i+(a-avg)*(a-avg), 0) / arr.length;
			return {
				avg: avg,
				stddev: Math.pow(variance, 0.5),
			};
		};
		const DATAPOINTS = 1000;
		const INTERVAL = Math.floor(this.blockSummary.length / DATAPOINTS);
		const data = [];
		for(let i=0; i<DATAPOINTS; i++) {
			const points = this.blockSummary.slice(INTERVAL*i, INTERVAL*(i+1));
			const s0 = computeStat(points.map((p) => p.size));
			const s1 = computeStat(points.map((p) => p.strippedsize));
			const s2 = computeStat(points.map((p) => p.weight));
			data.push([INTERVAL*i, 0.001*s0.avg, 0.001*s0.stddev, 0.001*s1.avg, 0.001*s1.stddev, 0.001*s2.avg, 0.001*s2.stddev]);
		}
		const WIDTH = document.getElementById('plot')!.clientWidth;
		const HEIGHT = document.getElementById('plot')!.clientHeight;
		const MARGIN = {
			top: 10,
			bottom: 50,
			left: 50,
			right: 10,
		};
		// Append SVG element.
		const svg = d3.select('#plot')
			.append('svg')
				.attr('width', WIDTH)
				.attr('height', HEIGHT)
				.append('g')
		// Create x-axis.
		const x = d3.scaleLinear()
			.domain([0, data[data.length-1][0]])
			.range([MARGIN.left, WIDTH-MARGIN.left-MARGIN.right]);
		svg.append('g')
			.attr('transform', 'translate(0,' + (HEIGHT-MARGIN.top-MARGIN.bottom) + ')')
			.call(d3.axisBottom(x));
		// Create y-axis.
		const y = d3.scaleLinear()
			.domain([0, d3.max(data, function(d) { return Math.max(d[1]+0.5*d[2], d[3]+0.5*d[4], d[5]+0.5*d[6]); }) as number])
			.range([HEIGHT-MARGIN.top-MARGIN.bottom, MARGIN.top]);
		svg.append('g')
			.attr('transform', 'translate(' + MARGIN.left + ', 0)')
			.call(d3.axisLeft(y));
		// Legend.
		const legend = svg.selectAll('.legends')
			.data(['Size', 'Stripped Size', 'Weight'])
			.enter()
				.append('g')
			.attr('class', 'legends')
			.attr('transform', (d, i) => 'translate(' + (MARGIN.left + 10) + ', ' + (MARGIN.top + 10 + 20*i) + ')');
		legend.append('rect')
			.attr('x', 5)
			.attr('y', 3)
			.attr('width', 20)
			.attr('height', 4)
			.style('fill', (d, i) => d3.schemeCategory10[i]);
		legend.append('text')
			.attr('x', 30)
			.attr('y', 10)
			.text((d, i) => d)
			.attr('class', 'textselected')
			.style('text-anchor', 'start')
			.style('font-size', 15);
		// Plot.
		for(let i=0; i<3; i++) {
			svg.append('path')
				.datum(data)
				.attr('fill', d3.schemeCategory10[i])
				.attr('d', d3.area()
					.x((d) => x(d[0]))
					.y0((d) => y(d[2*i+1]-0.5*d[2*i+2]))
					.y1((d) => y(d[2*i+1]+0.5*d[2*i+2])) as any
				);
		}
	}
	async asyncData({ params, error, $config }: Context) {
		const cs = new Chainseeker($config.apiEndpoint);
		const status = await cs.getStatus();
		const blockSummary = await cs.getBlockSummary(0, status.blocks);
		return {
			status,
			blockSummary,
		};
	}
	mounted() {
		this.drawGraph();
	}
}
</script>

