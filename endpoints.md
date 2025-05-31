# API Endpoints

**Pagination**: Most endpoints support pagination via `page` and `per_page` query parameters. These endpoints return a `total` field in the response to indicate the total number of items available. Default values are `page=1` and `per_page=20`. The actual data is then returned in the `data` field of the response. Pagination also implies searchability, with a `query` parameter to filter results based on a search term. For locations, this search would typically filter on `name`, `description`, and `excerpt`. For profiles, it would filter on `username`, `email`, `firstName`, and `lastName`.

**Includes**: Many endpoints support the `include` query parameter to conditionally fetch related entities (e.g., `institution`, `authority`, `tags`). These fields are indicated with the `?` symbol on the fields in the response model.

## Authentication

### `POST /auth/login`

Login with username (_which can be the username or email address_) and password (non-SSO users).

**Body**

```typescript
{
	username: string;
	password: string;
}
```

**Response**

-   200 OK, sets auth cookie

---

### `POST /auth/sso/{provider}`

Login with SSO provider.

**Path Parameters**

-   `provider`: SSO provider name (e.g., `google`, `smartschool`, `hogent`, `ugent`)

---

### `POST /auth/logout`

Logout the current user.

**Response**

-   204 No Content, clears auth cookie

---

### `POST /auth/register`

Register a new profile (non-SSO users).

**Body**

```typescript
{
	username: string;
	password: string;
	firstName: string;
	lastName: string;
	email: string;
}
```

**Response**

-   201 Created

---

### `GET /auth/me`

Get current authenticated user profile.

**Response**

```typescript
Profile {
	id: number;
	username: string;
	email: string;
	firstName: string;
	lastName: string;
	state: ProfileState;
	avatarUrl: string | null;
	insitution: Institution | null;
}
```

---

## Institutions

### `GET /institutions`

List all institutions.

**Response**

```typescript
Institution[] {
  name: string;
  email: string|null;
  phone: string|null;
  street: string|null;
  number: string|null;
  zip: string|null;
  city: string|null;
  province: string|null;
  country: string|null;
}
```

---

## Profiles

### `GET /profiles`

List all profiles (paginated).

**Response**

```typescript
Profile[] {
  id: number;
  username: string|null;
  email: string;
  firstName: string;
  lastName: string;
  state: ProfileState;
  avatarUrl: string|null;
  institutionName: string|null;
}
```

---

### `GET /profiles/{id}`

Get a specific profile.

**Response**

```typescript
Profile {
  id: number;
  username: string|null;
  email: string;
  firstName: string;
  lastName: string;
  state: ProfileState;
  institutionName: string|null;
  avatarUrl: string|null;
  authorities?: Authority[];
}
```

---

### `PATCH /profiles/{id}`

Update a profile. (Only for non-SSO users)

**Body**

```typescript
{
  email?: string; // Sets pending email
  firstName?: string;
  lastName?: string;
  password?: string;
  passwordConfirmation?: string;
  insitutionName?: string; // Only for admins
  state?: ProfileState; // Only for admins
}
```

---

### `POST /profiles/{id}/avatar`

Upload a profile avatar image. The image should be a valid image file (e.g., PNG, JPEG).

**Body**

```typescript
{
	file: File;
}
```

---

### `DELETE /profiles/{id}/avatar`

Delete a profile avatar image. The endpoint should check if the user has permission to delete the avatar (e.g., the profile itself or an admin). Special caution should be taken to ensure that the avatar image is deleted from the filesystem.

---

### `POST /profiles/{id}/block`

Block a user (admins).

**Body**

```typescript
{
	reason: string | null;
}
```

---

### `POST /profiles/{id}/unblock`

Unblock a user (admins). Also clears the reason for blocking.

---

### `GET /profiles/{id}/authorities`

List all authorities a profile is a member of.

**Response**

```typescript
AuthorityProfile[] {
  authority: Authority;
  permissions: string[];
}
```

---

### `GET /profiles/{id}/reservations`

List all reservations made by a profile.

**Response**

```typescript
Reservation[] {
  id: number;
  openingTime: OpeningTime;
  baseBlockIndex: number;
  blockCount: number; // Number of consecutive blocks reserved
  startTime: string; // Computed start time of the reservation block
  endTime: string; // Computed end time of the reservation block
  confirmedAt: string|null;
  confirmedBy?: Profile|null;
  updatedAt: string;
  createdAt: string;
}
```

### `GET /profiles/{id}/reviews`

List all reviews made by a profile.

**Response**

```typescript
Review[] {
    id: number;
    locationId: number;
    rating: number;
    body: string|null;
    createdAt: string;
    updatedAt: string;
}
```

---

## `GET /profiles/{id}/locations`

List all locations a profile has permissions to manage.

**Response**

