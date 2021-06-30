
import Head from 'next/head';
import Layout from '../components/layout';

export default function Home() {
	return <Layout>
		<Head>
			<title>chainseeker.info - an open source block explorer</title>
		</Head>
		<div>
			Hello, world!
		</div>
	</Layout>;
}

