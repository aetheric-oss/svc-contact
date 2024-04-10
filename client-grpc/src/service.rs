//! Client Library: Client Functions, Structs, Traits

/// gRPC object traits to provide wrappers for grpc functions
#[tonic::async_trait]
pub trait Client<T>
where
    Self: Sized + lib_common::grpc::Client<T> + lib_common::grpc::ClientConnect<T>,
    T: Send + Clone,
{
    /// The type expected for ReadyRequest structs.
    type ReadyRequest;
    /// The type expected for ReadyResponse structs.
    type ReadyResponse;
    /// The type expected for CargoConfirmationRequest structs.
    type CargoConfirmationRequest;
    /// The type expected for CargoConfirmationResponse structs.
    type CargoConfirmationResponse;

    /// Returns a [`tonic::Response`] containing a [`ReadyResponse`](Self::ReadyResponse)
    /// Takes an [`ReadyRequest`](Self::ReadyRequest).
    ///
    /// # Errors
    ///
    /// Returns [`tonic::Status`] with [`tonic::Code::Unknown`] if the server is not ready.
    ///
    /// # Examples
    /// ```
    /// use lib_common::grpc::get_endpoint_from_env;
    /// use svc_contact_client_grpc::prelude::*;
    ///
    /// async fn example () -> Result<(), Box<dyn std::error::Error>> {
    ///     let (host, port) = get_endpoint_from_env("SERVER_HOSTNAME", "SERVER_PORT_GRPC");
    ///     let client = ContactClient::new_client(&host, port, "contact");
    ///     let response = client
    ///         .is_ready(contact::ReadyRequest {})
    ///         .await?;
    ///     println!("RESPONSE={:?}", response.into_inner());
    ///     Ok(())
    /// }
    /// ```
    async fn is_ready(
        &self,
        request: Self::ReadyRequest,
    ) -> Result<tonic::Response<Self::ReadyResponse>, tonic::Status>;

    /// Returns a [`tonic::Response`] containing a [`CargoConfirmationResponse`](Self::CargoConfirmationResponse)
    /// Takes an [`CargoConfirmationRequest`](Self::CargoConfirmationRequest).
    ///
    /// # Errors
    ///
    /// Returns [`tonic::Status`] with [`tonic::Code::Unknown`] if the server is not ready.
    ///
    /// # Examples
    /// ```
    /// use lib_common::grpc::get_endpoint_from_env;
    /// use svc_contact_client_grpc::prelude::*;
    ///
    /// async fn example () -> Result<(), Box<dyn std::error::Error>> {
    ///     let (host, port) = get_endpoint_from_env("SERVER_HOSTNAME", "SERVER_PORT_GRPC");
    ///     let client = ContactClient::new_client(&host, port, "contact");
    ///     let response = client
    ///         .cargo_confirmation(contact::CargoConfirmationRequest {
    ///             parcel_id: uuid::Uuid::new_v4().to_string(),
    ///             itinerary_id: uuid::Uuid::new_v4().to_string(),
    ///         })
    ///         .await?;
    ///     println!("RESPONSE={:?}", response.into_inner());
    ///     Ok(())
    /// }
    /// ```
    async fn cargo_confirmation(
        &self,
        request: Self::CargoConfirmationRequest,
    ) -> Result<tonic::Response<Self::CargoConfirmationResponse>, tonic::Status>;
}