```typescript
LocationProfile[] {
    location: Location;
    permissions: string[];
}
```

---

## Authorities

### `GET /authorities`

List all authorities (paginated).

**Response**

```typescript
PaginatedResponse<Authority[]> {
  page: number;
  perPage: number;
  total: number;
  data: Authority[] {
    id: number;
    name: string;
    description: string|null;
    createdBy?: Profile|null;
    updatedBy?: Profile|null;
    members?: Profile[];
    locations?: Location[];
  }
}
```

### `GET /authorities/permissions`

List all available authority permissions.

**Response**

```typescript
{
  manage_members: number;
  manage_locations: number;
  manage_tags: number;
  manage_opening_times: number;
  manage_reservations: number;
  ...
}
```

---

### `POST /authorities`

Create an authority.

**Body**

```typescript
{
	name: string;
	description: string | null;
}
```

---

### `PATCH /authorities/{id}`

Update authority.

**Body**

```typescript
{
    name?: string;
    description?: string|null;
}
```

---

### `GET /authorities/{id}/locations`

List all locations for an authority.

**Response**

```typescript
Location[] {
    id: number;
    name: string;
    description: Translation;
    excerpt: Translation;
    seatCount: number;
    isReservable: boolean;
    isVisible: boolean;
    street: string;
    number: string;
    zip: string;
    city: string;
    province: string;
    latitude: number;
    longitude: number;
    images?: Image[] {
        id: number;
        url: string;
    };
    approvedBy?: Profile|null;
    approvedAt?: string|null;
    createdBy?: Profile|null;
    createdAt: string;
    updatedBy?: Profile|null;
    updatedAt: string;
}
```

---

### `POST /authorities/{id}/locations`

Create a new location for an authority.


**Body**

```typescript
{
	name: string;
	description: Translation;
	excerpt: Translation;
	seatCount: number;
	isReservable: boolean;
	reservationBlockSize: number;
	minReservationLength: number | null;
	maxReservationLength: number | null;
	isVisible: boolean;
	street: string;
	number: string;
	zip: string;
	city: string;
	province: string;
	latitude: number;
	longitude: number;
}
```

---

### `GET /authorities/{id}/members`

List members of an authority.

**Response**

```typescript
Profile[] {
  id: number;
  username: string|null;
  email: string;
  firstName: string;
  lastName: string;
  state: ProfileState;
  permissions: string[];
  institution?: Institution|null;
  avatarUrl?: string|null;
}
```

---

### `POST /authorities/{id}/members`

Add profile to authority.

**Body**

```typescript
{
  profileId: number,
  permissions: string[];
}
```

---

### `DELETE /authorities/{id}/members/{profileId}`

Remove profile from authority.

---

### `POST /authorities/{id}/members/{profileId}/permissions`

Update permissions for a profile in an authority.

**Body**

```typescript
{
  permissions: string[];
}
```

---

## Locations

### `GET /locations/search`

Search for locations (`is_visible` = `true`). Filters can be applied via query parameters:

-   `language`: Language code (e.g. `nl`, `en`, `fr`, `de`)
-   `query`: Search query (filters on `name`, `description`, `excerpt`)
-   `center_lat`: Latitude of the center point for distance search
-   `center_lng`: Longitude of the center point for distance search
-   `distance`: Distance in meters from the center point (default: 1000)
-   `north_east_lat`: Latitude of the northeast corner for bounding box search
-   `north_east_lng`: Longitude of the northeast corner for bounding box search
-   `south_west_lat`: Latitude of the southwest corner for bounding box search
-   `south_west_lng`: Longitude of the southwest corner for bounding box search

**Response**

```typescript
Partial<Location> {
  id: number;
  name: string;
  city: string;
  province: string;
  latitude: number;
  longitude: number;
}
```

---

### `GET /locations`

List all locations (paginated).

**Response**

```typescript
PaginatedResponse<Location[]> {
  page: number;
  perPage: number;
  total: number;
  data: Location[] {
    id: number;
    name: string;
    description: Translation;
    excerpt: Translation;
    seatCount: number;
    isReservable: boolean;
    reservationBlockSize: number;
    minReservationLength: number;
    maxReservationLength: number;
    isVisible: boolean;
    street: string;
    number: string;
    zip: string;
    city: string;
    province: string;
    latitude: number;
    longitude: number;
    images?: Image[] {
        id: number;
        url: string;
    };
    openingTimes?: OpeningTime[]; // Opening times for the current week (monday - sunday)
    tags?: Tag[];
    authority?: Authority|null;
    approvedBy?: Profile|null; // Only available to admins
    approvedAt?: string|null; // Only available to admins
    createdBy?: Profile|null;
    createdAt: string;
    updatedBy?: Profile|null; // Only available to admins
    updatedAt: string;
  }
}
```

