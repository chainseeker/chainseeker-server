
import { Middleware } from '@nuxt/types';

const CoinMiddleware: Middleware = ({ params, error, $config }) => {
	const coin = params.coin;
	if(typeof $config.coins[coin] === 'undefined') {
		error({ statusCode: 404, message: 'Coin Not Found.' });
	}
	$config.coin = coin;
	$config.coinConfig = $config.coins[coin];
};

export default CoinMiddleware;

