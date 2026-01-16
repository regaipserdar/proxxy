
import { useQuery } from '@apollo/client';
import { TEST_CONNECTION, GET_HTTP_TRANSACTIONS } from '../graphql/operations';

export function GraphQLTest() {
  const { data: testData, loading: testLoading, error: testError } = useQuery(TEST_CONNECTION);
  const { data: requestsData, loading: requestsLoading, error: requestsError } = useQuery(GET_HTTP_TRANSACTIONS, {
    variables: { agentId: null }
  });

  return (
    <div className="p-4 bg-gray-900 text-white">
      <h2 className="text-xl font-bold mb-4">GraphQL Connection Test</h2>

      <div className="mb-4">
        <h3 className="text-lg font-semibold">Test Connection:</h3>
        {testLoading && <p>Loading...</p>}
        {testError && <p className="text-red-500">Error: {testError.message}</p>}
        {testData && <p className="text-green-500">Success: {testData.hello}</p>}
      </div>

      <div className="mb-4">
        <h3 className="text-lg font-semibold">HTTP Transactions:</h3>
        {requestsLoading && <p>Loading...</p>}
        {requestsError && <p className="text-red-500">Error: {requestsError.message}</p>}
        {requestsData && (
          <div>
            <p className="text-green-500">Success: {requestsData.requests?.length || 0} requests found</p>
            {requestsData.requests?.slice(0, 3).map((req: any) => (
              <div key={req.requestId} className="ml-4 text-sm">
                <p>{req.method} {req.url}</p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}