---

### `GET /locations/permissions`

List all available location permissions.

**Response**

```typescript
{
  manage_members: number;
  manage_images: number;
  manage_opening_times: number;
  manage_reservations: number;
  manage_tags: number;
  manage_location: number;
  ...
}
```

---

### `GET /locations/{id}`

Get a single location.

**Response**

```typescript
Location {
  id: number;
  name: string;
  description: Translation;
  excerpt: Translation;
  seatCount: number;
  reservationBlockSize: number;
  minReservationLength: number|null;
  maxReservationLength: number|null;
  isReservable: boolean;
  isVisible: boolean;
  street: string;
  number: string;
  zip: string;
  city: string;
  province: string;
  latitude: number;
  longitude: number;
  images?: Image[] {
      id: number;
      url: string;
  };
  openingTimes?: OpeningTime[]; // Opening times for the current week (monday - sunday)
  tags?: Tag[];
  authority?: Authority|null;
  approvedBy?: Profile|null; // Only available to admins
  approvedAt?: string|null; // Only available to admins
  createdBy?: Profile|null;
  createdAt: string;
  updatedBy?: Profile|null; // Only available to admins
  updatedAt: string;
}
```

---

### `POST /locations`

Create a new location.

**Body**

```typescript
{
	name: string;
	description: Translation;
	excerpt: Translation;
	seatCount: number;
	isReservable: boolean;
	reservationBlockSize: number;
	minReservationLength: number | null;
	maxReservationLength: number | null;
	isVisible: boolean;
	street: string;
	number: string;
	zip: string;
	city: string;
	province: string;
	latitude: number;
	longitude: number;
}
```

---

### `POST /locations/{id}/images`

Upload one or more images for a location. The images should be valid image files (e.g., PNG, JPEG). The endpoint should check if the user has permission to upload images for the location.

**Body**

```typescript
{
  files: File[];
}
```

---

### `DELETE /locations/{id}/images/{imageId}`

Delete an image for a location. The endpoint should check if the user has permission to delete images for the location.

---

### `PATCH /locations/{id}`

Update a location.

**Body**

```typescript
{
  name?: string,
  seatCount?: number;
  isReservable?: boolean;
  reservationBlockSize?: number;
  description?: Translation;
  excerpt?: Translation;
  minReservationLength?: number|null;
  maxReservationLength?: number|null;
  isVisible?: boolean,
  street?: string,
  number?: string,
  zip?: string,
  city?: string,
  province?: string,
  latitude?: number,
  longitude?: number
}
```

---

### `DELETE /locations/{id}`

Delete a location. Special caution should be taken to ensure that associated images are deleted from the filesystem. `ON DELETE` constraints in the database should handle other related entities.

---

### `POST /locations/{id}/approve`

Approve a location (admins). This sets the `approvedBy` and `approvedAt` fields. Clears the `rejected_at`, `rejected_by`, and `rejected_reason` fields if they exist.

---

### `POST /locations/{id}/reject`

Reject a location (admins). Clears the `approved_by` and `approved_at` fields if they exist.

**Body**

```typescript
{
	reason: string | null; // Reason for rejection
}
```

---

### `POST /locations/{id}/tags`

Sets the location's tags. As implied by the POST method, this replaces the existing tags with the provided ones.

**Body**

```typescript
{
  "tags": string[]
}
```

---

### `GET /locations/{id}/members`

List members of a location. Members are profiles that have permissions to manage the location.

**Response**

```typescript
Profile[] {
  id: number;
  username: string|null;
  email: string;
  firstName: string;
  lastName: string;
  state: ProfileState;
  permissions: string[];
  institution?: Institution|null;
}
```

---

### `POST /locations/{id}/members`

Add profile to a location to manage.

**Body**

```typescript
{
  profileId: number;
  permissions: string[];
}
```

---

### `DELETE /locations/{id}/members/{profileId}`

Remove profile from a location.

---

### `POST /locations/{id}/members/{profileId}/permissions`

Update permissions for a profile in a location.

**Body**

```typescript
{
  permissions: string[];
}
```

---

## Opening Times

### `GET /locations/{id}/opening-times`

List opening times for a location. Filters can be applied via query parameters:

-   `start_date`: Minimum date for opening times (format: `YYYY-MM-DD`). Default: monday of the current week. Maximum date is 6 months before the current date. Admins can override this limit.
-   `end_date`: Maximum date for opening times (format: `YYYY-MM-DD`). Default: sunday of the current week. Maximum date is 6 months after the current date. Admins can override this limit.

**Response**

```typescript
OpeningTime[] {
  id: number;
  startTime: string;
  endTime: string;
  seatCount: number|null;
  reservableFrom: string|null;
  reservableUntil: string|null;
  createdBy?: Profile|null;
  createdAt: string;
  updatedBy?: Profile|null; // Only available to admins
  updatedAt: string;
}
```

