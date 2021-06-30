
import React, { useState } from 'react';
import { useRouter } from 'next/router';
import { ThemeProvider, createMuiTheme, makeStyles, withStyles } from '@material-ui/core/styles';
import {
	Container, AppBar, Toolbar, IconButton, Typography, Button, Grid, Link, Box, TextField
} from '@material-ui/core';
import { Menu as MenuIcon } from '@material-ui/icons';
import primary from '@material-ui/core/colors/amber';

const theme = createMuiTheme({
	palette: {
		primary: primary,
	},
});

const useStyles = makeStyles((theme: Theme) => ({
	menuButton: {
		marginRight: theme.spacing(2),
	},
	title: {
		flexGrow: 1,
	},
}));


export class Layout extends React.Component {
	state = {
		query: '',
	}
	render() {
		const router = this.props.router;
		const classes = this.props.classes;
		return <>
			<ThemeProvider theme={theme}>
				<Container>
					<AppBar position="static" color="default">
						<Toolbar>
							<Grid
								justify="space-between"
								container
							>
								<Grid item>
									<Typography variant="h6" className={classes.title}>
										<Link href="/" color="inherit" style={{marginRight: "1em"}}>chainseeker</Link>
										<Button href="https://chainseeker.docs.apiary.io/" target="_blank" rel="noopener">
											Rest API
										</Button>
									</Typography>
								</Grid>
								<Grid item>
									<form onSubmit={(e) => { e.preventDefault(); router.push(`/search/${this.state.query}`); }}>
										<TextField
											id="query" label="blockid, height, txid, address"
											size="small" variant="outlined"
											style={{width: "30em", marginRight: "1em"}}
											onChange={(e) => { this.setState({ query: e.target.value }); }} />
										<Button type="submit" color="inherit">Search</Button>
									</form>
								</Grid>
							</Grid>
						</Toolbar>
					</AppBar>
					<Container style={{marginTop: "3ex", marginBottom: "3ex"}}>
						{this.props.children}
					</Container>
					<hr />
					<Box>
						<p>Copyright &copy; chainseeker 2017-{new Date().getFullYear()}. All rights reserved.</p>
						<p>Created & maintained by <Link href="https://twitter.com/visvirial" target="_blank" rel="noopener">@visvirial</Link>.</p>
					</Box>
				</Container>
			</ThemeProvider>
		</>;
	}
}

export default function layout(props) {
	const router = useRouter();
	const classes = useStyles();
	return <Layout {...props} router={router} classes={classes} />;
};

