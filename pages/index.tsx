
import React from 'react';
import Head from 'next/head';
import { useRouter } from 'next/router';
import Layout from '../components/layout';
import {
	TableContainer, Table, TableHead, TableBody, TableRow, TableCell, Paper
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import useSWR from 'swr';

import { Chainseeker } from 'chainseeker';
import { config } from '../config';

const MAX_LATEST_BLOCKS = 5;

const cs = new Chainseeker(config.apiEndpoint);

const useStyles = makeStyles({
	table: {
		minWidth: 650,
	},
});

const fetcher = async () => {
	//const status = await cs.getStatus();
	const status = await (await fetch(`${config.apiEndpoint}/v1/status`)).json();
	return {
		status,
	};
};

class Home extends React.Component {
	static async getInitialProps() {
		const status = await cs.getStatus();
		console.log(status);
		/*
		const recentBlocks = [];
		for(let height=status.blocks; height>status.blocks-MAX_LATEST_BLOCKS; height--) {
			recentBlocks.push(await cs.getBlock(height));
		}
		console.log(recentBlocks);
		*/
		return {
			status,
			//recentBlocks,
		};
	}
	render() {
		const router = this.props.router;
		const classes = this.props.classes;
		const data = this.props.data;
		return <Layout>
			<Head>
				<title>chainseeker.info - an open source block explorer</title>
			</Head>
			<div>
				<h1>Recent Blocks</h1>
				<TableContainer component={Paper}>
					<Table className={classes.table} aria-label="simple table">
						<TableHead>
							<TableRow>
								<TableCell>Height</TableCell>
							</TableRow>
						</TableHead>
						<TableBody>
							{/*
							{this.props.recentBlocks.map((block) => (
								<TableRow key={row.name}>
									<TableCell component="th" scope="row">
										{block.height}
									</TableCell>
								</TableRow>
							))}
							*/}
						</TableBody>
					</Table>
				</TableContainer>
				Hello, world!
				{data.status.blocks}
			</div>
		</Layout>;
	}
}

export default function home(props) {
	const router = useRouter();
	const classes = useStyles();
	const { data, error } = useSWR(config.apiEndpoint, fetcher);
	console.log(error);
	if(error) return <div>failed to load.</div>;
	if(!data) return <div>loading...</div>;
	console.log(data);
	return <Home {...props} router={router} classes={classes} data={data} />;
};