---

### `POST /locations/{id}/opening-times`

Create an opening time for a location. Should check if the user has permission to create opening times for the location, as well as if the opening time does not overlap with existing opening times.

**Body**

```typescript
{
  startTime: string,
  endTime: string,
  seatCount?: number|null,
  reservableFrom?: string|null,
  reservableUntil?: string|null
}
```

---

### `PATCH /locations/{id}/opening-times/{id}`

Update an opening time for a location. Should check if the user has permission to update opening times for the location, as well as if the updated opening time does not overlap with existing opening times.

**Body**

```typescript
{
  startTime?: string,
  endTime?: string,
  seatCount?: number|null,
  reservableFrom?: string|null,
  reservableUntil?: string|null
}
```

---

### `DELETE /locations/{id}/opening-times/{id}`

Delete an opening time for a location. Should check if the user has permission to delete opening times for the location.

---

### `GET /locations/{id}/opening-times/{id}/reservations`

List reservations for an opening time.

**Response**

```typescript
Reservation[] {
  id: number;
  baseBlockIndex: number;
  blockCount: number; // Number of consecutive blocks reserved
  startTime: string; // Computed start time of the reservation block (first block index of the reservation)
  endTime: string; // Computed end time of the reservation block (last block index of the reservation)
  profile: Profile;
  confirmedAt: string|null;
  confirmedBy?: Profile|null;
  updatedAt: string;
  createdAt: string;
}
```

---

### `GET /locations/{id}/reservations?date={date}`

List reservations for a location. If `date` is present in the query, it should return reservations for that specific date. If not, it should return all (paginated) reservations for the location, ordered by id.

**Response**

```typescript
Reservation[] {
  id: number;
  openingTime?: OpeningTime;
  baseBlockIndex: number;
  blockCount: number; // Number of consecutive blocks reserved
  startTime: string; // Computed start time of the reservation block
  endTime: string; // Computed end time of the reservation block
  profile: Profile;
  confirmedAt: string|null;
}
```

---

### `POST /locations/{id}/opening-times/{id}/reservations`

Create a reservation for an opening time. This endpoint checks the following:

1. The start and end times are within the opening time.
2. Convert the start and end times to block indices based on the `reservationBlockSize` of the opening time's location.
3. Per block index, count the existing reservations and ensure the number of reservations does not exceed the `seatCount` of the opening time / location.

**Body**

```typescript
{
  startTime: Time,
  endTime: Time
}
```

---

### `DELETE /locations/{id}/opening-times/{id}/reservations/{reservationId}`

Delete a reservations for a profile in an opening time. This endpoint should check if the user has permission to delete reservations for the opening time.

---

## Tags

### `GET /tags`

List all tags. No pagination needed, as the list will be quite small.

**Response**

```typescript
Tag[] {
  id: string;
  name: Translation;
}
```

---

### `POST /tags`

Create a new tag. Only for admins.

**Body**

```typescript
{
  id: string;
  name: Translation {
    nl: string,
    en: string,
    fr: string,
    de: string
  }
}
```

---

### `PATCH /tags/{id}`

Update an existing tag's name in the different languages.

**Body**

```typescript
{
  name?: Translation {
    nl?: string|null,
    en?: string|null,
    fr?: string|null,
    de?: string|null
  }
}
```

---

### `DELETE /tags/{id}`

Delete a tag.

---

## Translations

### `POST /translations`

Create a translation.

**Body**

```typescript
{
  nl: string,
  en: string,
  fr: string,
  de: string
}
```

---

### `PATCH /translations/{id}`

Update a translation.

**Body**

```typescript
{
  nl?: string|null,
  en?: string|null,
  fr?: string|null,
  de?: string|null
}
```

---

### `DELETE /translations/{id}`

Delete a translation.

---

## Reviews

### `GET /locations/{id}/reviews`

List (paginated) reviews for a location.

**Response**

```typescript
PaginatedResponse<Review[]> {
  page: number;
  perPage: number;
  total: number;
  data: Review[] {
    id: number;
    profile: Partial<Profile> {
        firstName: string; // Only first name is needed for non-admins
        // Admins can see the full profile
    };
    rating: number;
    body: string|null;
    createdAt: string;
    updatedAt: string;
  }
}
```

---

### `POST /locations/{id}/reviews`

Create a review. Check if the user did not already review the location.

**Body**

```typescript
{
  rating: number,
  body: string|null
}
```

---

### `PATCH /locations/{id}/reviews/{reviewId}`

Update a review. Check if the user is the author of the review or an admin.

**Body**

```typescript
{
  rating?: number,
  body?: string|null
}
```
